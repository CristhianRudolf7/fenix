"""Check transport discovery without running a model or changing task files."""
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock
from native_fenix import environment_workspace


class EnvironmentWorkspaceTests(unittest.IsolatedAsyncioTestCase):
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
