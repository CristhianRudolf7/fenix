"""Behavioral checks for original research, CAD, robot and memory helpers."""

import copy
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SKILLS = ROOT / "resources/skills"


def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, SKILLS / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


evidence = load("evidence", "research-evidence-audit/scripts/audit_evidence.py")
screening = load("screening", "systematic-review/scripts/screening_counts.py")
memory = load("memory", "model-selection/scripts/estimate_memory.py")
robot = load("robot", "robot-description/scripts/check_urdf.py")
geometry = load("geometry", "parametric-cad/scripts/inspect_geometry.py")
drawing = load("drawing", "cad-drawing-and-fabrication/scripts/inspect_drawing.py")
plate = load("plate", "parametric-cad/assets/mounting_plate.py")


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        manuscript = b"Observed accuracy was 0.75.\n"
        source = b"Sample size: 20\nCorrect predictions: 15\n"
        (self.root / "paper.txt").write_bytes(manuscript)
        (self.root / "result.txt").write_bytes(source)
        self.ledger = {
            "manuscript": {"path": "paper.txt", "sha256": hashlib.sha256(manuscript).hexdigest()},
            "sources": [{"id": "run-1", "path": "result.txt", "sha256": hashlib.sha256(source).hexdigest()}],
            "claims": [{"id": "claim-1", "text": "Observed accuracy was 0.75.", "line_start": 1, "line_end": 1,
                        "status": "supported", "reason": "15 divided by 20", "evidence": [
                            {"source_id": "run-1", "line_start": 1, "line_end": 2, "quote": "Correct predictions: 15"}]}],
        }

    def test_bound_evidence_is_structural_not_semantic_proof(self):
        result = evidence.audit(self.ledger, self.root)
        self.assertEqual(result["evidence_locators_checked"], 1)
        self.assertFalse(result["semantic_support_verified_by_tool"])

    def test_stale_source_and_manuscript_are_rejected(self):
        for name in ("paper.txt", "result.txt"):
            with self.subTest(name=name):
                path = self.root / name
                original = path.read_bytes()
                path.write_bytes(original + b"Changed.\n")
                with self.assertRaisesRegex(ValueError, "Stale source"):
                    evidence.audit(self.ledger, self.root)
                path.write_bytes(original)

    def test_wrong_quote_or_location_is_rejected(self):
        for patch in ({"quote": "Correct predictions: 20"}, {"line_start": 1, "line_end": 1}, {"line_end": 50}):
            with self.subTest(patch=patch):
                document = copy.deepcopy(self.ledger)
                document["claims"][0]["evidence"][0].update(patch)
                with self.assertRaises(ValueError):
                    evidence.audit(document, self.root)

    def test_duplicate_identity_and_dangling_source_are_rejected(self):
        document = copy.deepcopy(self.ledger)
        document["sources"] *= 2
        with self.assertRaises(ValueError):
            evidence.audit(document, self.root)
        self.ledger["claims"][0]["evidence"][0]["source_id"] = "missing"
        with self.assertRaises(ValueError):
            evidence.audit(self.ledger, self.root)

    def test_unresolved_is_preserved_but_resolved_needs_evidence(self):
        self.ledger["claims"][0]["evidence"] = []
        with self.assertRaises(ValueError):
            evidence.audit(self.ledger, self.root)
        self.ledger["claims"][0]["status"] = "unresolved"
        result = evidence.audit(self.ledger, self.root)
        self.assertEqual(result["supplied_judgments"], {"unresolved": 1})

    def test_symlink_escape_is_rejected(self):
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / "evidence.txt"
            target.write_text("external")
            (self.root / "link.txt").symlink_to(target)
            self.ledger["sources"][0]["path"] = "link.txt"
            with self.assertRaisesRegex(ValueError, "escapes"):
                evidence.audit(self.ledger, self.root)

    def test_cli_success_and_failure_exit_status(self):
        path = self.root / "ledger.json"
        path.write_text(json.dumps(self.ledger))
        command = [sys.executable, str(SKILLS / "research-evidence-audit/scripts/audit_evidence.py"), str(path), "--root", str(self.root)]
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["structural_status"], "valid")
        (self.root / "result.txt").write_text("stale")
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)


class ScreeningTests(unittest.TestCase):
    def ledger(self):
        states = ["included", "included", "duplicate", "screen_excluded", "full_text_excluded", "not_retrieved", "pending", "full_text_pending"]
        records = [{"id": f"r{i}", "search_id": "s1", "state": state, "reason": "Recorded decision"} for i, state in enumerate(states)]
        records[0]["study_id"] = records[1]["study_id"] = "study-A"
        records[2]["duplicate_of"] = "r0"
        return {"searches": [{"id": "s1", "database": "fixture", "query": "fixed search", "searched_on": "2026-09-19"}], "records": records}

    def test_reports_studies_and_pending_are_separate(self):
        result = screening.count_screening(self.ledger())
        expected = {"identified_records": 8, "duplicates_removed": 1, "screening_pending": 1,
                    "records_screened": 6, "records_excluded": 1, "reports_sought": 5,
                    "full_text_pending": 1, "reports_not_retrieved": 1, "reports_assessed": 3,
                    "reports_excluded": 1, "reports_included": 2, "studies_included": 1}
        for key, value in expected.items():
            self.assertEqual(result[key], value, key)
        self.assertFalse(result["screening_complete"])

    def test_invalid_decisions_and_references_are_rejected(self):
        patches = [(0, {"state": "accepted"}), (0, {"search_id": "missing"}), (0, {"study_id": ""}),
                   (2, {"duplicate_of": "r2"}), (3, {"reason": ""}), (0, {"duplicate_of": "r1"})]
        for index, patch in patches:
            with self.subTest(patch=patch):
                document = self.ledger()
                document["records"][index].update(patch)
                with self.assertRaises(ValueError):
                    screening.count_screening(document)

    def test_unretrieved_reports_are_not_exclusions(self):
        result = screening.count_screening(self.ledger())
        self.assertEqual(result["reports_not_retrieved"], 1)
        self.assertEqual(result["reports_excluded"], 1)
        self.assertEqual(result["reports_assessed"], result["reports_excluded"] + result["reports_included"])


class MemoryTests(unittest.TestCase):
    def test_gqa_cache_and_binary_units(self):
        kv = {"layers": 32, "kv_heads": 8, "head_dim": 128, "context": 8192, "batch": 2, "kv_bits": 16}
        result = memory.estimate(8_000_000_000, 4, kv, available_gib=16)
        self.assertEqual(result["estimated_weight_bytes"], 4_000_000_000)
        self.assertEqual(result["estimated_kv_bytes"], 2 * 1024 ** 3)
        self.assertFalse(result["fit_verified"])
        self.assertEqual(result["capacity_assessment"], "requires_runtime_measurement")

    def test_missing_dimensions_do_not_imply_fit(self):
        result = memory.estimate(32_000_000_000, 4, available_gib=16)
        self.assertIsNone(result["estimated_kv_bytes"])
        self.assertIsNone(result["assumed_runtime_overhead_bytes"])
        self.assertFalse(result["fit_verified"])

    def test_insufficient_capacity_and_invalid_input(self):
        self.assertEqual(memory.estimate(8_000_000_000, 16, available_gib=8)["capacity_assessment"], "insufficient_under_estimate")
        for parameters, bits in [(0, 4), (True, 4), (8, float("nan")), (8, -1)]:
            with self.assertRaises(ValueError):
                memory.estimate(parameters, bits)
        with self.assertRaises(ValueError):
            memory.estimate(8, 4, {"layers": 32})


URDF = b'''<robot name="fixture"><link name="base"/><link name="arm"><inertial>
<mass value="1"/><inertia ixx="0.1" iyy="0.1" izz="0.1" ixy="0" ixz="0" iyz="0"/>
</inertial></link><joint name="hinge" type="revolute"><parent link="base"/><child link="arm"/>
<axis xyz="0 0 1"/><limit lower="-1" upper="1" effort="2" velocity="1"/></joint></robot>'''


class RobotTests(unittest.TestCase):
    def test_valid_tree_retains_unchecked_physics(self):
        result = robot.check_urdf(URDF)
        self.assertEqual((result["root_link"], result["links"], result["joints"]), ("base", 2, 1))
        self.assertEqual(result["links_without_inertials"], ["base"])
        self.assertTrue(result["unchecked"])

    def test_invalid_topology_axes_limits_and_inertia(self):
        replacements = [(b'parent link="base"', b'parent link="missing"'), (b'xyz="0 0 1"', b'xyz="0 0 0"'),
                        (b'lower="-1"', b'lower="2"'), (b'ixx="0.1"', b'ixx="-0.1"'),
                        (b'value="1"', b'value="nan"'), (b'="0.1"', b'="1e308"'),
                        (b'<link name="base"/>', b'<link name="base"/><link name="orphan"/>')]
        for old, new in replacements:
            with self.subTest(new=new), self.assertRaises(ValueError):
                robot.check_urdf(URDF.replace(old, new))

    def test_entities_and_mimic_cycles_are_rejected(self):
        with self.assertRaises(ValueError):
            robot.check_urdf(b'<!DOCTYPE robot [<!ENTITY x "x">]>' + URDF)
        cycle = b'''<robot name="r"><link name="a"/><link name="b"/><link name="c"/>
<joint name="j1" type="fixed"><parent link="a"/><child link="b"/><mimic joint="j2"/></joint>
<joint name="j2" type="fixed"><parent link="b"/><child link="c"/><mimic joint="j1"/></joint></robot>'''
        with self.assertRaisesRegex(ValueError, "cycle"):
            robot.check_urdf(cycle)


class GeometryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import build123d as cad

        cls.temp = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        shape = plate.make_plate()
        cad.export_step(shape, cls.root / "plate.step")
        cad.export_stl(shape, cls.root / "plate.stl")

    def test_step_roundtrip_against_independent_dimensions_and_volume(self):
        result = geometry.inspect_geometry(self.root / "plate.step")
        self.assertEqual(geometry.check_expectations(result, [40, 30, 4], 1), [])
        self.assertAlmostEqual(result["volume_mm3"], 4800 - 2 * math.pi * 2.5 ** 2 * 4, places=5)
        self.assertTrue(geometry.check_expectations(result, [41, 30, 4], 1))

    def test_mesh_roundtrip_and_explicit_units(self):
        result = geometry.inspect_geometry(self.root / "plate.stl", "mm")
        self.assertTrue(result["valid_geometry"])
        self.assertAlmostEqual(result["volume_mm3"], 4800 - 2 * math.pi * 2.5 ** 2 * 4, delta=0.5)
        with self.assertRaisesRegex(ValueError, "units"):
            geometry.inspect_geometry(self.root / "plate.stl")
        scaled = geometry.inspect_geometry(self.root / "plate.stl", "cm")
        self.assertAlmostEqual(scaled["size_mm"][0], 400)

    def test_open_mesh_is_not_a_valid_solid(self):
        import trimesh

        path = self.root / "open.stl"
        trimesh.Trimesh(vertices=[[0, 0, 0], [1, 0, 0], [0, 1, 0]], faces=[[0, 1, 2]], process=False).export(path)
        result = geometry.inspect_geometry(path, "mm")
        self.assertFalse(result["valid_geometry"])
        self.assertIsNone(result["volume_mm3"])

    def test_invalid_plate_dimensions_are_rejected(self):
        with self.assertRaises(ValueError):
            plate.make_plate(hole_pitch=50)


class DrawingTests(unittest.TestCase):
    def test_corrupt_drawing_returns_structured_cli_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "broken.dxf"
            path.write_text("This is not a DXF file")
            result = subprocess.run([sys.executable, str(SKILLS / "cad-drawing-and-fabrication/scripts/inspect_drawing.py"), str(path)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("error", json.loads(result.stdout))

    def test_saved_units_bounds_and_open_profile(self):
        import ezdxf

        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "part.dxf"
            doc = ezdxf.new("R2013")
            doc.units = ezdxf.units.MM
            doc.modelspace().add_lwpolyline([(0, 0), (10, 0), (10, 20), (0, 20)], close=True)
            doc.saveas(path)
            result = drawing.inspect_drawing(path)
            self.assertEqual(result["bounds_mm"], [[0, 0, 0], [10, 20, 0]])
            self.assertFalse(result["manufacturing_ready_verified"])
            doc.units = 0
            doc.modelspace().add_lwpolyline([(1, 1), (2, 2)], close=False)
            doc.saveas(path)
            result = drawing.inspect_drawing(path)
            self.assertFalse(result["units_known"])
            self.assertIsNone(result["bounds_mm"])
            self.assertEqual(result["open_lwpolylines"], 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
