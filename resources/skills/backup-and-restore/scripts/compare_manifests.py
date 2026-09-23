#!/usr/bin/env python3
"""Compare supplied snapshot manifests; never traverse or alter the recorded paths."""

import argparse
import json
import re
import sys

MAX_BYTES = 8 * 1024 * 1024
MAX_FILES = 100_000
FIELDS = {"schema_version", "snapshot_id", "algorithm", "complete", "errors", "files"}


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("Duplicate JSON key")
        result[key] = value
    return result


def read_manifest(path):
    with open(path, "rb") as source:
        raw = source.read(MAX_BYTES + 1)
    if len(raw) > MAX_BYTES:
        raise ValueError("Manifest exceeds 8 MiB")
    return json.loads(raw, object_pairs_hook=unique_object)


def validate(data):
    if not isinstance(data, dict) or set(data) != FIELDS:
        raise ValueError("Expected exactly the documented manifest fields")
    if type(data["schema_version"]) is not int or data["schema_version"] != 1:
        raise ValueError("Unsupported manifest schema")
    if not isinstance(data["snapshot_id"], str) or not data["snapshot_id"].strip():
        raise ValueError("A nonempty snapshot_id is required")
    if data["algorithm"] != "sha256":
        raise ValueError("Only SHA-256 manifests are supported")
    if type(data["complete"]) is not bool:
        raise ValueError("complete must be a boolean")
    if not isinstance(data["errors"], list) or not all(
        isinstance(error, str) and error.strip() for error in data["errors"]
    ):
        raise ValueError("errors must be a list of nonempty strings")
    files = data["files"]
    if not isinstance(files, dict) or len(files) > MAX_FILES:
        raise ValueError("files must be a mapping of at most 100000 paths")
    for path, digest in files.items():
        if (
            not isinstance(path, str)
            or not path
            or "\\" in path
            or ":" in path
            or any(ord(char) < 32 or ord(char) == 127 for char in path)
            or any(part in ("", ".", "..") for part in path.split("/"))
        ):
            raise ValueError("File paths must be unambiguous relative POSIX paths")
        if not isinstance(digest, str) or not re.fullmatch(r"[a-fA-F0-9]{64}", digest):
            raise ValueError("Each file needs a SHA-256 digest, not an error placeholder")
    # A regular file cannot also be a parent directory in this format.
    for path in files:
        parts = path.split("/")
        if any("/".join(parts[:i]) in files for i in range(1, len(parts))):
            raise ValueError("Conflicting file and directory paths")


def compare(baseline, restored):
    validate(baseline)
    validate(restored)
    if baseline["snapshot_id"] != restored["snapshot_id"]:
        raise ValueError("Baseline and restore must identify the same snapshot")
    before, after = baseline["files"], restored["files"]
    missing = sorted(before.keys() - after.keys())
    added = sorted(after.keys() - before.keys())
    changed = sorted(
        path for path in before.keys() & after.keys()
        if before[path].lower() != after[path].lower()
    )
    gaps = []
    for label, manifest in (("baseline", baseline), ("restored", restored)):
        if not manifest["complete"]:
            gaps.append(f"{label}: collection incomplete")
        if manifest["errors"]:
            gaps.append(f"{label}: {len(manifest['errors'])} collection errors")
        if not manifest["files"]:
            gaps.append(f"{label}: empty inventory")
    verdict = "mismatch" if missing or added or changed else "inconclusive" if gaps else "match"
    return {
        "verdict": verdict,
        "baseline_files": len(before),
        "restored_files": len(after),
        "missing": missing,
        "added": added,
        "changed": changed,
        "evidence_gaps": gaps,
        "scope": "Supplied file manifests only; file contents and application recovery not tested",
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline")
    parser.add_argument("restored")
    args = parser.parse_args(argv)
    try:
        result = compare(read_manifest(args.baseline), read_manifest(args.restored))
    except (OSError, ValueError, RecursionError) as error:
        # Do not echo parser input, paths or collection-error strings into shared logs.
        print(json.dumps({"verdict": "invalid", "error_type": type(error).__name__}))
        return 2
    print(json.dumps(result, ensure_ascii=True, indent=2))
    return {"match": 0, "mismatch": 1, "inconclusive": 2}[result["verdict"]]


if __name__ == "__main__":
    sys.exit(main())
