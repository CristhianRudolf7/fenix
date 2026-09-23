#!/usr/bin/env python3
"""Provision a resource-bounded account controller with mandatory external execution."""
import argparse
import json
import os
from pathlib import Path
import subprocess
from urllib.parse import urlsplit
from fenix_isolation import (Limits, PolicyError, account_root, account_slice,
                             image_id, private_json, private_write, registry, verify_storage)
from fenix_execution.policy import Policy


def provision(args):
    config = private_json(args.execution_config)
    Policy(config)
    accounts = registry(args.registry, config.get("registry_uid"))
    account = next((a for a in accounts.values() if a["email"] == args.email.lower()), None)
    if account is None or account.get("disabled", False):
        raise PolicyError("Active account not found")
    if Path(config["registry"]).resolve() != args.registry.resolve() or Path(config["users_root"]).resolve() != args.users_root.resolve():
        raise PolicyError("Execution service and controller must use the same account registry")
    identities = config["accounts"]
    owner = identities[account["id"]]
    uid, gid = owner["uid"], owner["gid"]
    if type(uid) is not int or type(gid) is not int or min(uid, gid) < 1:
        raise PolicyError("Tenant identity must be explicitly non-root")
    if len({a["uid"] for a in identities.values()}) != len(identities):
        raise PolicyError("Every account must have a different host UID")
    root = account_root(args.users_root, account["id"])
    if root.stat().st_uid != uid or root.stat().st_gid != gid:
        raise PolicyError("Storage ownership does not match the configured account identity")
    verify_storage(root, owner["disk_bytes"])
    limits = Limits(**config.get("controller_limits", {"memory_mib": 1024, "pids": 128}))
    aggregate = Limits(**owner.get("limits", config.get("account_limits", {"memory_mib": 4096, "pids": 512})))
    parent = account_slice(account["id"], aggregate)
    image = image_id(args.image)
    if image != image_id(config["image"]):
        raise PolicyError("Controller and execution policy must use the same approved image")
    socket_dir = Path(config["socket"]).parent.resolve(strict=True)
    if socket_dir.is_relative_to(root) or socket_dir.stat().st_mode & 0o022:
        raise PolicyError("Execution socket directory must be protected and outside account storage")
    name = "fenix-user-" + account["id"]
    if subprocess.run(["docker", "network", "inspect", name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15).returncode:
        subprocess.run(["docker", "network", "create", "--opt", "com.docker.network.bridge.enable_icc=false", name], check=True, timeout=15)
    guard = ["python3", str(Path(__file__).with_name("tenant-network-guard.py")), name,
             *[part for address in config.get("host_public_ips", []) for part in ("--deny-ip", address)]]
    subprocess.run(guard if os.geteuid() == 0 else ["sudo", "-n", *guard], check=True, timeout=45)
    env = {"FENIX_UPSTREAM_TOKEN": account["upstream_token"], "FENIX_USER_ROOT": str(root),
           "FENIX_USER_ID": account["id"], "HOME": str(root / "workspace"),
           "FENIX_AUTH_EPOCH": str(account.get("auth_epoch", 0)),
           "FENIX_EXECUTION_SOCKET": str(Path(config["socket"])), "FENIX_EXECUTION_REQUIRED": "1"}
    env_path = args.registry.parent / (account["id"] + ".env")
    if env_path.is_relative_to(root):
        raise PolicyError("Controller environment file must remain outside account storage")
    private_write(env_path, "".join(f"{k}={v}\n" for k, v in env.items()).encode())
    command = ["docker", "create", "--name", name, "--restart", "no", "--init",
               "--label", "fenix.controller=" + config.get("instance", "default"), "--label", "fenix.owner=" + account["id"],
               "--read-only", "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
               "--user", f"{uid}:{gid}", "--network", name, "--cgroup-parent", parent,
               "--sysctl", "net.ipv6.conf.all.disable_ipv6=1", "--sysctl", "net.ipv6.conf.default.disable_ipv6=1",
               "--publish", f"127.0.0.1:{urlsplit(account['upstream']).port}:{account.get('runtime_port', 4173)}",
               "--tmpfs", f"/tmp:rw,nosuid,nodev,size={limits.tmp_mib}m,mode=1777",
               "--mount", f"type=bind,src={root},dst={root},bind-propagation=rprivate",
               "--mount", f"type=bind,src={socket_dir},dst={socket_dir},readonly",
               "--workdir", str(root / "workspace"), "--env-file", str(env_path),
               *limits.docker(), image, "--port", str(account.get("runtime_port", 4173)),
               "--data-dir", str(root / "state"), "--skills-dir", str(root / "skills")]
    subprocess.run(command, check=True, timeout=60)
    print(f"Created {name} using {image}. Start through fenix-controller@{account['id']}.service after the execution service is healthy.")


def main():
    if os.geteuid() != 0:
        raise PolicyError("Controller provisioning requires host root and an explicit non-root account identity")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("registry", type=Path)
    parser.add_argument("users_root", type=Path)
    parser.add_argument("email")
    parser.add_argument("--image", default="fenix-tenant:local")
    parser.add_argument("--execution-config", required=True, type=Path)
    provision(parser.parse_args())


if __name__ == "__main__":
    main()
