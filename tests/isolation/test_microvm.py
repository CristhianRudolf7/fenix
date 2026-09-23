"""MicroVM policy, real startup-gate protocol and persistent workspace tests.

These run without KVM and do not claim to boot a virtual machine. The opt-in
test_microvm_acceptance suite separately exercises real Kata/Cloud Hypervisor.
"""
import asyncio
import copy
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import AsyncMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from fenix_execution.microvm import Bundle, GATE_PREFIX, MicroVM, digest, guest_identity, require_kvm, validate_configuration, vm_cgroup, wrapper_text
from fenix_execution.backends import backend
from fenix_execution.policy import Policy
from fenix_execution.service import Execution, Records, Service
from fenix_execution.workspaces import environment, variables
from fenix_isolation import PolicyError, private_write
from test_execution import ALICE, BOB, Fixture

SCRIPTS = Path(__file__).resolve().parents[2] / "scripts"
spec = importlib.util.spec_from_file_location("prepare_microvm", SCRIPTS / "prepare-microvm.py")
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


def configuration():
    return dict(hypervisor=dict(clh=dict(shared_fs="virtio-fs", virtio_fs_cache="never",
        disable_block_device_use=True, disable_seccomp=False, enable_debug=False,
        enable_annotations=[], disable_nested_virtualization=True, guest_hook_path="",
        default_memory=256, default_maxmemory=4096, default_vcpus=1, default_maxvcpus=4)),
        runtime=dict(sandbox_cgroup_only=True, disable_guest_seccomp=False, disable_new_netns=False,
                     internetworking_model="tcfilter", sandbox_bind_mounts=[], enable_debug=False,
                     enable_tracing=False, enable_pprof=False, experimental=[], static_sandbox_resource_mgmt=True),
        agent=dict(kata=dict(enable_debug=False, enable_tracing=False, debug_console_enabled=False)))


class RuntimePolicyTests(unittest.TestCase):
    def test_runtime_registration_preserves_existing_daemon_settings(self):
        existing = {"default-runtime": "runc", "log-driver": "local", "runtimes": {"custom": {"path": "/bin/custom"}}}
        merged = prepare.merge_runtime(existing, "/opt/fenix/shim")
        self.assertEqual(merged["default-runtime"], "runc")
        self.assertEqual(merged["runtimes"]["custom"], existing["runtimes"]["custom"])
        self.assertNotIn("fenix-kata", existing["runtimes"])
        with self.assertRaises(PolicyError): prepare.merge_runtime(merged, "/other/shim")

    def test_kvm_failure_and_unknown_backend_never_construct_docker(self):
        with patch("fenix_execution.microvm.os.open", side_effect=FileNotFoundError), patch("fenix_execution.microvm.Docker.__init__") as docker:
            with self.assertRaisesRegex(PolicyError, "fallback is disabled"):
                backend(dict(backend="microvm"))
            docker.assert_not_called()
        with self.assertRaises(PolicyError): backend(dict(backend="e2b-cloud"))

    def test_unsafe_runtime_settings_are_rejected(self):
        validate_configuration(configuration())
        cases = [("runtime", "disable_guest_seccomp", True), ("runtime", "disable_new_netns", True),
                 ("runtime", "sandbox_cgroup_only", False), ("runtime", "sandbox_bind_mounts", ["/"]),
                 ("hypervisor", "disable_seccomp", True), ("hypervisor", "enable_annotations", [".*"]),
                 ("hypervisor", "shared_fs", "none"), ("hypervisor", "virtio_fs_cache", "always")]
        for section, key, value in cases:
            with self.subTest(key=key):
                data = configuration()
                (data[section]["clh"] if section == "hypervisor" else data[section])[key] = value
                with self.assertRaises(PolicyError): validate_configuration(data)
        data = configuration(); data["factory"] = dict(vm_cache_number=1)
        with self.assertRaises(PolicyError): validate_configuration(data)

    def test_host_kernel_and_malformed_guest_reports_are_rejected(self):
        line = GATE_PREFIX + json.dumps(dict(boot_id=ALICE, kernel="guest")).encode() + b"\n"
        self.assertEqual(guest_identity(line, BOB), ALICE)
        for candidate in (line, b"not-a-guest\n", GATE_PREFIX + b"x" * 5000):
            with self.assertRaises(PolicyError): guest_identity(candidate, ALICE)

    def test_bundle_checksums_changes_and_wrapper_binding(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            paths = {}
            for name in ("shim", "wrapper", "gate", "hypervisor", "kernel", "image", "virtiofsd"):
                path = root / name; path.write_text("test-asset"); path.chmod(0o700); paths[name] = path
            data = configuration()
            data["hypervisor"]["clh"].update(path=str(paths["hypervisor"]), kernel=str(paths["kernel"]),
                    image=str(paths["image"]), virtio_fs_daemon=str(paths["virtiofsd"]))
            config = root / "configuration-clh.toml"; config.write_text(prepare.toml(data)); config.chmod(0o600)
            paths["wrapper"].write_text(wrapper_text(paths["shim"], config))
            manifest = root / "manifest.json"
            value = dict(schema=1, runtime="fenix-kata", configuration=str(config),
                         **{name: str(paths[name]) for name in ("shim", "wrapper", "gate")},
                         assets={str(p): digest(p) for p in [config, *paths.values()]})
            private_write(manifest, json.dumps(value).encode())
            bundle = Bundle(manifest, owner=os.geteuid()); bundle.unchanged()
            paths["gate"].write_text("changed")
            with self.assertRaises(PolicyError): bundle.unchanged()
            with self.assertRaises(PolicyError): Bundle(manifest, owner=os.geteuid())

    def test_vmm_cgroup_must_belong_to_exact_account(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); proc = root / "proc"; proc.mkdir()
            hv = root / "cloud-hypervisor"; hv.touch()
            process = proc / "123"; process.mkdir()
            (process / "exe").symlink_to(hv)
            cid = "a" * 64
            (process / "cmdline").write_bytes((str(hv) + "\0--api-socket\0/run/vc/vm/" + cid + "/clh.sock\0").encode())
            path = "/fenixacct.slice/fenixacct-" + ALICE.replace("-", "") + ".slice/kata_" + cid
            (process / "cgroup").write_text("0::" + path + "\n")
            self.assertEqual(vm_cgroup(ALICE, cid, hv, proc), path)
            with self.assertRaises(PolicyError): vm_cgroup(BOB, cid, hv, proc)
            with self.assertRaises(PolicyError): vm_cgroup(ALICE, "b" * 64, hv, proc)


class PersistentEnvironmentTests(unittest.TestCase):
    def setUp(self): self.f = Fixture()
    def tearDown(self): self.f.close()

    def test_environment_survives_execution_and_is_project_scoped(self):
        root = self.f.users / ALICE
        first = environment(root, root / "workspace", os.geteuid(), os.getegid())
        (Path(first.source) / "installed-tool").write_text("persistent")
        first.close()
        resumed = environment(root, root / "workspace", os.geteuid(), os.getegid())
        try:
            self.assertEqual((Path(resumed.source) / "installed-tool").read_text(), "persistent")
            second = environment(root, root / "projects" / "another", os.geteuid(), os.getegid())
            try: self.assertNotEqual(second.path, resumed.path)
            finally: second.close()
        finally: resumed.close()

    def test_environment_rejects_symlink_parent_and_foreign_workspace(self):
        root = self.f.users / ALICE
        (root / "environments").symlink_to(self.f.users / BOB, target_is_directory=True)
        with self.assertRaises(OSError): environment(root, root / "workspace", os.geteuid(), os.getegid())
        with self.assertRaises(ValueError): environment(root, self.f.users / BOB / "workspace", os.geteuid(), os.getegid())

    def test_language_server_gets_read_only_home_and_read_commands_get_none(self):
        self.f.policy.config["persistent_environments"] = True
        self.f.policy.accounts[ALICE].update(uid=os.geteuid(), gid=os.getegid())
        request = self.f.spec(); request["kind"] = "language_server"
        launch = self.f.policy.launch(self.f.accounts[0], request)
        try:
            self.assertIsNotNone(launch.home)
            self.assertTrue(all(readonly for _, readonly in launch.mounts))
            self.assertEqual(launch.profile, "offline")
        finally: launch.close()
        file = self.f.users / ALICE / "workspace" / "input"; file.write_text("x")
        launch = self.f.policy.launch(self.f.accounts[0], self.f.spec(read_paths=[str(file)]))
        try: self.assertIsNone(launch.home)
        finally: launch.close()

    def test_environment_cannot_inject_loader_or_provider_credentials(self):
        env = variables("/workspace/tools")
        self.assertEqual(env["HOME"], "/workspace/tools")
        self.assertNotIn("LD_PRELOAD", env)
        self.assertNotIn("FENIX_UPSTREAM_TOKEN", env)
        self.assertTrue(env["NPM_CONFIG_PREFIX"].startswith(env["HOME"] + "/"))


class GateProtocolTests(unittest.IsolatedAsyncioTestCase):
    async def run_gate(self, grant):
        process = await asyncio.create_subprocess_exec(sys.executable, "-I", str(SCRIPTS / "microvm-gate.py"),
                   sys.executable, "-I", "-c", "print('workload-started')",
                   stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
        try:
            line = await asyncio.wait_for(process.stdout.readline(), 3)
            self.assertTrue(line.startswith(GATE_PREFIX))
            self.assertIsNone(process.returncode)
            out, err = await asyncio.wait_for(process.communicate(grant), 3)
            return process.returncode, out, err
        finally:
            if process.returncode is None:
                process.kill(); await process.wait()

    async def test_workload_waits_for_host_grant(self):
        code, out, err = await self.run_gate(b"FENIX-START\n")
        self.assertEqual(code, 0, err); self.assertEqual(out, b"workload-started\n")

    async def test_invalid_grant_and_disconnect_never_start_workload(self):
        for grant in (b"NO\n", b""):
            code, out, _ = await self.run_gate(grant)
            self.assertNotEqual(code, 0); self.assertNotIn(b"workload-started", out)


class ComputeBudgetTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.f = Fixture(); self.runtime = AsyncMock()
        self.service = Service(self.f.policy, self.runtime, self.f.records)

    async def asyncTearDown(self):
        await self.service.shutdown(); self.f.close()

    async def test_vm_overhead_is_counted_before_provisioning(self):
        self.f.policy.runtime_overhead = 256
        with self.assertRaisesRegex(PolicyError, "Account execution memory"):
            await self.service.create(self.f.accounts[0], self.f.spec())
        self.runtime.create.assert_not_awaited()

    async def test_revocation_during_boot_does_not_release_guest_gate(self):
        grant = await self.service.create(self.f.accounts[0], self.f.spec(timeout=40))
        async def attach(execution):
            self.f.accounts[0]["disabled"] = True
            self.f.save()
            return AsyncMock()
        self.runtime.attach.side_effect = attach
        with self.assertRaises(PolicyError):
            await self.service.attach(grant, asyncio.StreamReader(), AsyncMock())
        self.runtime.release.assert_not_awaited()

    async def test_monthly_reservations_prevent_concurrent_overspend_and_refund_unused_time(self):
        self.f.policy.accounts[ALICE]["execution_seconds_per_month"] = 60
        first = await self.service.create(self.f.accounts[0], self.f.spec(timeout=40))
        with self.assertRaisesRegex(PolicyError, "monthly execution"):
            await self.service.create(self.f.accounts[0], self.f.spec(timeout=40))
        self.assertEqual(self.f.records.usage(ALICE)["reserved_seconds"], 40)
        self.assertEqual(self.f.records.usage(BOB)["reserved_seconds"], 0)
        await self.service.destroy(self.service.owned(ALICE, first["id"]))
        self.assertEqual(self.f.records.usage(ALICE)["reserved_seconds"], 0)
        self.assertLess(self.f.records.usage(ALICE)["used_seconds"], 5)
        await self.service.create(self.f.accounts[0], self.f.spec(timeout=40))

    async def test_restart_cannot_reset_outstanding_budget(self):
        launch = self.f.policy.launch(self.f.accounts[0], self.f.spec(timeout=40))
        execution = Execution(launch)
        try:
            self.f.records.insert(execution, 60)
            self.f.records.db.close()
            for handler in self.f.records.log.handlers: handler.close()
            self.f.records = Records(self.f.root / "state")
            self.service.records = self.f.records
            self.assertEqual(self.f.records.usage(ALICE)["used_seconds"], 40)
            self.assertEqual(self.f.records.usage(ALICE)["reserved_seconds"], 0)
            with self.assertRaises(PolicyError): self.f.records.insert(Execution(launch), 60)
        finally: launch.close()
