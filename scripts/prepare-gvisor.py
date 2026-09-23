#!/usr/bin/env python3
"""Pin an installed gVisor release and prepare its systrap Docker registration.

No runtime downloads, service restarts, default-runtime changes or account
migrations are performed. Install a checksum-verified complete release first.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.path.insert(0, str(Path(__file__).resolve().parent))
from fenix_isolation import PolicyError, private_write
from fenix_execution.gvisor import Bundle, RUNSC_ARGS, RUNTIME, host_requirements, merge_runtime, runtime_entry, sidecar_files
from fenix_execution.runtime_assets import digest, protected_path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runsc", type=Path, default=Path("/usr/local/bin/runsc"))
    parser.add_argument("--gate", type=Path, default=Path("/opt/fenix/isolation/sandbox-gate.py"))
    parser.add_argument("--output", type=Path, default=Path("/etc/fenix/gvisor"))
    parser.add_argument("--register-runtime", action="store_true")
    parser.add_argument("--docker-config", type=Path, default=Path("/etc/docker/daemon.json"))
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise PolicyError("Run preparation as the host operator (root)")
    host_requirements()
    if subprocess.run(["systemctl", "is-active", "--quiet", "fenix-execution.service"], check=False).returncode == 0:
        raise PolicyError("Stop the execution service and its workloads before changing the isolation runtime")
    if subprocess.check_output(["docker", "ps", "--filter", "label=fenix.isolation=gvisor", "--format", "{{.ID}}"], timeout=20).strip():
        raise PolicyError("Stop active Fenix gVisor workloads before preparing a bundle")
    runsc = protected_path(args.runsc)
    gate = protected_path(args.gate)
    sidecars = sidecar_files(runsc)
    # Exercise flag parsing before writing configuration. Never use runsc install,
    # whose older versions can download missing sidecars or change Docker defaults.
    version = subprocess.check_output([str(runsc), *RUNSC_ARGS, "--version"], text=True, timeout=20).strip()
    if not version.startswith("runsc version "):
        raise PolicyError("The installed executable did not identify itself as runsc")
    os.umask(0o077)
    args.output.mkdir(parents=True, mode=0o700, exist_ok=True)
    info = args.output.lstat()
    if args.output.is_symlink() or info.st_uid != 0 or info.st_mode & 0o077:
        raise PolicyError("Bundle output must be a private root-owned directory")
    manifest = args.output / "manifest.json"
    if manifest.exists():
        raise PolicyError("Bundle already exists; stop workloads and prepare an explicit upgrade")
    assets = [runsc, gate, *sidecars]
    value = dict(schema=1, runtime=RUNTIME, platform="systrap", runsc=str(runsc), gate=str(gate),
                 runtime_args=RUNSC_ARGS, version=version, assets={str(p): digest(p) for p in assets})
    private_write(manifest, json.dumps(value, indent=2).encode())
    Bundle(manifest)
    fragment = {"runtimes": {RUNTIME: runtime_entry(runsc)}}
    private_write(args.output / "docker.fragment.json", json.dumps(fragment, indent=2).encode())
    if args.register_runtime:
        if args.docker_config.exists():
            old = json.loads(protected_path(args.docker_config).read_text())
        else:
            parent = args.docker_config.parent.lstat()
            if args.docker_config.parent.is_symlink() or parent.st_uid != 0 or parent.st_mode & 0o022:
                raise PolicyError("Docker configuration parent must be protected")
            old = {}
        merged = json.dumps(merge_runtime(old, runsc), indent=2).encode()
        with tempfile.TemporaryDirectory(prefix="validate-", dir=args.output) as temporary:
            candidate = Path(temporary) / "daemon.json"
            private_write(candidate, merged)
            subprocess.run(["dockerd", "--validate", "--config-file", str(candidate)], check=True, timeout=20)
        if args.docker_config.exists():
            private_write(args.output / "docker.before-fenix.json", args.docker_config.read_bytes())
        private_write(args.docker_config, merged)
    print(json.dumps(dict(manifest=str(manifest), docker_fragment=str(args.output / "docker.fragment.json"),
                         docker_reload_required=args.register_runtime, activated=False,
                         platform="systrap", kvm_required=False, subscription_required=False), indent=2))


if __name__ == "__main__":
    main()
