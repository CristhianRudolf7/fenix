"""Replay failed summary inputs to isolate the output ceiling from task solving."""

import argparse
import hashlib
import json
import time
from pathlib import Path
from urllib.request import Request, urlopen


def run(root):
    rows = []
    for task in ("db-wal-recovery", "build-cython-ext"):
        source = None
        for path in sorted((root / "api").glob("fenix-" + task + "*/*.request.json")):
            payload = json.loads(path.read_text())
            messages = payload.get("messages") or []
            if not messages or "summarize context" not in str(messages[0].get("content", "")):
                continue
            response_path = path.with_name(path.name.replace(".request.json", ".response.txt"))
            response = json.loads(response_path.read_text())
            if response["choices"][0]["finish_reason"] == "length":
                source = path
                break
        if source is None:
            raise RuntimeError("No failed summary request found for " + task)
        # Alternate ceilings on the same immutable input to avoid grouping all
        # cold-cache observations in one condition. Do not change the prompt.
        for repeat in range(3):
            for cap in (2048, 16384):
                payload = json.loads(source.read_text())
                payload["max_tokens"] = cap
                identifier = f"probe-summary-{task}-{cap}-{repeat}"
                started = time.monotonic()
                request = Request("http://172.17.0.1:48931/" + identifier + "/v1/chat/completions",
                                  json.dumps(payload).encode(), {"Content-Type": "application/json"})
                with urlopen(request, timeout=180) as response:
                    result = json.load(response)
                choice = result["choices"][0]
                message = choice["message"]
                text = message.get("content")
                accepted = (choice["finish_reason"] == "stop" and isinstance(text, str)
                            and bool(text.strip()) and not message.get("tool_calls"))
                row = {"task": task, "repeat": repeat, "max_tokens": cap,
                       "source_request": str(source),
                       "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                       "api_trial": identifier, "seconds": time.monotonic() - started,
                       "finish_reason": choice["finish_reason"],
                       "passes_summary_shape_check": accepted,
                       "visible_characters": len(text or ""), "usage": result.get("usage")}
                rows.append(row)
                (root / "compaction-probes.json").write_text(json.dumps(rows, indent=2))
                print(json.dumps(row), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    run(parser.parse_args().root)
