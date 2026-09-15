"""Remove audit-only images after both baseline harnesses finished a task."""

import argparse
import json
import subprocess
from pathlib import Path


def cleanup(root):
    marker = root / "cleaned-task-images.json"
    done = set(json.loads(marker.read_text())) if marker.exists() else set()
    initial = (root / "initial-docker-images.txt").read_text()
    selection = json.loads((root / "selection.json").read_text())["tasks"]
    completed = {}
    for path in (root / "jobs").glob("*/*/result.json"):
        if path.parent.parent.name.startswith("diagnostic"):
            continue
        try:
            row = json.loads(path.read_text())
        except (OSError, ValueError):
            continue
        if row.get("finished_at"):
            completed.setdefault(row["task_name"].split("/")[-1], set()).add(row["agent_info"]["name"])
    with (root / "image-cleanup.log").open("a") as log:
        for task in selection:
            name = task["name"]
            if name in done or not {"fenix", "terminus-2"}.issubset(completed.get(name, set())):
                continue
            image = task["resources"].get("docker_image")
            if not image or image in initial:
                done.add(name)
                continue
            probe = subprocess.run(["docker", "image", "inspect", image,
                                    "--format", "{{json .Id}} {{json .RepoDigests}}"],
                                   capture_output=True, text=True)
            if probe.returncode:
                done.add(name)
                continue
            log.write(image + " " + probe.stdout)
            removed = subprocess.run(["docker", "image", "rm", image], stdout=log, stderr=log)
            if removed.returncode == 0:
                done.add(name)
            log.flush()
    marker.write_text(json.dumps(sorted(done), indent=2))
    print("Image cleanup complete for", len(done), "finished task pairs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    cleanup(parser.parse_args().root)
