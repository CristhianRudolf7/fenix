#!/usr/bin/env python3
"""Prepare a pinned, self-hosted Kata/Cloud Hypervisor bundle for Fenix.

Requires a separately installed, verified Kata release. Writes configuration and
a Docker configuration fragment; never restarts Docker or migrates user data.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
sys.path.insert(0, str(Path(__file__).resolve().parent))
from fenix_isolation import PolicyError, private_write
from fenix_execution.microvm import Bundle, digest, protected_path, validate_configuration, wrapper_text


def harden(data, memory, cpus):
    data = copy.deepcopy(data)
    if set(data.get("hypervisor", {})) != {"clh"}:
        raise PolicyError("Start with the installed configuration-clh.toml")
    hv = data["hypervisor"]["clh"]
    hv.update(shared_fs="virtio-fs", virtio_fs_cache="never", disable_block_device_use=True,
              disable_seccomp=False, enable_debug=False, enable_annotations=[],
              disable_nested_virtualization=True, guest_hook_path="", virtio_fs_extra_args=[],
              default_memory=256, default_maxmemory=memory, default_vcpus=1, default_maxvcpus=cpus)
    data.setdefault("runtime", {}).update(sandbox_cgroup_only=True, disable_guest_seccomp=False,
        disable_new_netns=False, internetworking_model="tcfilter", sandbox_bind_mounts=[],
        enable_debug=False, enable_tracing=False, enable_pprof=False, experimental=[],
        static_sandbox_resource_mgmt=True)
    data.setdefault("agent", {}).setdefault("kata", {}).update(enable_debug=False, enable_tracing=False, debug_console_enabled=False)
    data.setdefault("factory", {}).update(enable_template=False, vm_cache_number=0)
    for key in ("path", "kernel", "image", "virtio_fs_daemon"):
        # Distribution symlinks are resolved once, then the concrete asset is pinned.
        hv[key] = str(protected_path(Path(hv[key]).resolve(strict=True)))
    validate_configuration(data)
    return data


def toml(data, prefix=""):
    lines = []
    for key, value in data.items():
        if isinstance(value, dict):
            continue
        if not isinstance(value, (str, bool, int, float, list)):
            raise PolicyError("Unsupported upstream configuration value")
        lines.append(json.dumps(key) + " = " + json.dumps(value))
    for key, value in data.items():
        if isinstance(value, dict):
            table = (prefix + "." if prefix else "") + json.dumps(key)
            lines.extend(["", "[" + table + "]", toml(value, table)])
    return "\n".join(lines) + "\n"


def merge_runtime(existing, wrapper):
    if not isinstance(existing, dict) or not isinstance(existing.get("runtimes", {}), dict):
        raise PolicyError("Invalid Docker daemon configuration")
    result = copy.deepcopy(existing)
    runtimes = result.setdefault("runtimes", {})
    expected = {"runtimeType": str(wrapper)}
    if "fenix-kata" in runtimes and runtimes["fenix-kata"] != expected:
        raise PolicyError("Refusing to replace a different fenix-kata runtime")
    runtimes["fenix-kata"] = expected
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kata-root", type=Path, default=Path("/opt/kata"))
    parser.add_argument("--output", type=Path, default=Path("/etc/fenix/microvm"))
    parser.add_argument("--gate", type=Path, default=Path("/opt/fenix/isolation/microvm-gate.py"))
    parser.add_argument("--max-memory-mib", type=int, default=8192)
    parser.add_argument("--max-cpus", type=int, default=8)
    parser.add_argument("--register-runtime", action="store_true", help="Merge the runtime into Docker configuration after validation; do not reload Docker")
    parser.add_argument("--docker-config", type=Path, default=Path("/etc/docker/daemon.json"))
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise PolicyError("Run bundle preparation as the host operator (root)")
    if not 512 <= args.max_memory_mib <= 131072 or not 1 <= args.max_cpus <= 64:
        raise PolicyError("Invalid maximum VM resources")
    if subprocess.run(["systemctl", "is-active", "--quiet", "fenix-execution.service"], check=False).returncode == 0:
        raise PolicyError("Stop the execution service and its workloads before changing the VM runtime")
    active = subprocess.check_output(["docker", "ps", "--filter", "label=fenix.isolation=microvm", "--format", "{{.ID}}"], timeout=20)
    if active.strip():
        raise PolicyError("Stop active Fenix microVM workloads before preparing a runtime bundle")
    os.umask(0o077)
    # Current Kata accepts KATA_CONF_FILE only at a compiled default location.
    # Install a regular configuration.toml there rather than pointing the wrapper
    # at configuration-clh.toml (which can be rejected by newer shim versions).
    source_config = protected_path(args.kata_root / "share/defaults/kata-containers/configuration-clh.toml")
    config = source_config.with_name("configuration.toml")
    shim = protected_path((args.kata_root / "bin/containerd-shim-kata-v2").resolve(strict=True))
    gate = protected_path(args.gate.resolve(strict=True))
    with source_config.open("rb") as source:
        data = harden(tomllib.load(source), args.max_memory_mib, args.max_cpus)
    encoded_config = toml(data)
    validate_configuration(tomllib.loads(encoded_config))
    args.output.mkdir(mode=0o700, parents=True, exist_ok=True)
    if args.output.is_symlink() or args.output.stat().st_uid != 0 or args.output.stat().st_mode & 0o077:
        raise PolicyError("Bundle output must be a private root-owned directory")
    manifest = args.output / "manifest.json"
    if manifest.exists():
        raise PolicyError("Bundle already exists; stop workloads and prepare an explicit upgrade")
    backup = config.with_name("configuration.before-fenix.toml")
    if not backup.exists():
        with backup.open("xb") as target:
            target.write(protected_path(config.resolve(strict=True)).read_bytes() if config.exists() else source_config.read_bytes())
    private_write(config, encoded_config.encode())
    wrapper = args.output / "containerd-shim-fenix-v2"
    private_write(wrapper, wrapper_text(shim, config).encode())
    wrapper.chmod(0o700)
    assets = [config, shim, wrapper, gate] + [Path(data["hypervisor"]["clh"][k]) for k in ("path", "kernel", "image", "virtio_fs_daemon")]
    value = dict(schema=1, runtime="fenix-kata", configuration=str(config), shim=str(shim),
                 wrapper=str(wrapper), gate=str(gate), assets={str(path): digest(path) for path in assets})
    private_write(manifest, json.dumps(value, indent=2).encode())
    Bundle(manifest)
    fragment = {"runtimes": {"fenix-kata": {"runtimeType": str(wrapper)}}}
    private_write(args.output / "docker.fragment.json", json.dumps(fragment, indent=2).encode())
    if args.register_runtime:
        old = {}
        if args.docker_config.exists():
            old = json.loads(protected_path(args.docker_config).read_text())
        else:
            if not args.docker_config.parent.is_dir() or args.docker_config.parent.stat().st_uid != 0 or args.docker_config.parent.stat().st_mode & 0o022:
                raise PolicyError("Docker configuration parent must be protected")
        merged = json.dumps(merge_runtime(old, wrapper), indent=2).encode()
        with tempfile.TemporaryDirectory(prefix="validate-", dir=args.output) as temporary:
            candidate = Path(temporary) / "daemon.json"
            private_write(candidate, merged)
            subprocess.run(["dockerd", "--validate", "--config-file", str(candidate)], check=True, timeout=20)
        if args.docker_config.exists():
            private_write(args.output / "docker.before-fenix.json", args.docker_config.read_bytes())
        private_write(args.docker_config, merged)
    print(json.dumps(dict(manifest=str(manifest), docker_fragment=str(args.output / "docker.fragment.json"),
                         docker_reload_required=args.register_runtime, activated=False, subscription_required=False), indent=2))


if __name__ == "__main__":
    main()
