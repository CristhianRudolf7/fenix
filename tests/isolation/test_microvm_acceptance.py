"""Real VM acceptance; opt in on a disposable KVM host with a pinned bundle.

FENIX_MICROVM_TESTS=1 FENIX_MICROVM_MANIFEST=/etc/fenix/microvm/manifest.json
FENIX_TEST_IMAGE=fenix-workspace:local python3 -m unittest discover ...
An explicitly requested VM test FAILS if KVM/the runtime is unavailable.
"""
import asyncio
import json
import os
from pathlib import Path
import subprocess
import shutil
import unittest
import test_docker as docker_tests
import test_browser as browser_tests
from fenix_execution.microvm import Bundle, MicroVM, require_kvm

ENABLED = os.environ.get("FENIX_MICROVM_TESTS") == "1" and os.geteuid() == 0


class MicroVMAcceptance(docker_tests.DockerAcceptance):
    __unittest_skip__ = not ENABLED
    __unittest_skip_why__ = "Requires explicit root opt-in, KVM and a pinned Kata/Cloud Hypervisor bundle"
    backend_class = MicroVM

    async def asyncSetUp(self):
        require_kvm()
        self.manifest = os.environ["FENIX_MICROVM_MANIFEST"]
        Bundle(self.manifest)
        try:
            await super().asyncSetUp()
        except BaseException:
            # Missing runtime/insufficient RAM must not leave test volumes behind.
            if hasattr(self, "users"):
                for path in self.users.iterdir():
                    if os.path.ismount(path):
                        subprocess.run(["umount", str(path)], check=True)
                shutil.rmtree(self.root)
            raise

    def configure_fixture(self):
        self.config.update(backend="microvm", runtime="fenix-kata", microvm_manifest=self.manifest,
                           microvm_overhead_mib=256, persistent_environments=True,
                           max_runner_memory_mib=4096)
        self.config["runner_limits"].update(memory_mib=1024, pids=512, cpus=1, deadline_seconds=120)
        self.config["account_limits"].update(memory_mib=4096, pids=2048, cpus=2)

    async def test_fresh_guest_kernel_and_persistent_tools_after_shutdown(self):
        first, reader, writer = await self.start("mkdir -p \"$HOME/.local/bin\"; printf persistent > \"$HOME/.local/bin/test-tool\"; printf saved; sleep 1", timeout=90)
        boot = self.backend.boots[first["id"]]
        self.assertNotEqual(boot, Path("/proc/sys/kernel/random/boot_id").read_text().strip())
        output, code = await self.output(reader); self.assertEqual(code, 0, output)
        await self.request(operation="cancel", id=first["id"])
        writer.close()
        second, reader, writer = await self.start("cat \"$HOME/.local/bin/test-tool\"; sleep 1", timeout=90)
        self.assertNotEqual(self.backend.boots[second["id"]], boot)
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("persistent", output)
        writer.close()

    async def test_documents_can_be_created_and_seen_by_native_file_tools(self):
        code = "from docx import Document; d=Document(); d.add_paragraph('Fenix workspace'); d.save('result.docx')"
        import shlex
        grant, reader, writer = await self.start("python3 -c " + shlex.quote(code), timeout=90)
        output, status = await self.output(reader)
        self.assertEqual(status, 0, output)
        self.assertTrue((self.users / self.owners[0] / "workspace/result.docx").is_file())
        writer.close()


class MicroVMBrowserAcceptance(unittest.IsolatedAsyncioTestCase):
    __unittest_skip__ = not ENABLED
    __unittest_skip_why__ = MicroVMAcceptance.__unittest_skip_why__
    backend_class = MicroVM
    configure_fixture = MicroVMAcceptance.configure_fixture
    asyncTearDown = docker_tests.DockerAcceptance.asyncTearDown
    save_registry = docker_tests.DockerAcceptance.save_registry
    request = docker_tests.DockerAcceptance.request

    async def asyncSetUp(self):
        require_kvm()
        self.manifest = os.environ["FENIX_MICROVM_MANIFEST"]
        Bundle(self.manifest)
        await docker_tests.DockerAcceptance.asyncSetUp(self)

    test_browser_keeps_inner_sandbox_and_cdp_is_owner_scoped = browser_tests.BrowserAcceptance.test_browser_keeps_inner_sandbox_and_cdp_is_owner_scoped
