#!/usr/bin/env python3
"""Recompute screening counts without judging study eligibility."""

import argparse
from collections import Counter
from datetime import date
import json
from pathlib import Path
import sys

STATES = {"pending", "duplicate", "screen_excluded", "full_text_pending", "not_retrieved", "full_text_excluded", "included"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def text(value):
    return isinstance(value, str) and bool(value.strip())


def index(rows, label):
    require(isinstance(rows, list), f"{label} must be an array")
    result = {}
    for row in rows:
        require(isinstance(row, dict) and text(row.get("id")), f"Invalid {label} row")
        require(row["id"] not in result, f"Duplicate {label} id")
        result[row["id"]] = row
    return result


def count_screening(document):
    require(isinstance(document, dict), "Ledger must be an object")
    searches = index(document.get("searches"), "search")
    records = index(document.get("records"), "record")
    require(bool(searches), "At least one search is required")
    for search in searches.values():
        require(text(search.get("database")) and text(search.get("query")), "Missing search provenance")
        stamp = search.get("searched_on")
        require(isinstance(stamp, str) and len(stamp) == 10, "Search date must be YYYY-MM-DD")
        date.fromisoformat(stamp)
    counts = Counter()
    studies = set()
    for key, row in records.items():
        state = row.get("state")
        require(isinstance(state, str) and state in STATES, "Unknown screening state")
        origin = row.get("search_id")
        require(isinstance(origin, str) and origin in searches, "Unknown search origin")
        if state in {"duplicate", "screen_excluded", "not_retrieved", "full_text_excluded"}:
            require(text(row.get("reason")), "A removal or nonretrieval needs a reason")
        if state == "duplicate":
            target = row.get("duplicate_of")
            require(isinstance(target, str) and target in records and target != key, "Invalid duplicate target")
            require(records[target].get("state") != "duplicate", "Duplicate chains are ambiguous")
        else:
            require("duplicate_of" not in row, "Only duplicate records may have duplicate_of")
        if state == "included":
            require(text(row.get("study_id")), "Included reports need a study_id")
            studies.add(row["study_id"])
        counts[state] += 1
    screened = len(records) - counts["duplicate"] - counts["pending"]
    sought = screened - counts["screen_excluded"]
    assessed = sought - counts["full_text_pending"] - counts["not_retrieved"]
    return {
        "identified_records": len(records), "duplicates_removed": counts["duplicate"],
        "screening_pending": counts["pending"], "records_screened": screened,
        "records_excluded": counts["screen_excluded"], "reports_sought": sought,
        "full_text_pending": counts["full_text_pending"], "reports_not_retrieved": counts["not_retrieved"],
        "reports_assessed": assessed, "reports_excluded": counts["full_text_excluded"],
        "reports_included": counts["included"], "studies_included": len(studies),
        "screening_complete": counts["pending"] + counts["full_text_pending"] == 0,
        "eligibility_verified_by_tool": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ledger", type=Path)
    args = parser.parse_args()
    try:
        with args.ledger.open("rb") as stream:
            data = stream.read(16 * 1024 * 1024 + 1)
        require(len(data) <= 16 * 1024 * 1024, "Ledger exceeds 16 MiB")
        result = count_screening(json.loads(data))
    except (ValueError, OSError, TypeError, KeyError) as error:
        print(json.dumps({"error": str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
