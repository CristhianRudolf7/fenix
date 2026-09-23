#!/usr/bin/env python3
"""Read-only host eligibility check. A pass does not replace the VM acceptance tests."""
import argparse
import json
from pathlib import Path
import shutil
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from fenix_execution.microvm import Bundle, require_kvm


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path)
    args = parser.parse_args()
    checks = {}
    try:
        require_kvm()
        checks["kvm"] = {"ok": True}
    except Exception as error:
        checks["kvm"] = {"ok": False, "reason": str(error)}
    checks["cgroup_v2"] = {"ok": Path("/sys/fs/cgroup/cgroup.controllers").is_file()}
    for command in ("docker", "systemctl", "iptables", "mount", "umount"):
        checks[command] = {"ok": shutil.which(command) is not None}
    if args.manifest:
        try:
            Bundle(args.manifest)
            checks["bundle"] = {"ok": True}
        except Exception as error:
            checks["bundle"] = {"ok": False, "reason": str(error)}
    result = dict(eligible=all(item["ok"] for item in checks.values()), checks=checks,
                  guest_boot_tested=False, subscription_required=False)
    print(json.dumps(result, indent=2))
    return 0 if result["eligible"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
