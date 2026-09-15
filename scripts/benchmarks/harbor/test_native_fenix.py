"""Check transport discovery without running a model or changing task files."""
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock
from native_fenix import environment_workspace, wait_for_report


class EnvironmentWorkspaceTests(unittest.IsolatedAsyncioTestCase):
    async def test_accepts_report_written_between_ready_and_exit_checks(self):
        environment = SimpleNamespace(exec=AsyncMock(side_effect=[
            SimpleNamespace(return_code=1),
            SimpleNamespace(return_code=1),
            SimpleNamespace(return_code=0),
        ]))
        await wait_for_report(environment)
        self.assertEqual(environment.exec.await_count, 3)

    async def test_rejects_exit_without_a_final_report(self):
        environment = SimpleNamespace(exec=AsyncMock(return_value=SimpleNamespace(return_code=1)))
        with self.assertRaisesRegex(RuntimeError, "exited before reporting"):
            await wait_for_report(environment)

    async def test_uses_the_container_workdir_instead_of_assuming_app(self):
        environment = SimpleNamespace(exec=AsyncMock(return_value=SimpleNamespace(return_code=0, stdout="/workspace\n")))
        self.assertEqual(await environment_workspace(environment), "/workspace")
        environment.exec.assert_awaited_once_with("pwd")

    async def test_does_not_invent_a_directory_after_discovery_failure(self):
        for code, output in [(1, "/workspace"), (0, "relative"), (0, "/a\n/b"), (0, "")]:
            environment = SimpleNamespace(exec=AsyncMock(return_value=SimpleNamespace(return_code=code, stdout=output)))
            with self.assertRaises(RuntimeError):
                await environment_workspace(environment)


if __name__ == "__main__":
    unittest.main()
