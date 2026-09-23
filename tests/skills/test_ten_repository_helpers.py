"""Behavioral invariants for the original evaluation and timeline utilities."""

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]


def load(name, relative):
    path = ROOT / "resources/skills" / relative
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module, path


OUTCOMES, OUTCOME_PATH = load("paired", "experiment-and-benchmark-design/scripts/compare_outcomes.py")
TIMELINE, TIMELINE_PATH = load("timeline", "video-editing/scripts/check_timeline.py")


def evaluation():
    return {
        "schema_version": 1,
        "conditions": {name: {"revision": name, "execution_contract": "fixed-v1"} for name in ("baseline", "candidate")},
        "tasks": [{"id": "a", "split": "selection", "critical": True}, {"id": "b", "split": "selection", "critical": False}],
        "run_ids": ["r1", "r2"],
        "gate": {"min_delta": 0, "max_task_regression": 0},
        "outcomes": [{"condition": condition, "task_id": task, "run_id": run, "status": "completed", "score": int(condition == "candidate"), "evidence": "fixture/check.json"} for condition in ("baseline", "candidate") for task in ("a", "b") for run in ("r1", "r2")],
    }


def timeline():
    return {"fps": "30000/1001", "duration_frames": 120, "tolerance_frames": 2, "scenes": [{"id": "a", "start_frame": 0, "end_frame": 60}, {"id": "b", "start_frame": 60, "end_frame": 120}], "cues": [{"id": "show-button", "scene_id": "b", "visual_frame": 70, "audio_frame": 71}]}


class OutcomeTests(unittest.TestCase):
    def test_paired_improvement_and_unknown_usage(self):
        result = OUTCOMES.compare(evaluation())
        self.assertEqual(result["by_split"]["selection"]["delta"], 1)
        self.assertTrue(result["gate"]["meets_declared_gate"])
        self.assertEqual(result["usage"]["candidate"]["cost"], {"known_trials": 0, "total": None, "missing_trials": 4})
        self.assertFalse(result["statistical_superiority_established"])

    def test_identity_comparison_has_zero_delta_and_no_gain(self):
        data = evaluation()
        for row in data["outcomes"]:
            row["score"] = 0.5
        result = OUTCOMES.compare(data)
        self.assertEqual(result["by_split"]["selection"]["delta"], 0)
        self.assertEqual(result["by_split"]["selection"]["improved"], [])
        self.assertFalse(result["gate"]["meets_declared_gate"])

    def test_missing_baseline_cannot_manufacture_passing_gate(self):
        data = evaluation()
        data["outcomes"] = [row for row in data["outcomes"] if row["condition"] != "baseline"]
        result = OUTCOMES.compare(data)
        self.assertEqual(len(result["missing"]), 4)
        self.assertEqual(result["by_split"]["selection"]["planned_trials_per_condition"], 4)
        self.assertFalse(result["gate"]["meets_declared_gate"])

    def test_timeouts_and_errors_remain_in_denominator(self):
        data = evaluation()
        data["outcomes"][-1].update(status="timeout", score=0)
        data["outcomes"][-2].update(status="error", score=0)
        result = OUTCOMES.compare(data)
        self.assertEqual(result["by_split"]["selection"]["candidate"], 0.5)
        self.assertEqual(result["statuses"]["candidate"]["timeout"], 1)
        self.assertEqual(result["statuses"]["candidate"]["error"], 1)

    def test_aggregate_gain_cannot_hide_critical_regression(self):
        data = evaluation()
        for row in data["outcomes"]:
            if row["task_id"] == "a":
                row["score"] = 0.5 if row["condition"] == "baseline" else 0.25
        result = OUTCOMES.compare(data)
        self.assertGreater(result["by_split"]["selection"]["delta"], 0)
        self.assertFalse(result["gate"]["meets_declared_gate"])

    def test_critical_repetition_regression_is_not_averaged_away(self):
        data = evaluation()
        for row in data["outcomes"]:
            if row["task_id"] == "a":
                row["score"] = int((row["condition"], row["run_id"]) in {("baseline", "r1"), ("candidate", "r2")})
        self.assertFalse(OUTCOMES.compare(data)["gate"]["meets_declared_gate"])

    def test_test_split_cannot_select_candidate(self):
        data = evaluation()
        data["tasks"][1]["split"] = "test"
        for row in data["outcomes"]:
            if row["task_id"] == "a":
                row["score"] = 0
        result = OUTCOMES.compare(data)
        self.assertEqual(result["by_split"]["test"]["delta"], 1)
        self.assertFalse(result["gate"]["meets_declared_gate"])

    def test_no_selection_set_does_not_pass(self):
        data = evaluation()
        for task in data["tasks"]:
            task["split"] = "development"
        self.assertFalse(OUTCOMES.compare(data)["gate"]["meets_declared_gate"])

    def test_foreign_or_duplicate_identity_is_rejected(self):
        for field in ("condition", "task_id", "run_id"):
            data = evaluation()
            data["outcomes"][0][field] = "unknown"
            with self.assertRaises(ValueError):
                OUTCOMES.compare(data)
        data = evaluation()
        data["outcomes"].append(copy.deepcopy(data["outcomes"][0]))
        with self.assertRaises(ValueError):
            OUTCOMES.compare(data)

    def test_duplicate_tasks_runs_and_invalid_contracts_are_rejected(self):
        for field in ("tasks", "run_ids"):
            data = evaluation()
            data[field].append(copy.deepcopy(data[field][0]))
            with self.assertRaises(ValueError):
                OUTCOMES.compare(data)
        data = evaluation()
        data["conditions"]["candidate"]["execution_contract"] = "different-tools"
        with self.assertRaises(ValueError):
            OUTCOMES.compare(data)

    def test_nonfinite_boolean_and_out_of_range_scores_are_rejected(self):
        for value in (True, "1", -0.1, 1.1, float("nan"), float("inf")):
            data = evaluation()
            data["outcomes"][0]["score"] = value
            with self.assertRaises(ValueError):
                OUTCOMES.compare(data)

    def test_incomplete_status_cannot_claim_score(self):
        data = evaluation()
        data["outcomes"][-1]["status"] = "timeout"
        with self.assertRaises(ValueError):
            OUTCOMES.compare(data)

    def test_usage_coverage_and_integer_tokens(self):
        data = evaluation()
        data["outcomes"][0].update(tokens=100, seconds=2.5, cost=0.02)
        result = OUTCOMES.compare(data)
        self.assertEqual(result["usage"]["baseline"]["tokens"], {"known_trials": 1, "total": 100, "missing_trials": 3})
        data["outcomes"][0]["tokens"] = 1.5
        with self.assertRaises(ValueError):
            OUTCOMES.compare(data)

    def test_noncritical_tolerance_is_explicit(self):
        data = evaluation()
        for row in data["outcomes"]:
            if row["task_id"] == "b":
                row["score"] = 0.4 if row["condition"] == "baseline" else 0.2
        self.assertFalse(OUTCOMES.compare(data)["gate"]["meets_declared_gate"])
        data["gate"]["max_task_regression"] = 0.25
        self.assertTrue(OUTCOMES.compare(data)["gate"]["meets_declared_gate"])


class TimelineTests(unittest.TestCase):
    def test_rational_fps_preserves_exact_time(self):
        result = TIMELINE.check(timeline())
        self.assertTrue(result["valid"])
        self.assertEqual(result["duration_seconds_exact"], "1001/250")
        self.assertFalse(result["media_inspected"])

    def test_half_open_bounds(self):
        data = timeline()
        data["cues"][0].update(visual_frame=120, audio_frame=120)
        self.assertFalse(TIMELINE.check(data)["valid"])
        data["cues"][0].update(visual_frame=60, audio_frame=60)
        self.assertTrue(TIMELINE.check(data)["valid"])

    def test_gaps_overlaps_and_zero_length(self):
        for start in (59, 61, 120):
            data = timeline()
            data["scenes"][1]["start_frame"] = start
            self.assertFalse(TIMELINE.check(data)["valid"])

    def test_track_end_matches_duration(self):
        data = timeline()
        data["duration_frames"] = 130
        self.assertFalse(TIMELINE.check(data)["valid"])

    def test_wrong_event_cannot_satisfy_pair(self):
        data = timeline()
        data["cues"][0]["audio_frame"] = 90
        data["cues"].append({"id": "unrelated", "scene_id": "b", "visual_frame": 90, "audio_frame": 90})
        result = TIMELINE.check(data)
        self.assertFalse(result["valid"])
        self.assertIn({"cue_id": "show-button", "problem": "cue_misaligned", "delta_frames": -20}, result["issues"])

    def test_unknown_scene_and_duplicate_cue_fail(self):
        data = timeline()
        data["cues"][0]["scene_id"] = "missing"
        with self.assertRaises(ValueError):
            TIMELINE.check(data)
        data = timeline()
        data["cues"].append(copy.deepcopy(data["cues"][0]))
        with self.assertRaises(ValueError):
            TIMELINE.check(data)

    def test_duplicate_scenes_and_invalid_frames_fail(self):
        data = timeline()
        data["scenes"][1]["id"] = "a"
        with self.assertRaises(ValueError):
            TIMELINE.check(data)
        for value in (True, 60.5, -1, "60"):
            data = timeline()
            data["scenes"][1]["start_frame"] = value
            with self.assertRaises(ValueError):
                TIMELINE.check(data)

    def test_invalid_fps_and_tolerance(self):
        for fps in ("0", "0/1", "30/0", "NaN", "29.97", True):
            data = timeline()
            data["fps"] = fps
            with self.assertRaises((ValueError, ZeroDivisionError)):
                TIMELINE.check(data)
        data = timeline()
        data["tolerance_frames"] = -1
        with self.assertRaises(ValueError):
            TIMELINE.check(data)


class CliTests(unittest.TestCase):
    def run_cli(self, script, text):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input.json"
            path.write_text(text)
            result = subprocess.run([sys.executable, str(script), str(path)], capture_output=True, text=True, timeout=10)
            self.assertEqual(path.read_text(), text)
            self.assertEqual(sorted(p.name for p in Path(directory).iterdir()), ["input.json"])
            json.loads(result.stdout if result.returncode != 1 else result.stderr)
            return result

    def test_successful_and_unsuccessful_exits_preserve_input(self):
        self.assertEqual(self.run_cli(OUTCOME_PATH, json.dumps(evaluation())).returncode, 0)
        data = evaluation()
        data["outcomes"].pop()
        self.assertEqual(self.run_cli(OUTCOME_PATH, json.dumps(data)).returncode, 2)
        self.assertEqual(self.run_cli(TIMELINE_PATH, json.dumps(timeline())).returncode, 0)
        data = timeline()
        data["cues"][0]["audio_frame"] = 100
        self.assertEqual(self.run_cli(TIMELINE_PATH, json.dumps(data)).returncode, 2)

    def test_duplicate_keys_and_nonfinite_json_fail_even_in_metadata(self):
        for script in (OUTCOME_PATH, TIMELINE_PATH):
            for text in ('{"a":1,"a":2}', '{"metadata":{"score":NaN}}', '{"metadata":{"a":1,"a":2}}'):
                self.assertEqual(self.run_cli(script, text).returncode, 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
