#!/usr/bin/env python3
"""Read-only gVisor eligibility and exact Docker runtime registration checks."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from fenix_execution.gvisor import Bundle, host_requirements


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path("/etc/fenix/gvisor/manifest.json"))
    args = parser.parse_args()
    checks = {}
    try:
        host_requirements()
        checks["systrap_host"] = dict(ok=True)
    except Exception as error:
        checks["systrap_host"] = dict(ok=False, reason=str(error))
    for command in ("docker", "systemctl", "iptables", "mount", "umount"):
        checks[command] = dict(ok=shutil.which(command) is not None)
    try:
        bundle = Bundle(args.manifest)
        info = json.loads(subprocess.check_output(["docker", "info", "--format", "{{json .}}"], timeout=20))
        bundle.check_registration(info)
        if info.get("CgroupVersion") != "2" or info.get("CgroupDriver") != "systemd":
            raise ValueError("Docker requires systemd and cgroup v2")
        checks["bundle_and_runtime"] = dict(ok=True, version=bundle.data["version"])
    except Exception as error:
        checks["bundle_and_runtime"] = dict(ok=False, reason=str(error))
    eligible = all(item["ok"] for item in checks.values())
    print(json.dumps(dict(eligible=eligible, checks=checks, platform="systrap", kvm_required=False,
                         subscription_required=False, sandbox_execution_tested=False), indent=2))
    return 0 if eligible else 1


if __name__ == "__main__":
    raise SystemExit(main())
