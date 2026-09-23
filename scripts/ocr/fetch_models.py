"""Build-time only: fetch immutable official models and verify every file."""
import hashlib
import json
from pathlib import Path
import sys
import urllib.request


def fetch(destination):
    manifest = json.loads(Path(__file__).with_name("models.lock.json").read_text())
    for model in manifest["models"]:
        folder = destination / model["name"]
        folder.mkdir(parents=True, exist_ok=True)
        for entry in model["files"]:
            url = f'https://huggingface.co/PaddlePaddle/{model["name"]}/resolve/{model["revision"]}/{entry["name"]}'
            request = urllib.request.Request(url, headers={"User-Agent": "fenix-ocr-image-build"})
            with urllib.request.urlopen(request, timeout=120) as response:
                data = response.read(entry["bytes"] + 1)
            if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise RuntimeError("Model checksum mismatch: " + model["name"] + "/" + entry["name"])
            (folder / entry["name"]).write_bytes(data)
    (destination / "models.lock.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    fetch(Path(sys.argv[1]))
