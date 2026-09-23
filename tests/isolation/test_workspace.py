"""Real persistent tool-home behavior on the existing OCI backend.

This verifies the shared filesystem/policy path without claiming VM isolation.
"""
import os
import unittest
import test_docker as docker_tests


@unittest.skipUnless(docker_tests.ENABLED, "Requires root opt-in and a local tenant image")
class WorkspaceAcceptance(unittest.IsolatedAsyncioTestCase):
    asyncSetUp = docker_tests.DockerAcceptance.asyncSetUp
    asyncTearDown = docker_tests.DockerAcceptance.asyncTearDown
    save_registry = docker_tests.DockerAcceptance.save_registry
    request = docker_tests.DockerAcceptance.request
    start = docker_tests.DockerAcceptance.start
    output = docker_tests.DockerAcceptance.output

    def configure_fixture(self):
        self.config["persistent_environments"] = True

    async def test_virtualenv_tools_and_files_survive_runner_replacement(self):
        first, reader, writer = await self.start('set -eu; python3 -m venv --without-pip "$HOME/venv"; mkdir -p "$HOME/.local/bin"; printf "#!/bin/sh\\nprintf persistent-tool" > "$HOME/.local/bin/fenix-fixture"; chmod +x "$HOME/.local/bin/fenix-fixture"; printf first > project.txt')
        output, code = await self.output(reader); self.assertEqual(code, 0, output)
        writer.close(); await self.request(operation="cancel", id=first["id"])
        self.assertEqual((self.users / self.owners[0] / "workspace/project.txt").read_text(), "first")
        second, reader, writer = await self.start('set -eu; "$HOME/venv/bin/python" -c "import sys; assert sys.prefix != sys.base_prefix"; fenix-fixture; cat project.txt')
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("persistent-toolfirst", output)
        writer.close()

    async def test_language_server_cannot_modify_saved_environment(self):
        first, reader, writer = await self.start('printf existing > "$HOME/marker"')
        output, code = await self.output(reader); self.assertEqual(code, 0, output)
        writer.close(); await self.request(operation="cancel", id=first["id"])
        command = 'set -eu; cat "$HOME/marker"; if printf changed > "$HOME/marker" 2>/dev/null; then exit 1; fi; printf readonly'
        _, reader, writer = await self.start(command, kind="language_server")
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("existing", output); self.assertIn("readonly", output)
        writer.close()
