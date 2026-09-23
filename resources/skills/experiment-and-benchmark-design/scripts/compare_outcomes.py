#!/usr/bin/env python3
"""Summarize declared paired task outcomes without running or modifying agents."""

import argparse
import json
import math
import sys
from pathlib import Path


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError(f"Non-finite JSON number: {value}")


def identifier(value):
    if not isinstance(value, str) or not value.strip() or len(value) > 500:
        raise ValueError("IDs and provenance must be nonempty strings up to 500 characters")
    return value


def number(value, maximum=None):
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        raise ValueError("Expected a finite nonnegative number")
    if maximum is not None and value > maximum:
        raise ValueError(f"Number exceeds {maximum}")
    return value


def compare(data):
    if not isinstance(data, dict) or type(data.get("schema_version")) is not int or data["schema_version"] != 1:
        raise ValueError("Expected schema_version 1")
    conditions = data["conditions"]
    if not isinstance(conditions, dict) or set(conditions) != {"baseline", "candidate"}:
        raise ValueError("Exactly baseline and candidate conditions are required")
    contracts = []
    for condition in conditions.values():
        identifier(condition["revision"])
        contracts.append(identifier(condition["execution_contract"]))
    if contracts[0] != contracts[1]:
        raise ValueError("Declared execution contracts differ; controlled pairing is unsupported")
    tasks, runs, rows = data["tasks"], data["run_ids"], data["outcomes"]
    if not all(isinstance(value, list) for value in (tasks, runs, rows)) or not tasks or not runs:
        raise ValueError("Nonempty tasks/run_ids and an outcomes list are required")
    if len(tasks) * len(runs) > 100000 or len(rows) > 200000:
        raise ValueError("Planned comparison exceeds 100000 task/run pairs")
    task_map = {}
    for task in tasks:
        key = identifier(task["id"])
        if key in task_map or task["split"] not in ("development", "selection", "test"):
            raise ValueError("Duplicate task or unknown split")
        if type(task["critical"]) is not bool:
            raise ValueError("critical must be boolean")
        task_map[key] = task
    run_set = {identifier(run) for run in runs}
    if len(run_set) != len(runs):
        raise ValueError("Duplicate run ID")
    gate = data.get("gate", {})
    min_delta = number(gate.get("min_delta", 0), 1)
    max_regression = number(gate.get("max_task_regression", 0), 1)
    indexed = {}
    usage = {condition: {metric: {"known_trials": 0, "total": 0} for metric in ("tokens", "seconds", "cost")} for condition in conditions}
    statuses = {condition: {status: 0 for status in ("completed", "timeout", "error", "missing")} for condition in conditions}
    for row in rows:
        condition, task_id, run_id = row["condition"], row["task_id"], row["run_id"]
        if condition not in conditions or task_id not in task_map or run_id not in run_set:
            raise ValueError("Outcome refers to an undeclared condition, task or run")
        key = (condition, task_id, run_id)
        if key in indexed:
            raise ValueError("Duplicate outcome identity")
        score = number(row["score"], 1)
        status = row["status"]
        if status not in ("completed", "timeout", "error") or (status != "completed" and score != 0):
            raise ValueError("Invalid status or nonzero score for an unfinished trial")
        identifier(row["evidence"])
        indexed[key] = row
        statuses[condition][status] += 1
        for metric in usage[condition]:
            if metric in row:
                value = number(row[metric])
                if metric == "tokens" and type(value) is not int:
                    raise ValueError("tokens must be an integer")
                usage[condition][metric]["known_trials"] += 1
                total = usage[condition][metric]["total"] + value
                usage[condition][metric]["total"] = number(total)
    missing, per_task = [], []
    for task_id, task in task_map.items():
        means = {}
        for condition in conditions:
            scores = []
            for run_id in runs:
                row = indexed.get((condition, task_id, run_id))
                if row is None:
                    missing.append({"condition": condition, "task_id": task_id, "run_id": run_id})
                    statuses[condition]["missing"] += 1
                scores.append(row["score"] if row else 0)
            means[condition] = math.fsum(scores) / len(runs)
        per_task.append({"task_id": task_id, "split": task["split"], "critical": task["critical"], **means, "delta": means["candidate"] - means["baseline"]})
    by_split = {}
    for split in ("development", "selection", "test"):
        selected = [row for row in per_task if row["split"] == split]
        if selected:
            by_split[split] = {"tasks": len(selected), "planned_trials_per_condition": len(selected) * len(runs), "baseline": math.fsum(row["baseline"] for row in selected) / len(selected), "candidate": math.fsum(row["candidate"] for row in selected) / len(selected), "delta": math.fsum(row["delta"] for row in selected) / len(selected), "improved": [row["task_id"] for row in selected if row["delta"] > 0], "regressed": [row["task_id"] for row in selected if row["delta"] < 0]}
    reasons = []
    selection = [row for row in per_task if row["split"] == "selection"]
    if not selection:
        reasons.append("No selection tasks")
    elif by_split["selection"]["delta"] <= min_delta:
        reasons.append("Selection gain does not exceed min_delta")
    for row in selection:
        if row["delta"] < 0 and (row["critical"] or -row["delta"] > max_regression):
            reasons.append("Task regression exceeds gate: " + row["task_id"])
        if row["critical"]:
            for run_id in runs:
                before = indexed.get(("baseline", row["task_id"], run_id))
                after = indexed.get(("candidate", row["task_id"], run_id))
                if before is not None and after is not None and after["score"] < before["score"]:
                    reasons.append(f"Critical trial regressed: {row['task_id']}/{run_id}")
    if any(task_map[row["task_id"]]["split"] == "selection" for row in missing):
        reasons.append("Selection outcomes are missing")
    planned = len(tasks) * len(runs)
    for metrics in usage.values():
        for values in metrics.values():
            values["missing_trials"] = planned - values["known_trials"]
            if not values["known_trials"]:
                values["total"] = None
    return {"schema_version": 1, "conditions": conditions, "by_split": by_split, "per_task": per_task, "missing": missing, "statuses": statuses, "usage": usage, "gate": {"meets_declared_gate": not reasons, "reasons": reasons, "min_delta": min_delta, "max_task_regression": max_regression}, "evidence_verified": False, "statistical_superiority_established": False, "agent_executed": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path)
    args = parser.parse_args()
    try:
        with args.results.open("rb") as handle:
            raw = handle.read(8 * 1024 * 1024 + 1)
        if len(raw) > 8 * 1024 * 1024:
            raise ValueError("Input exceeds 8 MiB")
        data = json.loads(raw.decode("utf-8-sig"), object_pairs_hook=unique_object, parse_constant=reject_constant)
        result = compare(data)
        print(json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2))
        return 0 if result["gate"]["meets_declared_gate"] else 2
    except (OSError, ValueError, KeyError, TypeError, AttributeError, OverflowError, RecursionError) as exc:
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
