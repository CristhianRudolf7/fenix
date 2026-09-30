"""Check transport discovery without running a model or changing task files."""
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock
from native_fenix import environment_workspace, trial_config, wait_for_report


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


    def test_binds_the_trial_endpoint_for_each_configured_role_only(self):
        template = {"settings": {"base_url": "http://127.0.0.1/{trial}/v1"},
                    "fast": {"model": "fast", "base_url": "http://127.0.0.1/{trial}/v1"}}
        config = trial_config(template, "/app", "native-t1")
        self.assertEqual(config["settings"]["base_url"], "http://127.0.0.1/native-t1/v1")
        self.assertEqual(config["fast"]["base_url"], "http://127.0.0.1/native-t1/v1")
        self.assertEqual(config["settings"]["workspace"], "/app")
        self.assertEqual(template["fast"]["base_url"], "http://127.0.0.1/{trial}/v1")
        self.assertNotIn("fast", trial_config({"settings": {}}, "/app", "native-t1"))


if __name__ == "__main__":
    unittest.main()
