"""Shared, fail-closed Linux provisioning policy. No work is performed on import."""
from __future__ import annotations
import dataclasses
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
from urllib.parse import urlsplit
import uuid

class PolicyError(ValueError):
    pass

def identifier(value: str) -> str:
    if not isinstance(value, str) or str(uuid.UUID(value)) != value:
        raise PolicyError("Expected a canonical account or execution UUID")
    return value

def private_json(path: Path, owner_uid=None):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    with os.fdopen(fd, "r") as stream:
        metadata = os.fstat(stream.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_mode & 0o077:
            raise PolicyError("Configuration must be a private regular file (0600)")
        owners = (0, os.geteuid()) if owner_uid is None else (owner_uid,)
        if metadata.st_uid not in owners or metadata.st_size > 4 * 1024 * 1024:
            raise PolicyError("Configuration has an unexpected owner or size")
        return json.load(stream)

def registry(path: Path, owner_uid=None) -> dict[str, dict]:
    data = private_json(path, owner_uid)
    result, emails, upstreams, tokens = {}, set(), set(), set()
    if not isinstance(data, dict) or not isinstance(data.get("accounts"), list):
        raise PolicyError("Invalid account registry")
    for entry in data["accounts"]:
        if type(entry.get("disabled", False)) is not bool or type(entry.get("auth_epoch", 0)) is not int or entry.get("auth_epoch", 0) < 0:
            raise PolicyError("Invalid account revocation state")
        key = identifier(entry["id"])
        email = entry["email"]
        parsed = urlsplit(entry["upstream"])
        port = parsed.port
        if (parsed.scheme != "http" or parsed.hostname != "127.0.0.1"
                or not port or parsed.path not in ("", "/") or parsed.query
                or parsed.fragment or parsed.username or parsed.password):
            raise PolicyError("Upstream must be an explicit loopback HTTP port")
        token = entry["upstream_token"]
        if not isinstance(token, str) or not re.fullmatch(r"[A-Za-z0-9_-]{32,256}", token):
            raise PolicyError("Invalid upstream credential")
        if not isinstance(email, str) or email != email.strip().lower() or "@" not in email:
            raise PolicyError("Invalid normalized email")
        if key in result or email in emails or port in upstreams or token in tokens:
            raise PolicyError("Duplicate account, email, port or upstream credential")
        if type(entry.get("runtime_port", 4173)) is not int or not 1 <= entry.get("runtime_port", 4173) <= 65535:
            raise PolicyError("Invalid runtime port")
        result[key] = entry
        emails.add(email)
        upstreams.add(port)
        tokens.add(token)
    return result

def account_root(users: Path, account_id: str) -> Path:
    users = users.resolve(strict=True)
    root = users / identifier(account_id)
    if root.is_symlink() or root.resolve(strict=True).parent != users or not root.is_dir():
        raise PolicyError("Invalid account directory")
    if any(c in str(root) for c in (",", "\n", "\r", "\x00")):
        raise PolicyError("Account path cannot contain Docker mount delimiters")
    return root

def private_write(path: Path, content: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    fd, temporary = tempfile.mkstemp(prefix=".fenix-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)

@dataclasses.dataclass(frozen=True)
class Limits:
    memory_mib: int = 2048
    cpus: float = 2
    pids: int = 256
    nofile: int = 1024
    tmp_mib: int = 256
    log_mib: int = 8
    concurrent: int = 4
    deadline_seconds: int = 3600

    def __post_init__(self):
        bounds = {"memory_mib": (128, 131072), "pids": (16, 4096),
                  "nofile": (64, 65536), "tmp_mib": (16, 4096),
                  "log_mib": (1, 64), "concurrent": (1, 32),
                  "deadline_seconds": (1, 604800)}
        for name, (low, high) in bounds.items():
            value = getattr(self, name)
            if type(value) is not int or not low <= value <= high:
                raise PolicyError(f"Invalid resource limit: {name}")
        if type(self.cpus) not in (int, float) or not 0.1 <= self.cpus <= 64:
            raise PolicyError("Invalid CPU quota")
        if self.tmp_mib > self.memory_mib:
            raise PolicyError("Temporary storage exceeds the memory budget")

    def docker(self) -> list[str]:
        return ["--memory", f"{self.memory_mib}m", "--memory-swap", f"{self.memory_mib}m",
                "--cpus", str(self.cpus), "--pids-limit", str(self.pids),
                "--ulimit", f"nofile={self.nofile}:{self.nofile}",
                "--ulimit", "core=0:0", "--log-driver", "local",
                "--log-opt", f"max-size={self.log_mib}m", "--log-opt", "max-file=2"]

def image_id(image: str) -> str:
    value = subprocess.check_output(
        ["docker", "image", "inspect", "--format", "{{.Id}}", image],
        text=True, timeout=15).strip()
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", value):
        raise PolicyError("Cannot resolve an immutable local image digest")
    return value

def verify_storage(root: Path, maximum_bytes: int) -> None:
    """Require a dedicated, bounded filesystem, not an unchecked bind mount."""
    if maximum_bytes < 64 * 1024 * 1024 or not os.path.ismount(root):
        raise PolicyError("Account storage must be a dedicated bounded filesystem; use tenant-storage.py")
    info = os.statvfs(root)
    if info.f_blocks * info.f_frsize > maximum_bytes:
        raise PolicyError("Account filesystem exceeds its configured capacity")
    if os.stat(root).st_dev == os.stat(root.parent).st_dev:
        raise PolicyError("Account storage must not share the parent filesystem")

def account_fingerprint(account: dict) -> str:
    # Profile edits and self-service sign-in password changes do not revoke
    # execution credentials. Explicit disable/CLI resets still change auth_epoch.
    execution = {key: value for key, value in account.items()
                 if key not in {"display_name", "password_hash"}}
    return hashlib.sha256(json.dumps(execution, sort_keys=True).encode()).hexdigest()

def account_slice(account_id: str, limits: Limits) -> str:
    name = f"fenixacct-{identifier(account_id).replace('-', '')}.slice"
    subprocess.run(["systemctl", "set-property", "--runtime", name,
                    f"MemoryMax={limits.memory_mib}M", "MemorySwapMax=0",
                    f"CPUQuota={limits.cpus * 100:g}%", f"TasksMax={limits.pids}"],
                   check=True, timeout=15)
    subprocess.run(["systemctl", "start", name], check=True, timeout=15)
    return name
