"""Summarize official verifier outcomes and observed provider billing separately."""

import argparse
import collections
import csv
import datetime
import json
from pathlib import Path


def duration(phase):
    if not phase or not phase.get("finished_at"):
        return None
    return (datetime.datetime.fromisoformat(phase["finished_at"]) -
            datetime.datetime.fromisoformat(phase["started_at"])).total_seconds()


def summarize(root):
    rows = []
    exclusions_path = root / "baseline-exclusions.json"
    exclusions = json.loads(exclusions_path.read_text()) if exclusions_path.exists() else {}
    for path in sorted((root / "jobs").glob("*/*/result.json")):
        result = json.loads(path.read_text())
        agent = result["agent_info"]["name"]
        name = result["task_name"].split("/")[-1]
        metadata = (result.get("agent_result") or {}).get("metadata") or {}
        identifier = metadata.get("api_trial", "terminus2-" + name)
        calls = [json.loads(p.read_text()) for p in (root / "api" / identifier).glob("*.metadata.json")]
        request_paths = list((root / "api" / identifier).glob("*.request.json"))
        call_purposes = collections.Counter()
        for request_path in request_paths:
            payload = json.loads(request_path.read_text())
            messages = payload.get("messages") or []
            first = str(messages[0].get("content", "")) if messages else ""
            if first == "Reply with OK.":
                purpose = "connection_check"
            elif "summarize context" in first:
                purpose = "active_compaction"
            elif payload.get("tools"):
                purpose = "agent_tool_loop"
            else:
                purpose = "reference_or_background"
            call_purposes[purpose] += 1
        usages = [c["usage"] for c in calls if c.get("usage")]
        job = path.parent.parent.name
        cohort = ("excluded_setup" if result["trial_name"] in exclusions else
                  "diagnostic" if job.startswith("diagnostic") else "baseline")
        counts = collections.Counter()
        tools = collections.Counter()
        errors = []
        tool_seconds = collections.Counter()
        starts = {}
        nonzero_commands = set()
        timed_out_commands = set()
        quick_completed_polls = 0
        for detail in (path.parent / "agent").glob("task-*.json"):
            for event in json.loads(detail.read_text())["events"]:
                counts[event["kind"]] += 1
                data = event["data"]
                if event["kind"] == "tool_started":
                    tools[data.get("tool")] += 1
                    starts[data["call_id"]] = event["created_at"]
                if event["kind"] == "tool_result" and data.get("call_id") in starts:
                    elapsed = (datetime.datetime.fromisoformat(event["created_at"]) -
                               datetime.datetime.fromisoformat(starts[data["call_id"]])).total_seconds()
                    tool_seconds[data.get("tool")] += elapsed
                    if data.get("tool") == "command_status":
                        outcome = data.get("result") or {}
                        exit_code = outcome.get("exit_code")
                        if isinstance(exit_code, int) and exit_code != 0:
                            nonzero_commands.add(outcome.get("process_id"))
                        if outcome.get("status") == "timed_out":
                            timed_out_commands.add(outcome.get("process_id"))
                        if outcome.get("status") == "completed" and elapsed < 0.05:
                            quick_completed_polls += 1
                if event["kind"] == "tool_result" and data.get("is_error"):
                    errors.append({"tool": data.get("tool"), "result": data.get("result")})
        verifier = result.get("verifier_result") or {}
        reward = (verifier.get("rewards") or {}).get("reward")
        failure = result.get("exception_info") or {}
        test_report = path.parent / "verifier/ctrf.json"
        checks = json.loads(test_report.read_text()).get("results", {}).get("summary", {}) if test_report.exists() else {}
        runner_path = path.parent / "agent/runner.json"
        runner = json.loads(runner_path.read_text()) if runner_path.exists() else {}
        root_task_id = (runner.get("submitted") or {}).get("task_id")
        root_detail_path = path.parent / "agent" / ("task-" + str(root_task_id) + ".json")
        root_task = json.loads(root_detail_path.read_text()).get("task", {}) if root_detail_path.exists() else {}
        row = {"task": name, "agent": agent, "trial": result["trial_name"],
               "job": job, "cohort": cohort,
               "exclusion_reason": exclusions.get(result["trial_name"]), "reward": reward,
               "error_type": failure.get("exception_type"),
               "error": failure.get("exception_message"),
               "runtime_status": root_task.get("status"), "runtime_error": root_task.get("error"),
               "controller_stop_reason": runner.get("stop_reason"),
               "terminal_quiescence_seconds": runner.get("terminal_quiescence_seconds", 0),
               "checks_passed": checks.get("passed"), "checks_failed": checks.get("failed"),
               "checks_total": checks.get("tests"),
               "agent_seconds": duration(result.get("agent_execution")),
               "environment_seconds": duration(result.get("environment_setup")),
               "setup_seconds": duration(result.get("agent_setup")),
               "verifier_seconds": duration(result.get("verifier")),
               "api_calls": len(calls), "calls_with_usage": len(usages),
               "calls_with_reported_cost": sum("cost" in u for u in usages),
               "requests_recorded": len(request_paths),
               "requests_without_completed_metadata": len(request_paths) - len(calls),
               "api_seconds_sum": sum(c.get("seconds", 0) for c in calls),
               "input_tokens": sum(u.get("prompt_tokens", 0) for u in usages),
               "cached_input_tokens": sum((u.get("prompt_tokens_details") or {}).get("cached_tokens", 0) for u in usages),
               "output_tokens": sum(u.get("completion_tokens", 0) for u in usages),
               "provider_reported_usd": sum(u.get("cost", 0) for u in usages),
               "api_errors": sum(c.get("http_status") != 200 or bool(c.get("error")) for c in calls),
               "distinct_commands_observed_nonzero": len(nonzero_commands),
               "distinct_commands_observed_timed_out": len(timed_out_commands),
               "completed_status_polls_under_50ms": quick_completed_polls,
               "events": dict(counts), "tools": dict(tools), "call_purposes": dict(call_purposes),
               "tool_seconds_sum": dict(tool_seconds),
               "tool_errors": errors,
               "result_path": str(path)}
        rows.append(row)
    (root / "summary.json").write_text(json.dumps(rows, indent=2))
    fields = [k for k in rows[0] if k not in ("events", "tools", "tool_errors", "call_purposes", "tool_seconds_sum")] if rows else []
    with (root / "summary.csv").open("w") as stream:
        writer = csv.DictWriter(stream, fieldnames=fields, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)
    for row in rows:
        print(f"{row['agent']:10} {row['task']:30} reward={row['reward']} "
              f"seconds={row['agent_seconds']} calls={row['api_calls']} "
              f"cost=${row['provider_reported_usd']:.5f} error={row['error_type']}")
    return rows


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    summarize(parser.parse_args().root)
