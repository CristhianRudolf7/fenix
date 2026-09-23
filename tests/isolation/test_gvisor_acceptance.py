"""Opt-in real systrap acceptance, without KVM or production accounts.

FENIX_GVISOR_TESTS=1 FENIX_GVISOR_MANIFEST=/etc/fenix/gvisor/manifest.json
Missing runtime is a failure when explicitly requested, never a skip/fallback.
"""
import asyncio
import json
import os
from pathlib import Path
import shutil
import shlex
import subprocess
import unittest
import test_docker as docker_tests
import test_browser as browser_tests
import test_workspace as workspace_tests
from fenix_execution.gvisor import Bundle, GVisor, host_requirements

ENABLED = os.environ.get("FENIX_GVISOR_TESTS") == "1" and os.geteuid() == 0


class GVisorFixture:
    backend_class = GVisor

    async def asyncSetUp(self):
        host_requirements()
        self.manifest = os.environ["FENIX_GVISOR_MANIFEST"]
        bundle = Bundle(self.manifest)
        bundle.check_registration(json.loads(subprocess.check_output(["docker", "info", "--format", "{{json .}}"])))
        try:
            await docker_tests.DockerAcceptance.asyncSetUp(self)
        except BaseException:
            if hasattr(self, "users"):
                for path in self.users.iterdir():
                    if os.path.ismount(path): subprocess.run(["umount", str(path)], check=True)
                shutil.rmtree(self.root)
            raise

    def configure_fixture(self):
        self.config.update(backend="gvisor", runtime="fenix-runsc", gvisor_manifest=self.manifest,
                           persistent_environments=True, runtime_overhead_mib=128, max_runner_memory_mib=2048)
        self.config["runner_limits"].update(memory_mib=512, pids=256, cpus=1, deadline_seconds=120)
        self.config["account_limits"].update(memory_mib=2048, pids=1024, cpus=2)


class GVisorAcceptance(GVisorFixture, docker_tests.DockerAcceptance):
    __unittest_skip__ = not ENABLED
    __unittest_skip_why__ = "Requires root opt-in and a pinned gVisor systrap runtime"

    async def test_runner_files_environment_limits_and_owner_bound_handles(self):
        root = self.users / self.owners[0]
        command = f'test -f /proc/gvisor/kernel_is_gvisor; test ! -e /dev/kvm; test ! -e /var/run/docker.sock; test ! -e {root}/state/synthetic-secret; test ! -e {self.users/self.owners[1]}/workspace; test -z "${{FENIX_UPSTREAM_TOKEN-}}"; id -u; printf verified; sleep 10'
        grant, reader, writer = await self.start("set -eu; " + command, timeout=60)
        denied = await self.request(1, operation="status", id=grant["id"]); self.assertFalse(denied["ok"])
        # Guest cgroups are virtual; enforce and inspect the actual host Sentry
        # cgroup rather than accepting the guest's accounting as proof.
        for _ in range(200):
            metadata = self.backend.mount_dir / grant["id"] / "container.json"
            record = json.loads(metadata.read_text())
            if "sandbox_cgroup" in record: break
            await asyncio.sleep(.1)
        else: self.fail("Sandbox startup was not verified")
        control = Path("/sys/fs/cgroup") / record["sandbox_cgroup"].lstrip("/")
        self.assertEqual((control / "memory.max").read_text().strip(), str(self.policy.runner_limits.memory_mib * 1024 * 1024))
        self.assertEqual((control / "pids.max").read_text().strip(), str(self.policy.runner_limits.pids))
        quota, period = map(int, (control / "cpu.max").read_text().split())
        self.assertAlmostEqual(quota / period, self.policy.runner_limits.cpus)
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("61001", output); self.assertIn("verified", output)
        writer.close()

    async def test_native_file_changes_are_visible_to_running_sandbox(self):
        workspace = self.users / self.owners[0] / "workspace"
        path = workspace / "shared.txt"; path.write_text("before"); os.chown(path, 61001, 61001)
        grant, reader, writer = await self.start("set -eu; cat shared.txt; printf ready > ready; while test ! -e continue; do sleep .1; done; cat shared.txt; printf sandbox > returned", timeout=60)
        for _ in range(200):
            if (workspace / "ready").exists(): break
            await asyncio.sleep(.1)
        else: self.fail("Sandbox did not reach shared file check")
        path.write_text("after"); (workspace / "continue").touch()
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("beforeafter", output)
        self.assertEqual((workspace / "returned").read_text(), "sandbox")
        writer.close()

    async def test_thread_compatibility_preserves_clone3_and_namespace_denials(self):
        code = """import ctypes, errno, threading
libc = ctypes.CDLL(None, use_errno=True)
assert libc.syscall(ctypes.c_long(435), ctypes.c_void_p(), ctypes.c_size_t(0)) == -1
assert ctypes.get_errno() == errno.ENOSYS
assert libc.unshare(0x10000000) == -1
assert ctypes.get_errno() == errno.EPERM
result = []
thread = threading.Thread(target=lambda: result.append('thread-ok'))
thread.start(); thread.join()
assert result == ['thread-ok']
print('thread-ok; clone3 and namespace creation remain denied')
"""
        _, reader, writer = await self.start("python3 -c " + shlex.quote(code), timeout=60)
        output, status = await self.output(reader)
        self.assertEqual(status, 0, output)
        self.assertIn("thread-ok", output)
        writer.close()

    async def test_node_and_native_compiler_work_in_the_project(self):
        program = '#include <stdio.h>\nint main(void) { puts("compiler-ok"); return 0; }\n'
        # JSON quoting supplies the worker source without shell interpretation.
        worker = 'require("node:worker_threads").parentPort.postMessage("node-ok")'
        node = 'const {Worker}=require("node:worker_threads"); const w=new Worker(' + json.dumps(worker) + ',{eval:true}); w.once("message",console.log);'
        command = "set -eu; printf %s " + shlex.quote(program) + " > main.c; cc main.c -o main; ./main; node -e " + shlex.quote(node)
        _, reader, writer = await self.start(command, timeout=60)
        output, status = await self.output(reader)
        self.assertEqual(status, 0, output)
        self.assertIn("compiler-ok", output); self.assertIn("node-ok", output)
        writer.close()


class GVisorWorkspaceAcceptance(GVisorFixture, workspace_tests.WorkspaceAcceptance):
    __unittest_skip__ = not ENABLED
    __unittest_skip_why__ = GVisorAcceptance.__unittest_skip_why__


class GVisorBrowserAcceptance(GVisorFixture, browser_tests.BrowserAcceptance):
    __unittest_skip__ = not ENABLED
    __unittest_skip_why__ = GVisorAcceptance.__unittest_skip_why__
