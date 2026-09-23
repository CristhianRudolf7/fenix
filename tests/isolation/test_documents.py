"""Document requests cannot replace the worker, grant network, or widen mounts."""
import json
from pathlib import Path
import unittest
from test_execution import Fixture, ALICE, BOB
from fenix_isolation import PolicyError


class DocumentPolicyTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.path = self.f.users/ALICE/"workspace"/"invoice.pdf"
        self.path.write_bytes(b"%PDF-fixture")
    def tearDown(self): self.f.close()
    def spec(self):
        spec = self.f.spec()
        req = dict(path=str(self.path), sha256="a"*64, mode="auto", start_page=1, page_count=5, dpi=200)
        spec.update(kind="document", argv=["/opt/fenix-ocr/bin/python", "-I", "-u", "/usr/local/lib/fenix-ocr/worker.py", json.dumps(req)],
                    read_paths=[str(self.path)], profile="development", timeout=1000)
        return spec
    def test_only_input_mounted_readonly_and_network_always_offline(self):
        launch = self.f.policy.launch(self.f.accounts[0], self.spec())
        try:
            self.assertEqual(launch.profile, "offline"); self.assertTrue(launch.readonly)
            self.assertEqual(len(launch.mounts), 1); self.assertTrue(launch.mounts[0][1])
            self.assertEqual(launch.mounts[0][0].path, self.path)
            self.assertLessEqual(launch.timeout, 300); self.assertIsNone(launch.home)
        finally: launch.close()
    def test_worker_override_extra_mounts_environment_and_tty_are_rejected(self):
        for mutate in [lambda s: s["argv"].__setitem__(0, "/bin/bash"),
                       lambda s: s.update(tty=True), lambda s: s.update(environment={"LANG": "en"}),
                       lambda s: s["read_paths"].append(str(self.path.parent)),
                       lambda s: s.update(read_paths=[]),
                       lambda s: s["argv"].__setitem__(4, '{}')]:
            spec = self.spec(); mutate(spec)
            with self.assertRaises(PolicyError): self.f.policy.launch(self.f.accounts[0], spec)
    def test_foreign_file_directory_and_symlink_are_rejected(self):
        for path in [self.f.users/BOB/"workspace"/"other.pdf", self.path.parent, self.path.with_name("link.pdf")]:
            if path.name == "other.pdf": path.write_bytes(b"private")
            if path.name == "link.pdf": path.symlink_to(self.path)
            spec = self.spec(); request = json.loads(spec["argv"][-1]); request["path"] = str(path)
            spec["argv"][-1] = json.dumps(request); spec["read_paths"] = [str(path)]
            with self.assertRaises((PolicyError, ValueError)): self.f.policy.launch(self.f.accounts[0], spec)


if __name__ == "__main__": unittest.main()
