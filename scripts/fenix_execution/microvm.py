"""Self-hosted VM execution through Kata's maintained containerd shim.

Cloud Hypervisor supplies the hardware boundary; Kata owns guest processes and
virtio-fs exports. No hosted API, E2B credential, subscription or host fallback.
The Fenix service still owns admission, authentication, network and data scope.
"""
from __future__ import annotations
import asyncio
import dataclasses
import fcntl
import json
import os
from pathlib import Path
import re
import shlex
import stat
import tomllib
from fenix_isolation import PolicyError, identifier, private_json, private_write
from .docker import Docker
from .runtime_assets import digest, protected_path

GATE_PATH = "/opt/fenix/microvm-gate.py"
GATE_PREFIX = b"FENIX-MICROVM/1 "


def wrapper_text(shim, configuration):
    return ("#!/bin/sh\nset -eu\n"
            "export KATA_CONF_FILE=" + shlex.quote(str(configuration)) + "\n"
            "exec " + shlex.quote(str(shim)) + ' "$@"\n')


def validate_configuration(data):
    if set(data.get("hypervisor", {})) != {"clh"}:
        raise PolicyError("This backend requires the Cloud Hypervisor Kata configuration")
    sections = {
        "hypervisor": (data["hypervisor"]["clh"], {
            "shared_fs": "virtio-fs", "virtio_fs_cache": "never",
            "disable_block_device_use": True, "disable_seccomp": False,
            "enable_debug": False, "enable_annotations": [],
            "disable_nested_virtualization": True, "guest_hook_path": "",
        }),
        "runtime": (data.get("runtime", {}), {
            "sandbox_cgroup_only": True, "disable_guest_seccomp": False,
            "disable_new_netns": False, "internetworking_model": "tcfilter",
            "sandbox_bind_mounts": [], "enable_debug": False,
            "enable_tracing": False, "enable_pprof": False, "experimental": [],
            "static_sandbox_resource_mgmt": True,
        }),
        "agent": (data.get("agent", {}).get("kata", {}), {
            "enable_debug": False, "enable_tracing": False, "debug_console_enabled": False,
        }),
    }
    for name, (actual, expected) in sections.items():
        for key, value in expected.items():
            if type(actual.get(key)) is not type(value) or actual[key] != value:
                raise PolicyError(f"Unsafe or missing Kata setting: {name}.{key}")
    if data.get("factory", {}).get("enable_template", False) or data.get("factory", {}).get("vm_cache_number", 0):
        raise PolicyError("Shared VM templates and VM caches are disabled")
    hv = data["hypervisor"]["clh"]
    if hv.get("virtio_fs_extra_args", []) != []:
        raise PolicyError("Custom virtiofsd arguments require a new reviewed policy")
    for key in ("default_memory", "default_maxmemory", "default_vcpus", "default_maxvcpus"):
        if type(hv.get(key)) is not int or hv[key] <= 0:
            raise PolicyError("Kata VM sizes must be explicitly bounded")
    if hv["default_memory"] > hv["default_maxmemory"] or hv["default_vcpus"] > hv["default_maxvcpus"]:
        raise PolicyError("Kata defaults exceed the VM maximum")
    return hv


class Bundle:
    def __init__(self, manifest, owner=0):
        self.path = protected_path(manifest, owner)
        self.data = private_json(self.path, owner)
        if self.data.get("schema") != 1 or self.data.get("runtime") != "fenix-kata":
            raise PolicyError("Invalid Fenix microVM bundle manifest")
        assets = self.data.get("assets")
        if not isinstance(assets, dict) or not 7 <= len(assets) <= 32:
            raise PolicyError("Invalid runtime asset manifest")
        self.stamps = {}
        for name, checksum in assets.items():
            path = protected_path(name, owner)
            if not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum) or digest(path) != checksum:
                raise PolicyError("MicroVM runtime asset checksum mismatch")
            self.stamps[path] = path.stat()
        config = Path(self.data["configuration"])
        if str(config) not in assets:
            raise PolicyError("Runtime manifest omits its configuration")
        with open(config, "rb") as source:
            self.hv = validate_configuration(tomllib.load(source))
        required = [self.data[k] for k in ("configuration", "shim", "wrapper", "gate")]
        required += [self.hv[k] for k in ("path", "kernel", "image", "virtio_fs_daemon")]
        if any(name not in assets for name in required):
            raise PolicyError("Runtime manifest omits a required asset")
        for name in (self.data["shim"], self.data["wrapper"], self.hv["path"], self.hv["virtio_fs_daemon"]):
            if not os.access(name, os.X_OK):
                raise PolicyError("Runtime binary is not executable")
        if Path(self.data["wrapper"]).read_text() != wrapper_text(self.data["shim"], config):
            raise PolicyError("Runtime wrapper does not select the pinned Kata configuration")
        self.stamps[self.path] = self.path.stat()

    def unchanged(self):
        # Assets are operator-owned. A deployment change requires restarting and
        # validating the entire bundle, never silently approving new bytes.
        for path, previous in self.stamps.items():
            now = path.lstat()
            if (now.st_dev, now.st_ino, now.st_mode, now.st_uid, now.st_size, now.st_mtime_ns, now.st_ctime_ns) != (
                    previous.st_dev, previous.st_ino, previous.st_mode, previous.st_uid, previous.st_size, previous.st_mtime_ns, previous.st_ctime_ns):
                raise PolicyError("MicroVM assets changed while the service was running")


def require_kvm():
    try:
        fd = os.open("/dev/kvm", os.O_RDWR | os.O_CLOEXEC | os.O_NOFOLLOW)
        try:
            if not stat.S_ISCHR(os.fstat(fd).st_mode) or fcntl.ioctl(fd, 0xAE00, 0) != 12:
                raise PolicyError("Unsupported KVM API")
        finally:
            os.close(fd)
    except OSError as error:
        raise PolicyError("MicroVM execution requires usable /dev/kvm; host/container fallback is disabled") from error


def guest_identity(line, host_boot):
    if len(line) > 4096 or not line.startswith(GATE_PREFIX) or not line.endswith(b"\n"):
        raise PolicyError("MicroVM boot verification failed")
    data = json.loads(line[len(GATE_PREFIX):])
    boot = identifier(data["boot_id"])
    if boot == host_boot or not isinstance(data.get("kernel"), str):
        raise PolicyError("The workload did not start in a separate guest kernel")
    return boot


def vm_cgroup(owner, container, hypervisor, proc=Path("/proc")):
    """Find the actual VMM, not a guest PID reported by Docker/Kata."""
    prefix = "/fenixacct.slice/fenixacct-" + identifier(owner).replace("-", "") + ".slice/"
    matches = []
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            if (entry / "exe").resolve() != Path(hypervisor):
                continue
            argv = (entry / "cmdline").read_bytes().decode().split("\0")
            if not any(container in arg.split("/") for arg in argv):
                continue
            group = next(line[3:] for line in (entry / "cgroup").read_text().splitlines() if line.startswith("0::"))
            if not group.startswith(prefix) or ".." in Path(group).parts or container not in group:
                raise PolicyError("MicroVM is outside its account cgroup")
            matches.append(group)
        except (FileNotFoundError, ProcessLookupError, StopIteration):
            continue
    if len(matches) != 1:
        raise PolicyError("Cannot identify the account's isolated VMM cgroup")
    return matches[0]


class MicroVM(Docker):
    kind = "microvm"

    def __init__(self, config):
        require_kvm()
        self.bundle = Bundle(config["microvm_manifest"])
        if config.get("runtime") != self.bundle.data["runtime"]:
            raise PolicyError("MicroVM mode requires the pinned fenix-kata runtime")
        overhead = config.get("microvm_overhead_mib", 256)
        if type(overhead) is not int or overhead < self.bundle.hv["default_memory"]:
            raise PolicyError("VM memory overhead exceeds the admission reservation")
        limits = config.get("runner_limits", {})
        if limits.get("memory_mib", 2048) + overhead > self.bundle.hv["default_maxmemory"] or limits.get("cpus", 2) > self.bundle.hv["default_maxvcpus"]:
            raise PolicyError("Runner limits exceed the installed VM configuration")
        self.host_boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        self.boots = {}
        super().__init__(config)
        # Docker's alias is operator configuration; the guest gate and actual VMM
        # identity additionally verify what it launched before any user command.

    def browser_security_args(self):
        path = protected_path(self.config["browser_seccomp"])
        # Kata applies seccomp in the guest. The host-only AppArmor profile for
        # runc is not a guest profile; retain Chromium's namespace sandbox there.
        return ["--security-opt", "seccomp=" + str(path)]

    def create_args(self, execution, launch, parent, sources=None):
        self.bundle.unchanged()
        gated = dataclasses.replace(launch, argv=["/usr/bin/python3", "-I", GATE_PATH, *launch.argv])
        args = super().create_args(execution, gated, parent, sources)
        at = args.index("--workdir")
        args[at:at] = ["--mount", "type=bind,src=" + self.bundle.data["gate"] + ",dst=" + GATE_PATH + ",readonly",
                       "--label", "fenix.isolation=microvm"]
        return args

    async def attach(self, execution):
        process = await super().attach(execution)
        try:
            line = await asyncio.wait_for(process.stdout.readline(), 60)
            # A terminal may translate LF to CRLF before the gate configures it.
            boot = guest_identity(line.replace(b"\r\n", b"\n"), self.host_boot)
            if boot in self.boots.values():
                raise PolicyError("Executions unexpectedly share a guest kernel")
            info = await self.inspect(execution)
            if info["HostConfig"]["Runtime"] != self.runtime:
                raise PolicyError("Docker used an unexpected runtime")
            metadata = self.mount_dir / execution / "container.json"
            record = private_json(metadata)
            group = await asyncio.to_thread(vm_cgroup, record["owner"], record["container"], self.bundle.hv["path"])
            record.update(vm_cgroup=group, guest_boot_id=boot)
            private_write(metadata, json.dumps(record).encode())
            self.bundle.unchanged()
            if boot in self.boots.values():
                raise PolicyError("Executions unexpectedly share a guest kernel")
            self.boots[execution] = boot
            return process
        except BaseException:
            if process.returncode is None:
                process.kill()
            await process.wait()
            # The service owns complete sandbox cleanup, including creation failures.
            raise

    async def release(self, execution, process):
        self.bundle.unchanged()
        if execution not in self.boots or process.returncode is not None:
            raise PolicyError("MicroVM startup was not verified")
        process.stdin.write(b"FENIX-START\n")
        await asyncio.wait_for(process.stdin.drain(), 5)

    async def guard_check(self, execution, profile, repair=False):
        self.bundle.unchanged()
        await super().guard_check(execution, profile, repair)

    async def destroy(self, execution):
        await super().destroy(execution)
        self.boots.pop(execution, None)


