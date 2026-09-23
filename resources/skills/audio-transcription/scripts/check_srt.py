#!/usr/bin/env python3
"""Check SRT structure and timing; report overlap without judging speech accuracy."""

import argparse
import json
import math
from pathlib import Path
import re
import sys

STAMP = r"(\d{2,}):([0-5]\d):([0-5]\d),(\d{3})"


def milliseconds(match):
    hours, minutes, seconds, millis = map(int, match)
    return ((hours * 60 + minutes) * 60 + seconds) * 1000 + millis


def check(text, duration=None):
    if duration is not None and (not math.isfinite(duration) or duration < 0):
        raise ValueError("Duration must be finite and nonnegative")
    text = text.lstrip("\ufeff").replace("\r\n", "\n").strip()
    if not text:
        raise ValueError("No subtitle cues")
    cues, overlaps = [], []
    previous_start, latest_end = -1, -1
    for expected, block in enumerate(re.split(r"\n[ \t]*\n", text), start=1):
        lines = block.splitlines()
        if len(lines) < 3 or lines[0].strip() != str(expected):
            raise ValueError("Expected sequential cue indices and nonempty text")
        match = re.fullmatch(STAMP + r" --> " + STAMP, lines[1].strip())
        if not match:
            raise ValueError("Invalid SRT timestamp line")
        start, end = milliseconds(match.groups()[:4]), milliseconds(match.groups()[4:])
        if end <= start or start < previous_start:
            raise ValueError("Invalid or out-of-order cue interval")
        if duration is not None and end > duration * 1000:
            raise ValueError("Cue extends beyond media duration")
        if not "\n".join(lines[2:]).strip():
            raise ValueError("Empty cue text")
        if start < latest_end:
            overlaps.append(expected)
        previous_start, latest_end = start, max(latest_end, end)
        cues.append({"index": expected, "start_ms": start, "end_ms": end})
    return {"status": "valid", "cue_count": len(cues), "end_ms": latest_end,
            "overlapping_cues": overlaps, "media_bound_checked": duration is not None,
            "speech_accuracy_verified": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--duration", type=float)
    args = parser.parse_args()
    try:
        with args.input.open("rb") as stream:
            data = stream.read(8 * 1024 * 1024 + 1)
        if len(data) > 8 * 1024 * 1024:
            raise ValueError("Input exceeds 8 MiB")
        result = check(data.decode("utf-8"), args.duration)
    except (ValueError, OSError, OverflowError) as error:
        print(json.dumps({"status": "invalid", "error": str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
