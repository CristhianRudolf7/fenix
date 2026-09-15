"""Run a frozen, paired Terminal-Bench subset and retain resumable job results."""

import argparse
import json
import os
import shutil
import subprocess
import time
from pathlib import Path


def run(root):
    root = root.resolve()
    selection = json.loads((root / "selection.json").read_text())["tasks"]
    initial_images = (root / "initial-docker-images.txt").read_text()
    reference = json.loads((root / "reference-smoke.json").read_text())["agents"][0]
    fenix = json.loads((root / "smoke-fixed.json").read_text())["agents"][0]
    configs = root / "configs"
    configs.mkdir(exist_ok=True)
    env = dict(os.environ, FENIX_AUDIT_ROOT=str(root),
               PYTHONPATH=str(Path(__file__).resolve().parent), TMPDIR=str(root / "tmp"))
    first_jobs = [root / "jobs" / name / "result.json" for name in
                  ("01-cancel-async-tasks-fenix", "01-cancel-async-tasks-reference")]
    while not all(p.exists() and json.loads(p.read_text()).get("finished_at") for p in first_jobs):
        time.sleep(5)
    for index, task in enumerate(selection[1:], 2):
        name = task["name"]
        job_name = f"{index:02d}-{name}"
        result_path = root / "jobs" / job_name / "result.json"
        if result_path.exists() and json.loads(result_path.read_text()).get("finished_at"):
            continue
        if shutil.disk_usage(root).free < 3 * 1024 ** 3:
            raise RuntimeError("Less than 3 GiB free; preserve existing jobs and free audit images before resuming")
        reference["kwargs"]["api_base"] = "http://172.17.0.1:48931/terminus2-" + name + "/v1"
        config = {"job_name": job_name, "jobs_dir": str(root / "jobs"),
                  "n_concurrent_trials": 2, "n_attempts": 1,
                  "retry": {"max_retries": 0}, "environment": {"type": "docker", "delete": True},
                  "agents": [fenix, reference], "tasks": [{"path": str(root / "dataset/tasks" / name)}]}
        path = configs / (job_name + ".json")
        path.write_text(json.dumps(config, indent=2))
        print(json.dumps({"starting": index, "task": name, "time": time.time()}), flush=True)
        with (root / (job_name + ".log")).open("w") as log:
            completed = subprocess.run([str(root / "venv/bin/harbor"), "run", "-c", str(path)],
                                       env=env, stdout=log, stderr=subprocess.STDOUT)
        print(json.dumps({"finished": index, "task": name, "exit_code": completed.returncode,
                          "time": time.time()}), flush=True)
        # Remove only the exact benchmark image introduced by this audit, without force.
        image = task["resources"].get("docker_image")
        if image and image not in initial_images:
            with (root / "image-cleanup.log").open("a") as log:
                subprocess.run(["docker", "image", "rm", image], stdout=log, stderr=log)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    run(parser.parse_args().root)
