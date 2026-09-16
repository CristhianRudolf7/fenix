"""Keep missing provider billing distinct from an explicitly free request."""
import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from experiment import summarize


class BillingSummaryTests(unittest.TestCase):
    def test_partial_or_invalid_billing_never_becomes_zero_cost(self):
        cases = [
            ([{}, {"cost": 0.01}], None),
            ([{"cost": None}], None),
            ([{"cost": "unknown"}], None),
            ([{"cost": False}], None),
            ([{"cost": float("nan")}], None),
            ([{"cost": float("inf")}], None),
            ([{"cost": 0}], 0),
            ([{"cost": 0.01}, {"cost": 0.02}], 0.03),
        ]
        for usages, expected in cases:
            with self.subTest(usages=usages), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                trial = root / "jobs/cohort/task__trial"
                trial.mkdir(parents=True)
                (trial / "result.json").write_text(json.dumps({
                    "finished_at": "2026-09-16T00:00:00Z", "task_name": "task",
                    "verifier_result": {"rewards": {"reward": 0}},
                }))
                api = root / "api/native-task__trial"
                api.mkdir(parents=True)
                for index, usage in enumerate(usages):
                    (api / f"{index}.metadata.json").write_text(json.dumps({
                        "http_status": 200, "usage": usage,
                    }))
                with contextlib.redirect_stdout(io.StringIO()):
                    summarize(root)
                cost = json.loads((root / "results.json").read_text())[0]["cost_usd"]
                if expected is None:
                    self.assertIsNone(cost)
                else:
                    self.assertAlmostEqual(cost, expected)


if __name__ == "__main__":
    unittest.main()
