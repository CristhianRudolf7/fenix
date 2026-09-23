#!/usr/bin/env python3
"""Check a primary scene track and explicit audiovisual cue pairs, without rendering."""

import argparse
from fractions import Fraction
import json
from pathlib import Path
import re
import sys


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError(f"Non-finite JSON number: {value}")


def frame(value):
    if type(value) is not int or not 0 <= value <= 1000000000:
        raise ValueError("Frames must be integers between 0 and 1000000000")
    return value


def identifier(value):
    if not isinstance(value, str) or not value.strip() or len(value) > 500:
        raise ValueError("IDs must be nonempty strings up to 500 characters")
    return value


def check(data):
    fps_text = data["fps"]
    if not isinstance(fps_text, str) or not re.fullmatch(r"[0-9]{1,9}(?:/[0-9]{1,9})?", fps_text):
        raise ValueError("fps must be a positive integer or fraction string")
    fps = Fraction(fps_text)
    if fps <= 0:
        raise ValueError("fps must be positive")
    duration = frame(data["duration_frames"])
    tolerance = frame(data.get("tolerance_frames", 2))
    scenes, cues = data["scenes"], data.get("cues", [])
    if not isinstance(scenes, list) or not scenes or not isinstance(cues, list) or len(scenes) + len(cues) > 100000:
        raise ValueError("Expected a nonempty scene list and at most 100000 scene/cue rows")
    indexed, issues = {}, []
    cursor = 0
    for scene in scenes:
        key = identifier(scene["id"])
        if key in indexed:
            raise ValueError("Duplicate scene ID: " + key)
        start, end = frame(scene["start_frame"]), frame(scene["end_frame"])
        indexed[key] = (start, end)
        if start != cursor:
            issues.append({"scene_id": key, "problem": "gap" if start > cursor else "overlap_or_out_of_order", "expected_start": cursor, "actual_start": start})
        if end <= start or end > duration:
            issues.append({"scene_id": key, "problem": "invalid_scene_bounds"})
        cursor = end
    if cursor != duration:
        issues.append({"problem": "track_duration_mismatch", "track_end": cursor, "duration_frames": duration})
    seen = set()
    for cue in cues:
        key = identifier(cue["id"])
        scene_id = identifier(cue["scene_id"])
        if key in seen or scene_id not in indexed:
            raise ValueError("Duplicate cue ID or unknown scene")
        seen.add(key)
        visual, audio = frame(cue["visual_frame"]), frame(cue["audio_frame"])
        start, end = indexed[scene_id]
        if not (start <= visual < end and start <= audio < end):
            issues.append({"cue_id": key, "problem": "cue_outside_scene"})
        if abs(visual - audio) > tolerance:
            issues.append({"cue_id": key, "problem": "cue_misaligned", "delta_frames": visual - audio})
    seconds = Fraction(duration, 1) / fps
    return {"valid": not issues, "issues": issues, "scene_count": len(scenes), "cue_count": len(cues), "fps": str(fps), "duration_seconds_exact": str(seconds), "duration_seconds": float(seconds), "media_inspected": False, "cue_meaning_verified": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("timeline", type=Path)
    args = parser.parse_args()
    try:
        with args.timeline.open("rb") as handle:
            raw = handle.read(8 * 1024 * 1024 + 1)
        if len(raw) > 8 * 1024 * 1024:
            raise ValueError("Input exceeds 8 MiB")
        data = json.loads(raw.decode("utf-8-sig"), object_pairs_hook=unique_object, parse_constant=reject_constant)
        result = check(data)
        print(json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2))
        return 0 if result["valid"] else 2
    except (OSError, ValueError, KeyError, TypeError, AttributeError, ZeroDivisionError, OverflowError, RecursionError) as exc:
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
