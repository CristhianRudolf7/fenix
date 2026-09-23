"""Self-hosted gVisor isolation using systrap, without KVM or hosted services."""
from __future__ import annotations
import asyncio
import copy
import dataclasses
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
from fenix_isolation import PolicyError, identifier, private_json, private_write
from .docker import Docker
from .runtime_assets import digest, protected_path, stamp

RUNTIME = "fenix-runsc"
GATE_PATH = "/opt/fenix/sandbox-gate.py"
GATE_PREFIX = b"FENIX-SANDBOX/1 "
# Pin every security-relevant choice rather than depending on Docker defaults.
# Bind mounts are shared with native file tools; no private overlay may hide writes.
RUNSC_ARGS = [
    "--platform=systrap", "--network=sandbox", "--directfs=false",
    "--file-access-mounts=shared", "--overlay2=none",
    "--host-uds=none", "--host-fifo=none", "--net-raw=false",
    "--allow-packet-socket-write=false", "--gvisor-marker-file=true",
    "--ignore-cgroups=false", "--systemd-cgroup=true", "--rootless=false",
    "--oci-seccomp=true", "--allow-flag-override=false", "--allow-suid=false",
    "--debug=false", "--strace=false", "--profile=false",
    "--sidecar-usage-policy=STRICT", "--sidecar-release-enforcement-policy=ALWAYS",
    "--gofer-network-namespace=new", "--character-device-policy=emulated-only",
]


def host_requirements():
    version = re.match(r"(\d+)\.(\d+)", platform.release())
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "aarch64") or not version or tuple(map(int, version.groups())) < (5, 6):
        raise PolicyError("gVisor systrap requires Linux 5.6+ on x86_64 or aarch64")
    if not Path("/sys/fs/cgroup/cgroup.controllers").is_file():
        raise PolicyError("Fenix gVisor execution requires cgroup v2")


def runtime_entry(runsc):
    return {"path": str(runsc), "runtimeArgs": list(RUNSC_ARGS)}


def merge_runtime(existing, runsc):
    if not isinstance(existing, dict) or not isinstance(existing.get("runtimes", {}), dict):
        raise PolicyError("Invalid Docker daemon configuration")
    result = copy.deepcopy(existing)
    runtimes = result.setdefault("runtimes", {})
    expected = runtime_entry(runsc)
    if RUNTIME in runtimes and runtimes[RUNTIME] != expected:
        raise PolicyError("Refusing to replace a different fenix-runsc runtime")
    runtimes[RUNTIME] = expected
    return result


def sidecar_files(runsc, owner=0):
    directory = Path(runsc).parent / "gvisor-bin"
    info = directory.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != owner or info.st_mode & 0o022:
        raise PolicyError("gVisor sidecars must be installed in a protected directory")
    files = sorted(directory.iterdir())
    if not 1 <= len(files) <= 32:
        raise PolicyError("Incomplete or excessive gVisor sidecar installation")
    for path in files:
        protected_path(path, owner)
        if not os.access(path, os.X_OK):
            raise PolicyError("gVisor sidecar is not executable")
    if directory / "gvisor_sentry" not in files:
        raise PolicyError("Install the complete gVisor release, including gvisor_sentry")
    return files


class Bundle:
    def __init__(self, manifest, owner=0):
        self.path = protected_path(manifest, owner)
        self.data = private_json(self.path, owner)
        if self.data.get("schema") != 1 or self.data.get("runtime") != RUNTIME or self.data.get("platform") != "systrap":
            raise PolicyError("Invalid gVisor systrap bundle")
        self.runsc = protected_path(self.data["runsc"], owner)
        self.gate = protected_path(self.data["gate"], owner)
        if self.data.get("runtime_args") != RUNSC_ARGS:
            raise PolicyError("gVisor runtime arguments do not match the isolation policy")
        sidecars = sidecar_files(self.runsc, owner)
        self.sentry = self.runsc.parent / "gvisor-bin/gvisor_sentry"
        required = {str(p) for p in (self.runsc, self.gate, *sidecars)}
        assets = self.data.get("assets")
        if not isinstance(assets, dict) or set(assets) != required:
            raise PolicyError("gVisor manifest must pin the runtime, gate and every sidecar")
        if not os.access(self.runsc, os.X_OK):
            raise PolicyError("gVisor runtime is not executable")
        for path, checksum in assets.items():
            if not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum) or digest(path) != checksum:
                raise PolicyError("gVisor runtime asset checksum mismatch")
        self.stamps = {Path(p): stamp(p) for p in (*assets, self.path, self.sentry.parent)}

    def unchanged(self):
        for path, previous in self.stamps.items():
            if stamp(path) != previous:
                raise PolicyError("gVisor assets changed; stop workloads and validate the upgrade")
        # Some filesystems give successive same-size writes identical timestamps.
        # Metadata equality is not evidence that a pinned executable is unchanged.
        for path, checksum in self.data["assets"].items():
            if digest(path) != checksum:
                raise PolicyError("gVisor runtime asset checksum changed; validate the upgrade")

    def check_registration(self, info):
        actual = info.get("Runtimes", {}).get(RUNTIME, {})
        if actual.get("path") != str(self.runsc) or actual.get("runtimeArgs", []) != RUNSC_ARGS or actual.get("runtimeType") or actual.get("options"):
            raise PolicyError("Docker must register the exact pinned gVisor systrap runtime; no fallback is permitted")


def gate_report(line):
    if len(line) > 4096 or not line.startswith(GATE_PREFIX) or not line.endswith(b"\n"):
        raise PolicyError("gVisor startup verification failed")
    try:
        data = json.loads(line[len(GATE_PREFIX):])
        if not isinstance(data, dict) or data.get("backend") != "gvisor" or not isinstance(data.get("kernel"), str):
            raise ValueError()
    except (ValueError, TypeError) as error:
        raise PolicyError("Invalid gVisor startup report") from error
    return data


def sentry_cgroup(owner, container, pid, sentry, proc=Path("/proc")):
    """Verify the host Sentry, not a PID or kernel string supplied by user code."""
    if type(pid) is not int or pid <= 0 or not re.fullmatch(r"[0-9a-f]{64}", container):
        raise PolicyError("Invalid gVisor sandbox identity")
    process = proc / str(pid)
    if (process / "exe").resolve() != Path(sentry):
        raise PolicyError("Container PID is not the pinned gVisor Sentry")
    argv = (process / "cmdline").read_bytes().decode().split("\0")
    for flag in ("--platform=systrap", "--network=sandbox", "--directfs=false", "--ignore-cgroups=false"):
        key = flag.split("=", 1)[0] + "="
        if [arg for arg in argv if arg.startswith(key)] != [flag]:
            raise PolicyError("Sentry is not running with the required systrap isolation flags")
    expected = "/fenixacct.slice/fenixacct-" + identifier(owner).replace("-", "") + ".slice/docker-" + container + ".scope"
    groups = [line[3:] for line in (process / "cgroup").read_text().splitlines() if line.startswith("0::")]
    if groups != [expected]:
        raise PolicyError("gVisor Sentry is outside its account resource cgroup")
    status = dict(line.split(":", 1) for line in (process / "status").read_text().splitlines() if ":" in line)
    if status.get("Seccomp", "").strip() != "2" or status.get("NoNewPrivs", "").strip() != "1":
        raise PolicyError("gVisor Sentry host seccomp protection is unavailable")
    return expected


class GVisor(Docker):
    kind = "gvisor"
    platform = "systrap"
    default_runtime = RUNTIME

    def __init__(self, config):
        host_requirements()
        if config.get("runtime", RUNTIME) != RUNTIME:
            raise PolicyError("gVisor execution requires the fenix-runsc runtime")
        if not config.get("gvisor_manifest"):
            raise PolicyError("Prepare a pinned gVisor bundle before enabling execution")
        self.bundle = Bundle(config["gvisor_manifest"])
        self.verified = set()
        super().__init__(config)
        self.bundle.check_registration(json.loads(subprocess.check_output(["docker", "info", "--format", "{{json .}}"], timeout=20)))

    def browser_security_args(self):
        # OCI seccomp is explicitly enabled inside gVisor. Its Sentry has its own
        # host filter; the runc AppArmor browser profile does not describe Sentry.
        return ["--security-opt", "seccomp=" + str(protected_path(self.config["browser_seccomp"]))]

    def create_args(self, execution, launch, parent, sources=None):
        self.bundle.unchanged()
        gated = dataclasses.replace(launch, argv=["/usr/bin/python3", "-I", GATE_PATH, *launch.argv])
        args = super().create_args(execution, gated, parent, sources)
        at = args.index("--workdir")
        args[at:at] = ["--mount", "type=bind,src=" + str(self.bundle.gate) + ",dst=" + GATE_PATH + ",readonly",
                       "--label", "fenix.isolation=gvisor"]
        return args

    async def attach(self, execution):
        self.bundle.check_registration(json.loads(await self.command("docker", "info", "--format", "{{json .}}")))
        process = await super().attach(execution)
        try:
            gate_report((await asyncio.wait_for(process.stdout.readline(), 60)).replace(b"\r\n", b"\n"))
            info = await self.inspect(execution)
            if info["HostConfig"]["Runtime"] != RUNTIME:
                raise PolicyError("Docker selected a different isolation runtime")
            metadata = self.mount_dir / execution / "container.json"
            record = private_json(metadata)
            group = await asyncio.to_thread(sentry_cgroup, record["owner"], record["container"], info["State"]["Pid"], self.bundle.sentry)
            record.update(sandbox_cgroup=group)
            private_write(metadata, json.dumps(record).encode())
            self.bundle.unchanged()
            self.verified.add(execution)
            return process
        except BaseException:
            if process.returncode is None:
                process.kill()
            await process.wait()
            raise

    async def release(self, execution, process):
        self.bundle.unchanged()
        if execution not in self.verified or process.returncode is not None:
            raise PolicyError("gVisor startup was not verified")
        process.stdin.write(b"FENIX-START\n")
        await asyncio.wait_for(process.stdin.drain(), 5)

    def exec_argv(self, argv):
        return ["/usr/bin/python3", "-I", GATE_PATH, "--exec", *argv]

    async def guard_check(self, execution, profile, repair=False):
        self.bundle.unchanged()
        await super().guard_check(execution, profile, repair)

    async def destroy(self, execution):
        await super().destroy(execution)
        self.verified.discard(execution)
