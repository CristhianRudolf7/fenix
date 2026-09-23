import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[2] / "resources/skills/backup-and-restore/scripts/compare_manifests.py"
SPEC = importlib.util.spec_from_file_location("backup_manifests", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def manifest():
    return {"schema_version": 1, "snapshot_id": "backup-001", "algorithm": "sha256",
            "complete": True, "errors": [], "files": {"data/a.txt": "a" * 64}}


class ManifestTests(unittest.TestCase):
    def test_complete_same_snapshot_matches_without_mutating_inputs(self):
        first, second = manifest(), manifest()
        second["files"]["data/a.txt"] = "A" * 64
        original = copy.deepcopy((first, second))
        self.assertEqual(MODULE.compare(first, second)["verdict"], "match")
        self.assertEqual((first, second), original)

    def test_missing_added_and_modified_are_all_preserved(self):
        first, second = manifest(), manifest()
        first["files"]["missing"] = "b" * 64
        second["files"].update({"data/a.txt": "c" * 64, "extra": "d" * 64})
        result = MODULE.compare(first, second)
        self.assertEqual(result["verdict"], "mismatch")
        self.assertEqual(result["missing"], ["missing"])
        self.assertEqual(result["added"], ["extra"])
        self.assertEqual(result["changed"], ["data/a.txt"])

    def test_identical_collection_errors_cannot_pass(self):
        data = manifest()
        data["errors"] = ["unreadable confidential path"]
        self.assertEqual(MODULE.compare(data, data)["verdict"], "inconclusive")
        data["files"]["data/a.txt"] = "ERROR:permission denied"
        with self.assertRaises(ValueError):
            MODULE.compare(data, data)

    def test_incomplete_and_empty_evidence_do_not_pass(self):
        for modification in ({"complete": False}, {"files": {}}):
            with self.subTest(modification=modification):
                data = manifest() | modification
                self.assertEqual(MODULE.compare(data, data)["verdict"], "inconclusive")

    def test_mismatch_keeps_collection_gaps(self):
        first, second = manifest(), manifest()
        second.update(complete=False, files={})
        result = MODULE.compare(first, second)
        self.assertEqual(result["verdict"], "mismatch")
        self.assertTrue(result["evidence_gaps"])

    def test_snapshot_mismatch_is_invalid(self):
        first, second = manifest(), manifest()
        second["snapshot_id"] = "different-backup"
        with self.assertRaises(ValueError):
            MODULE.compare(first, second)

    def test_ambiguous_paths_and_file_directory_collisions_are_invalid(self):
        for path in ("/absolute", "../outside", "a/../b", "a//b", "a/./b", "C:/x", "a\\b", "a\x00b"):
            with self.subTest(path=path):
                data = manifest() | {"files": {path: "a" * 64}}
                with self.assertRaises(ValueError):
                    MODULE.compare(data, data)
        data = manifest()
        data["files"]["data"] = "b" * 64
        with self.assertRaises(ValueError):
            MODULE.compare(data, data)

    def test_invalid_schema_and_digests(self):
        for change in ({"complete": "true"}, {"schema_version": True}, {"algorithm": "md5"},
                       {"errors": "ignored"}, {"files": []}, {"unexpected": True},
                       {"files": {"a": "g" * 64}}):
            with self.subTest(change=change):
                data = manifest() | change
                with self.assertRaises(ValueError):
                    MODULE.compare(data, data)

    def test_cli_exit_codes_and_read_only_behavior(self):
        with tempfile.TemporaryDirectory() as directory:
            a, b = Path(directory) / "a.json", Path(directory) / "b.json"
            for change, expected in (({}, 0), ({"files": {}}, 1), ({"complete": False}, 2),
                                     ({"algorithm": "md5"}, 2)):
                a.write_text(json.dumps(manifest()))
                b.write_text(json.dumps(manifest() | change))
                before = a.read_bytes(), b.read_bytes()
                result = subprocess.run([sys.executable, str(SCRIPT), str(a), str(b)], capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, result.stderr)
                self.assertIn("verdict", json.loads(result.stdout))
                self.assertEqual((a.read_bytes(), b.read_bytes()), before)
                self.assertEqual(len(list(Path(directory).iterdir())), 2)

    def test_duplicate_keys_malformed_and_oversized_inputs_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input.json"
            for raw in ('{"files":{"x":1,"x":2}}', '{"complete":true,"complete":false}', '{bad', ' ' * (MODULE.MAX_BYTES + 1)):
                path.write_text(raw)
                with self.assertRaises(ValueError):
                    MODULE.read_manifest(path)

    def test_cli_error_does_not_echo_input_content(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input.json"
            path.write_text('{"secret":"PRIVATE-CANARY", bad}')
            result = subprocess.run([sys.executable, str(SCRIPT), str(path), str(path)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertNotIn("PRIVATE-CANARY", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
