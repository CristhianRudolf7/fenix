#!/usr/bin/env python3
"""Check local evidence bindings; never infer the truth of a research claim."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sys

MAX_BYTES = 16 * 1024 * 1024
STATUSES = {"supported", "contradicted", "overstated", "unresolved"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_bounded(path):
    with path.open("rb") as stream:
        data = stream.read(MAX_BYTES + 1)
    require(len(data) <= MAX_BYTES, "Input exceeds 16 MiB")
    return data


def nonempty(value, label):
    require(isinstance(value, str) and bool(value.strip()), f"Missing {label}")
    return value


def index_rows(rows, label):
    require(isinstance(rows, list), f"{label} must be an array")
    result = {}
    for row in rows:
        require(isinstance(row, dict), f"Invalid {label} row")
        key = nonempty(row.get("id"), f"{label} id")
        require(key not in result, f"Duplicate {label} id: {key}")
        result[key] = row
    return result


def read_source(binding, root):
    require(isinstance(binding, dict), "Invalid source binding")
    relative = Path(nonempty(binding.get("path"), "source path"))
    require(not relative.is_absolute(), "Source path must be relative")
    path = (root / relative).resolve(strict=True)
    require(path.is_relative_to(root), "Source escapes the project root")
    require(path.is_file(), "Source is not a regular file")
    digest = binding.get("sha256")
    require(isinstance(digest, str) and bool(re.fullmatch(r"[0-9a-f]{64}", digest)), "Invalid SHA-256")
    data = read_bounded(path)
    require(hashlib.sha256(data).hexdigest() == digest, f"Stale source: {relative}")
    return data.decode("utf-8").splitlines()


def check_locator(row, lines, key):
    start, end = row.get("line_start"), row.get("line_end")
    require(type(start) is int and type(end) is int, "Line bounds must be integers")
    require(1 <= start <= end <= len(lines), "Line locator outside the source")
    text = nonempty(row.get(key), key)
    require(text in "\n".join(lines[start - 1:end]), f"{key} not present at the recorded locator")


def audit(document, root):
    require(isinstance(document, dict), "Ledger must be an object")
    root = Path(root).resolve(strict=True)
    manuscript = read_source(document.get("manuscript"), root)
    sources = index_rows(document.get("sources"), "source")
    claims = index_rows(document.get("claims"), "claim")
    require(bool(claims), "Ledger contains no claims")
    texts = {key: read_source(row, root) for key, row in sources.items()}
    statuses = Counter()
    evidence_count = 0
    for claim in claims.values():
        check_locator(claim, manuscript, "text")
        status = claim.get("status")
        require(isinstance(status, str) and status in STATUSES, "Unknown claim status")
        nonempty(claim.get("reason"), "judgment reason")
        evidence = claim.get("evidence")
        require(isinstance(evidence, list), "Evidence must be an array")
        require(status == "unresolved" or bool(evidence), "Resolved claim has no evidence")
        for item in evidence:
            require(isinstance(item, dict), "Invalid evidence item")
            source_id = item.get("source_id")
            require(isinstance(source_id, str) and source_id in texts, "Unknown evidence source")
            check_locator(item, texts[source_id], "quote")
            evidence_count += 1
        statuses[status] += 1
    return {
        "structural_status": "valid",
        "claims_declared": len(claims),
        "sources_bound": len(sources),
        "evidence_locators_checked": evidence_count,
        "supplied_judgments": dict(sorted(statuses.items())),
        "semantic_support_verified_by_tool": False,
        "source_authenticity_verified_by_tool": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ledger", type=Path)
    parser.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = audit(json.loads(read_bounded(args.ledger)), args.root)
    except (ValueError, OSError, TypeError, KeyError) as error:
        print(json.dumps({"structural_status": "invalid", "error": str(error)}))
        return 1
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
