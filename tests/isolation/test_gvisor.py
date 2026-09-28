import dataclasses
"""gVisor policy tests; real runtime acceptance is a separate explicit suite."""
import asyncio
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import AsyncMock, patch
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from fenix_execution.backends import backend
from fenix_execution.gvisor import Bundle, GATE_PREFIX, GVisor, RUNSC_ARGS, gate_report, host_requirements, merge_runtime, runtime_entry, sentry_cgroup
from fenix_execution.runtime_assets import digest
from fenix_isolation import PolicyError, private_write
from test_execution import ALICE, BOB, Fixture


class RuntimeFixture:
    def __init__(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.runsc = self.root / "runsc"
        self.runsc.write_text("fixture"); self.runsc.chmod(0o755)
        sidecars = self.root / "gvisor-bin"; sidecars.mkdir(mode=0o755)
        self.sentry = sidecars / "gvisor_sentry"
        self.sentry.write_text("fixture"); self.sentry.chmod(0o755)
        self.gate = self.root / "gate.py"; self.gate.write_text("fixture"); self.gate.chmod(0o644)
        self.manifest = self.root / "manifest.json"
        self.data = dict(schema=1, runtime="fenix-runsc", platform="systrap", runsc=str(self.runsc), gate=str(self.gate),
                         runtime_args=list(RUNSC_ARGS), assets={str(p): digest(p) for p in (self.runsc, self.sentry, self.gate)})
        self.save()

    def save(self): private_write(self.manifest, json.dumps(self.data).encode())
    def bundle(self): return Bundle(self.manifest, owner=os.geteuid())
    def close(self): self.temp.cleanup()


class GVisorPolicyTests(unittest.TestCase):
    def setUp(self): self.f = RuntimeFixture()
    def tearDown(self): self.f.close()

    def test_default_backend_requires_pinned_gvisor_and_never_falls_back(self):
        with patch("fenix_execution.gvisor.host_requirements"), patch("fenix_execution.gvisor.Docker.__init__") as docker:
            with self.assertRaisesRegex(PolicyError, "pinned gVisor"):
                backend({})
            with self.assertRaisesRegex(PolicyError, "fenix-runsc"):
                backend(dict(runtime="runc"))
            docker.assert_not_called()

    def test_systrap_host_check_does_not_open_kvm(self):
        with patch("fenix_execution.gvisor.platform.system", return_value="Linux"), patch("fenix_execution.gvisor.platform.machine", return_value="x86_64"), patch("fenix_execution.gvisor.platform.release", return_value="6.12.0"), patch("fenix_execution.gvisor.Path.is_file", return_value=True), patch("os.open", side_effect=AssertionError("Unexpected device access")):
            host_requirements()

    def test_registration_preserves_other_runtimes_and_refuses_conflicts(self):
        old = {"default-runtime": "runc", "runtimes": {"other": {"path": "/bin/other"}}}
        new = merge_runtime(old, self.f.runsc)
        self.assertEqual(new["default-runtime"], "runc")
        self.assertEqual(new["runtimes"]["other"], old["runtimes"]["other"])
        self.assertNotIn("fenix-runsc", old["runtimes"])
        with self.assertRaises(PolicyError): merge_runtime(new, "/another/runsc")

    def test_alias_cannot_point_at_runc_or_enable_host_network(self):
        bundle = self.f.bundle()
        good = {"Runtimes": {"fenix-runsc": runtime_entry(self.f.runsc)}}
        bundle.check_registration(good)
        for key, value in (("path", "/usr/bin/runc"), ("runtimeArgs", ["--platform=kvm"]),
                           ("runtimeArgs", RUNSC_ARGS + ["--network=host"])):
            bad = copy.deepcopy(good); bad["Runtimes"]["fenix-runsc"][key] = value
            with self.subTest(key=key, value=value):
                with self.assertRaises(PolicyError): bundle.check_registration(bad)

    def test_bundle_rejects_changed_or_missing_sidecars(self):
        bundle = self.f.bundle(); bundle.unchanged()
        self.f.sentry.write_text("altered")
        with self.assertRaises(PolicyError): bundle.unchanged()
        with self.assertRaises(PolicyError): self.f.bundle()
        self.f.data["assets"].pop(str(self.f.sentry)); self.f.save()
        with self.assertRaises(PolicyError): self.f.bundle()

    def test_new_sidecar_is_not_silently_trusted(self):
        bundle = self.f.bundle()
        (self.f.sentry.parent / "unreviewed").write_text("extra")
        with self.assertRaises(PolicyError): bundle.unchanged()
        with self.assertRaises(PolicyError): self.f.bundle()

    def test_equal_metadata_cannot_hide_changed_executable_bytes(self):
        bundle = self.f.bundle()
        self.f.sentry.write_text("altered")
        with patch("fenix_execution.gvisor.stamp", side_effect=lambda path: bundle.stamps[Path(path)]):
            with self.assertRaisesRegex(PolicyError, "checksum changed"):
                bundle.unchanged()

    def test_world_writable_asset_and_unsafe_manifest_flags_are_rejected(self):
        self.f.runsc.chmod(0o777)
        with self.assertRaises(PolicyError): self.f.bundle()
        self.f.runsc.chmod(0o755)
        self.f.data["runtime_args"] = ["--network=host"]; self.f.save()
        with self.assertRaises(PolicyError): self.f.bundle()

    def test_gated_command_preserves_scope_and_never_requests_kvm(self):
        fixture = Fixture()
        try:
            adapter = object.__new__(GVisor)
            adapter.bundle = self.f.bundle(); adapter.instance = "test"; adapter.runtime = "fenix-runsc"
            adapter.image = "sha256:" + "a" * 64; adapter.config = {}
            adapter.mount_dir = fixture.root / "mounts"
            launch = fixture.policy.launch(fixture.accounts[0], fixture.spec())
            try:
                args = adapter.create_args(BOB, launch, "account.slice")
                self.assertIn("/opt/fenix/sandbox-gate.py", args)
                self.assertIn("/bin/true", args)
                self.assertIn("--read-only", args)
                self.assertNotIn("--privileged", args)
                self.assertNotIn("/dev/kvm", " ".join(args))
                self.assertEqual(launch.argv, ["/bin/true"])
                with patch("fenix_execution.gvisor.host_resolver", return_value=b"nameserver 192.0.2.53\n") as resolver:
                    web = dataclasses.replace(launch, profile="web", egress_hosts=None)
                    args = adapter.create_args(BOB, web, "account.slice")
                    self.assertIn(f"type=bind,src={fixture.root}/gvisor-resolv.conf,dst=/etc/resolv.conf,readonly", args)
                    resolver.assert_called_once()
                    resolver.reset_mock()
                    restricted = dataclasses.replace(web, egress_hosts=(), egress_pins={})
                    args = adapter.create_args(BOB, restricted, "account.slice")
                    resolver.assert_not_called()
                    self.assertIn("127.0.0.1", args)
                    adapter.create_args(BOB, launch, "account.slice")
                    resolver.assert_not_called()
            finally: launch.close()
        finally: fixture.close()

    def test_invalid_startup_report_is_rejected(self):
        good = GATE_PREFIX + b'{"backend":"gvisor","kernel":"4.4.0"}\n'
        self.assertEqual(gate_report(good)["backend"], "gvisor")
        for bad in (b"runc\n", GATE_PREFIX + b"[]\n", GATE_PREFIX + b"{bad}\n", GATE_PREFIX + b"x" * 5000):
            with self.subTest(report=bad[:30]):
                with self.assertRaises(PolicyError): gate_report(bad)

    def test_sentry_must_have_pinned_executable_exact_owner_and_host_seccomp(self):
        process = self.f.root / "proc/123"; process.mkdir(parents=True)
        (process / "exe").symlink_to(self.f.sentry)
        (process / "cmdline").write_bytes(b"gvisor_sentry\0--platform=systrap\0--network=sandbox\0--directfs=false\0--ignore-cgroups=false\0boot\0")
        cid = "a" * 64
        group = "/fenixacct.slice/fenixacct-" + ALICE.replace("-", "") + ".slice/docker-" + cid + ".scope"
        (process / "cgroup").write_text("0::" + group + "\n")
        (process / "status").write_text("Seccomp:\t2\nNoNewPrivs:\t1\n")
        self.assertEqual(sentry_cgroup(ALICE, cid, 123, self.f.sentry, process.parent), group)
        with self.assertRaises(PolicyError): sentry_cgroup(BOB, cid, 123, self.f.sentry, process.parent)
        with self.assertRaises(PolicyError): sentry_cgroup(ALICE, cid, 123, self.f.runsc, process.parent)
        args = (process / "cmdline").read_bytes()
        (process / "cmdline").write_bytes(args.replace(b"systrap", b"kvm"))
        with self.assertRaises(PolicyError): sentry_cgroup(ALICE, cid, 123, self.f.sentry, process.parent)
        (process / "cmdline").write_bytes(args)
        (process / "status").write_text("Seccomp:\t0\nNoNewPrivs:\t1\n")
        with self.assertRaises(PolicyError): sentry_cgroup(ALICE, cid, 123, self.f.sentry, process.parent)


class GVisorGateTests(unittest.IsolatedAsyncioTestCase):
    async def test_gate_on_host_refuses_to_execute_user_command(self):
        gate = Path(__file__).resolve().parents[2] / "scripts/sandbox-gate.py"
        process = await asyncio.create_subprocess_exec(sys.executable, "-I", str(gate),
            sys.executable, "-c", "print('must-not-run')", stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
        try:
            out, err = await asyncio.wait_for(process.communicate(), 5)
            self.assertEqual(process.returncode, 125, err)
            self.assertNotIn(b"must-not-run", out)
        finally:
            if process.returncode is None: process.kill(); await process.wait()

    async def test_unverified_sandbox_cannot_be_released(self):
        adapter = object.__new__(GVisor); adapter.bundle = unittest.mock.Mock(); adapter.verified = set()
        process = AsyncMock(); process.returncode = None
        with self.assertRaises(PolicyError): await adapter.release(ALICE, process)
        process.stdin.write.assert_not_called()
