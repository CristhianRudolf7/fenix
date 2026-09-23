"""Destination policies are operator-owned and fail closed before runner startup."""
import asyncio
import importlib.util
from pathlib import Path
import socket
import unittest
from unittest.mock import AsyncMock, patch
from test_execution import Fixture, ALICE, BOB
from fenix_execution.egress import effective, pin, validate
from fenix_execution.docker import Docker
from fenix_isolation import PolicyError


class EgressTests(unittest.TestCase):
    def test_account_cannot_widen_global_policy(self):
        self.assertEqual(effective({"web": ["example.com"]}, {"web": ["example.com", "evil.test"]}, "web"), ("example.com",))
        self.assertIsNone(effective({}, {}, "web"))
        self.assertEqual(effective({"web": []}, {}, "web"), ())
        self.assertEqual(effective({}, {"web": ["example.com"]}, "offline"), ())
        for host in ["*.example.com", "https://example.com", "example.com:443", "a..com", "EXAMPLE.com", "example.com/path"]:
            with self.assertRaises(PolicyError): validate({"web": [host]})

    def test_dns_is_pinned_and_mixed_private_answers_fail_closed(self):
        def row(ip): return (socket.AF_INET, socket.SOCK_STREAM, 6, "", (ip, 443))
        with patch("socket.getaddrinfo", return_value=[row("93.184.216.34")]):
            self.assertEqual(pin(("example.com",)), {"example.com": ("93.184.216.34",)})
            with self.assertRaises(PolicyError): pin(("example.com",), ["93.184.216.34"])
        for addresses in [[], [row("127.0.0.1")], [row("224.0.0.1")], [row("93.184.216.34"), row("169.254.169.254")]]:
            with patch("socket.getaddrinfo", return_value=addresses):
                with self.assertRaises(PolicyError): pin(("example.com",))

    def test_runner_cannot_supply_permissions_and_restricted_dns_is_disabled(self):
        f = Fixture()
        try:
            f.config["network_hosts"] = {"development": ["example.com"]}
            spec = f.spec(); spec["profile"] = "development"
            with self.assertRaises(PolicyError):
                f.policy.launch(f.accounts[0], dict(spec, network_hosts=["evil.test"]))
            launch = f.policy.launch(f.accounts[0], spec)
            try:
                self.assertEqual(launch.egress_hosts, ("example.com",))
                backend = object.__new__(Docker)
                backend.instance = "test"; backend.runtime = "runc"; backend.image = "sha256:" + "a" * 64
                with self.assertRaises(PolicyError): backend.create_args(BOB, launch, "test.slice")
                launch.egress_pins = {"example.com": ("93.184.216.34",)}
                args = backend.create_args(BOB, launch, "test.slice")
                self.assertEqual(args[args.index("--dns") + 1], "127.0.0.1")
                self.assertIn("example.com:93.184.216.34", args)
            finally: launch.close()
        finally: f.close()

    def test_firewall_denies_dns_unlisted_ips_and_empty_allowlists(self):
        spec = importlib.util.spec_from_file_location("guard", Path(__file__).resolve().parents[2] / "scripts/tenant-network-guard.py")
        guard = importlib.util.module_from_spec(spec); spec.loader.exec_module(guard)
        rules = guard.policy("FXtest", "FHtest", "web", ["1.1.1.1"], [], ["93.184.216.34"])
        allowed = [r for r in rules if r[1] == "FXtest" and r[-1] == "RETURN"]
        self.assertEqual(len(allowed), 1)
        self.assertIn("93.184.216.34/32", allowed[0])
        self.assertNotIn("53", sum(rules, []))
        empty = guard.policy("FXtest", "FHtest", "web", ["1.1.1.1"], [], [])
        self.assertFalse(any(r[1] == "FXtest" and r[-1] == "RETURN" for r in empty))

    def test_monitor_reuses_frozen_policy_and_rejects_missing_policy(self):
        async def check():
            backend = object.__new__(Docker)
            backend.command = AsyncMock(); backend.guard = Path("guard.py"); backend.deny_args = []
            backend.egress_policies = {ALICE: {"example.com": ("93.184.216.34",)}}
            await backend.guard_check(ALICE, "web")
            args = backend.command.call_args.args
            self.assertIn("--check", args); self.assertIn("--restricted", args)
            self.assertIn("93.184.216.34", args)
            with self.assertRaises(PolicyError): await backend.guard_check(BOB, "web")
        asyncio.run(check())
