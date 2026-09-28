#!/usr/bin/env python3
"""Verify a controller identity, volume and execution policy before every start."""
import argparse
import json
from pathlib import Path
import socket
import subprocess
from urllib.parse import urlsplit
from fenix_isolation import Limits, PolicyError, identifier, private_json, image_id
from fenix_execution.policy import Policy


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("config",type=Path)
    parser.add_argument("account")
    args=parser.parse_args()
    config=private_json(args.config)
    policy=Policy(config)
    owner=identifier(args.account)
    account=policy.current().get(owner)
    if not account or account.get("disabled",False):
        raise PolicyError("Account is disabled or missing")
    policy.storage(owner)
    subprocess.run(["python3",str(Path(__file__).with_name("tenant-network-guard.py")),"fenix-user-"+owner,
                    *[part for address in config.get("host_public_ips",[]) for part in ("--deny-ip",address)]],check=True,timeout=60)
    with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as connection:
        connection.settimeout(10)
        connection.connect(config["socket"])
        connection.sendall(json.dumps(dict(operation="health",account_id=owner,credential=account["upstream_token"],auth_epoch=account.get("auth_epoch",0))).encode()+b"\n")
        with connection.makefile("rb") as source:
            response=json.loads(source.readline(65536))
        if response.get("ok") is not True:
            raise PolicyError("Execution service rejected controller startup")
        kind = config.get("backend", "gvisor")
        runtime = config.get("runtime", "fenix-runsc" if kind == "gvisor" else "runc")
        if response.get("backend") != kind or response.get("runtime") != runtime:
            raise PolicyError("Execution service does not provide the configured isolation backend")
    info=json.loads(subprocess.check_output(["docker","inspect","fenix-user-"+owner],timeout=15))[0]
    if info["Config"]["User"] != f"{policy.accounts[owner]['uid']}:{policy.accounts[owner]['gid']}":
        raise PolicyError("Controller UID differs from execution policy")
    env=dict(value.split("=",1) for value in info["Config"].get("Env",[]) if "=" in value)
    if env.get("FENIX_UPSTREAM_TOKEN") != account["upstream_token"] or env.get("FENIX_EXECUTION_REQUIRED") != "1":
        raise PolicyError("Controller credentials or isolation configuration changed; reprovision it")
    if env.get("FENIX_AUTH_EPOCH") != str(account.get("auth_epoch",0)):
        raise PolicyError("Account authorization revision changed; reprovision the controller")
    if env.get("FENIX_USER_ID") != owner or env.get("FENIX_USER_ROOT") != str(policy.users_root/owner) or env.get("FENIX_EXECUTION_SOCKET") != config["socket"]:
        raise PolicyError("Controller execution transport or account binding differs from policy")
    host=info["HostConfig"]
    limits=Limits(**config.get("controller_limits", {"memory_mib":1024,"pids":128}))
    if (host.get("Privileged") or not host.get("ReadonlyRootfs") or host.get("PidMode")
            or host.get("IpcMode") != "private" or host.get("NetworkMode") != "fenix-user-"+owner
            or host.get("CapDrop") != ["ALL"] or "no-new-privileges" not in host.get("SecurityOpt",[]) or host.get("CapAdd")
            or host.get("Memory") != limits.memory_mib*1024*1024 or host.get("MemorySwap") != host.get("Memory")
            or host.get("PidsLimit") != limits.pids or host.get("NanoCpus") != int(limits.cpus*1000000000)):
        raise PolicyError("Controller isolation or resource limits differ from the approved policy")
    expected={str(policy.users_root/owner):(str(policy.users_root/owner),True),str(Path(config["socket"]).parent):(str(Path(config["socket"]).parent),False)}
    if config.get("skill_categories"):
        catalog = Path(config["skill_categories"]).resolve(strict=True)
        metadata = catalog.stat()
        if not catalog.is_file() or metadata.st_uid != 0 or metadata.st_mode & 0o022 or catalog.is_relative_to(policy.users_root):
            raise PolicyError("Skill category catalog must be an operator-owned read-only file")
        if env.get("FENIX_SKILL_CATEGORIES") != "/etc/fenix/skill-categories.json":
            raise PolicyError("Controller skill category configuration differs from policy")
        expected["/etc/fenix/skill-categories.json"] = (str(catalog), False)
    mounts={mount["Destination"]:(mount["Source"],mount["RW"]) for mount in info["Mounts"] if mount["Type"]=="bind"}
    if mounts != expected or any(mount["Type"] not in ("bind","tmpfs") for mount in info["Mounts"]):
        raise PolicyError("Controller has unexpected host mounts")
    if info["Image"] != image_id(config["image"]) or info["Config"].get("Labels",{}).get("fenix.controller") != config.get("instance","default"):
        raise PolicyError("Controller image or service ownership changed")
    ports={str(account.get("runtime_port",4173))+"/tcp":[{"HostIp":"127.0.0.1","HostPort":str(urlsplit(account["upstream"]).port)}]}
    if host.get("PortBindings") != ports or host.get("RestartPolicy",{}).get("Name") != "no":
        raise PolicyError("Controller ingress or automatic restart bypasses the supported startup path")

if __name__ == "__main__": main()
