"""Keep two trial slots occupied while removing only unused audit images."""

import argparse
import json
import os
import signal
import subprocess
import time
from pathlib import Path

from cleanup_completed import cleanup as cleanup_completed


def run(root):
    root = root.resolve()
    if (root / "jobs/remaining-08-20").exists():
        raise RuntimeError("The continuation job already exists; inspect it instead of launching duplicate trials")
    prior = root / "jobs/07-cobol-modernization/result.json"
    while not prior.exists() or not json.loads(prior.read_text()).get("finished_at"):
        time.sleep(3)
    # The old parent's child has finished. Retire that paused scheduler so it
    # cannot launch duplicate tasks when its wait() returns.
    for pid in json.loads((root / "paused-batch-pids.json").read_text()):
        try:
            os.kill(pid, signal.SIGTERM)
            os.kill(pid, signal.SIGCONT)
        except ProcessLookupError:
            pass
    env = dict(os.environ, FENIX_AUDIT_ROOT=str(root),
               PYTHONPATH=str(Path(__file__).resolve().parent), TMPDIR=str(root / "tmp"))
    seen = set()
    with (root / "remaining.log").open("w") as log:
        process = subprocess.Popen([str(root / "venv/bin/harbor"), "run", "-c", str(root / "remaining.json")],
                                   env=env, stdout=log, stderr=subprocess.STDOUT)
        while True:
            for path in (root / "jobs").glob("*/*/result.json"):
                if str(path) in seen:
                    continue
                try:
                    result = json.loads(path.read_text())
                except (OSError, ValueError):
                    continue
                if not result.get("finished_at"):
                    continue
                seen.add(str(path))
                name = result["task_name"].split("/")[-1]
                print(json.dumps({"task": name, "agent": result["agent_info"]["name"],
                                  "reward": result.get("verifier_result"),
                                  "error": (result.get("exception_info") or {}).get("exception_type"),
                                  "trial": result["trial_name"]}), flush=True)
            # Wait for both harnesses: a finished first trial can otherwise
            # remove an image while its paired trial is about to start.
            cleanup_completed(root)
            if process.poll() is not None:
                print(json.dumps({"queue_exit_code": process.returncode}), flush=True)
                break
            time.sleep(10)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    run(parser.parse_args().root)
