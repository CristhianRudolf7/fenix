#!/usr/bin/env python3
"""Resolve a local wall time without silently choosing a DST fold or gap."""

import argparse
from datetime import datetime, timezone
import json
import sys
from zoneinfo import ZoneInfo


def candidates(local_time, zone_name):
    if "T" not in local_time:
        raise ValueError("Use an ISO local date and time separated by T")
    wall = datetime.fromisoformat(local_time)
    if wall.tzinfo is not None:
        raise ValueError("Supply a local time without an offset and an IANA zone")
    zone = ZoneInfo(zone_name)
    found = {}
    for fold in (0, 1):
        local = wall.replace(tzinfo=zone, fold=fold)
        utc = local.astimezone(timezone.utc)
        back = utc.astimezone(zone)
        if back.replace(tzinfo=None) == wall:
            found[utc.isoformat()] = {"utc": utc.isoformat(), "local": back.isoformat(), "fold": back.fold}
    rows = [found[key] for key in sorted(found)]
    return {"zone": zone_name, "wall_time": wall.isoformat(),
            "status": {0: "nonexistent", 1: "unique", 2: "ambiguous"}[len(rows)], "candidates": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("local_time")
    parser.add_argument("zone")
    args = parser.parse_args()
    try:
        result = candidates(args.local_time, args.zone)
    except (ValueError, KeyError, OverflowError) as error:
        print(json.dumps({"error": str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "unique" else 2


if __name__ == "__main__":
    sys.exit(main())
