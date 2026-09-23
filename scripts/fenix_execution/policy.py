"""Ownership, filesystem and resource policy independent of the execution backend."""
from __future__ import annotations
import dataclasses
import hmac
import json
import os
from pathlib import Path
import re
import stat
from fenix_isolation import (Limits, PolicyError, account_fingerprint, account_root,
                             identifier, private_json, registry, verify_storage)

SAFE_ENV = {"LANG", "LC_ALL", "TERM", "COLORTERM", "NO_COLOR", "CI",
            "PYTHONDONTWRITEBYTECODE", "RUST_BACKTRACE"}
KINDS = {"shell", "language_server", "browser", "document"}


class PinnedPath:
    """Hold an inode through mount creation; never ask Docker to re-resolve a user symlink."""
    def __init__(self, root: Path, path: Path):
        if not path.is_absolute() or ".." in path.parts:
            raise PolicyError("Expected an absolute contained path")
        try:
            relative = path.relative_to(root)
        except ValueError as exc:
            raise PolicyError("Path belongs to another execution scope") from exc
        fd = os.open(root, os.O_PATH | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
        try:
            for part in relative.parts:
                nxt = os.open(part, os.O_PATH | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
                os.close(fd)
                fd = nxt
                mode = os.fstat(fd).st_mode
                if not (stat.S_ISDIR(mode) or stat.S_ISREG(mode)):
                    raise PolicyError("Symlinks and special files cannot be execution mounts")
            metadata = os.fstat(fd)
            if stat.S_ISREG(metadata.st_mode) and metadata.st_nlink != 1:
                raise PolicyError("Hard-linked mount inputs are not supported")
            self.fd, self.path = fd, path
        except BaseException:
            os.close(fd)
            raise

    @property
    def source(self):
        return f"/proc/{os.getpid()}/fd/{self.fd}"

    def close(self):
        if self.fd >= 0:
            os.close(self.fd)
            self.fd = -1


@dataclasses.dataclass
class Launch:
    owner: str
    task: str
    kind: str
    argv: list[str]
    cwd: Path
    environment: dict[str, str]
    mounts: list[tuple[PinnedPath, bool]]
    profile: str
    timeout: int
    tty: bool
    fingerprint: str
    uid: int
    gid: int
    limits: Limits
    account_limits: Limits
    readonly: bool = False
    home: Path | None = None
    egress_hosts: tuple[str, ...] | None = None
    egress_pins: dict[str, tuple[str, ...]] | None = None

    def close(self):
        for mount, _ in self.mounts:
            mount.close()


class Policy:
    def __init__(self, config: dict):
        self.config = config
        from .egress import validate
        validate(config.get("network_hosts", {}))
        for entry in config.get("accounts", {}).values():
            validate(entry.get("network_hosts", {}))
        kind = config.get("backend", "gvisor")
        if kind not in ("oci", "microvm", "gvisor"):
            raise PolicyError("Unknown execution backend; no fallback is permitted")
        if type(config.get("persistent_environments", False)) is not bool:
            raise PolicyError("Invalid persistent environment setting")
        self.runtime_overhead = (config.get("microvm_overhead_mib", 256) if kind == "microvm"
                                 else config.get("runtime_overhead_mib", 128) if kind == "gvisor" else 0)
        if type(self.runtime_overhead) is not int or not 0 <= self.runtime_overhead <= 4096:
            raise PolicyError("Invalid isolation runtime memory reservation")
        self.registry_path = Path(config["registry"])
        self.users_root = Path(config["users_root"]).resolve(strict=True)
        metadata = self.users_root.stat()
        if metadata.st_uid not in (0, os.geteuid()) or metadata.st_mode & 0o022:
            raise PolicyError("Account storage parent must be controlled by the host operator")
        for key in ("registry", "state_dir", "socket", "checkpoint_key", "integrations_file", "volume_dir"):
            if key in config and Path(config[key]).resolve().is_relative_to(self.users_root):
                raise PolicyError("Host control files must remain outside all account directories")
        self.accounts = config["accounts"]
        self.runner_limits = Limits(**config.get("runner_limits", {}))
        self.account_limits = Limits(**config.get("account_limits", {"memory_mib": 4096, "pids": 512}))
        maximum = config.get("max_executions", 32)
        memory = config.get("max_runner_memory_mib", 4096)
        if type(maximum) is not int or not 1 <= maximum <= 128 or type(memory) is not int or not self.runner_limits.memory_mib <= memory <= 131072:
            raise PolicyError("Invalid host admission budget")
        controller = Limits(**config.get("controller_limits", {"memory_mib": 1024, "pids": 128}))
        reserve = config.get("host_memory_reserve_mib", 1024)
        if type(reserve) is not int or reserve < 256:
            raise PolicyError("Host memory reserve must be at least 256 MiB")
        total = os.sysconf("SC_PHYS_PAGES") * os.sysconf("SC_PAGE_SIZE") // (1024 * 1024)
        if memory + controller.memory_mib * len(self.accounts) + reserve > total:
            raise PolicyError("Configured account and execution reservations exceed host RAM")
        ids, groups = set(), set()
        for account_id, entry in self.accounts.items():
            identifier(account_id)
            uid, gid = entry["uid"], entry["gid"]
            if (type(uid) is not int or type(gid) is not int or min(uid, gid) < 1
                    or uid in ids or gid in groups):
                raise PolicyError("Each account requires a unique non-root UID and GID")
            ids.add(uid)
            groups.add(gid)
            if type(entry["disk_bytes"]) is not int or entry["disk_bytes"] < 64 * 1024 * 1024:
                raise PolicyError("Invalid persistent storage quota")
            budget = entry.get("execution_seconds_per_month")
            if budget is not None and (type(budget) is not int or not 60 <= budget <= 2678400 * 128):
                raise PolicyError("Invalid account monthly execution budget")
            Limits(**entry.get("limits", dataclasses.asdict(self.account_limits)))
        registry_uid = config.get("registry_uid", os.geteuid())
        if type(registry_uid) is not int or registry_uid < 0 or registry_uid in ids:
            raise PolicyError("Registry owner must be a trusted operator or gateway identity, never a tenant")
        self.current()  # A corrupt/unreadable registry never becomes anonymous mode.

    def current(self):
        return registry(self.registry_path, self.config.get("registry_uid"))

    def authenticate(self, request):
        supplied = request.get("credential", "")
        account_id = request.get("account_id", "")
        if not isinstance(supplied, str) or not isinstance(account_id, str):
            raise PolicyError("Unauthorized")
        account = self.current().get(account_id)
        if (not account or account.get("disabled", False) or account_id not in self.accounts
                or type(request.get("auth_epoch", 0)) is not int or request.get("auth_epoch", 0) != account.get("auth_epoch", 0)
                or not hmac.compare_digest(supplied, account["upstream_token"])):
            raise PolicyError("Unauthorized")
        return account

    def active(self, owner, fingerprint):
        account = self.current().get(owner)
        return bool(account and not account.get("disabled", False)
                    and account_fingerprint(account) == fingerprint)

    def reservation(self, launch):
        return launch.limits.memory_mib + self.runtime_overhead

    def storage(self, owner):
        root = account_root(self.users_root, owner)
        entry = self.accounts[owner]
        if root.stat().st_uid != entry["uid"] or root.stat().st_gid != entry["gid"]:
            raise PolicyError("Account storage identity mismatch")
        verify_storage(root, entry["disk_bytes"])
        return root

    def launch(self, account, spec):
        if not isinstance(spec, dict) or set(spec) - {"task", "kind", "argv", "cwd", "workspace", "environment", "timeout", "tty", "read_paths", "resource_paths", "profile"}:
            raise PolicyError("Unknown execution specification fields")
        owner = account["id"]
        root = self.storage(owner)
        task, kind = spec.get("task"), spec.get("kind")
        if not isinstance(task, str) or not re.fullmatch(r"[A-Za-z0-9:_-]{1,128}", task) or kind not in KINDS:
            raise PolicyError("Invalid task or execution kind")
        argv = spec.get("argv")
        if (not isinstance(argv, list) or not 1 <= len(argv) <= 128
                or any(not isinstance(a, str) or len(a) > 32000 or "\0" in a for a in argv)
                or not argv[0] or sum(map(len, argv)) > 65536):
            raise PolicyError("Invalid execution arguments")
        env = spec.get("environment", {})
        if not isinstance(env, dict) or set(env) - SAFE_ENV or any(not isinstance(v, str) or len(v) > 256 or "\0" in v for v in env.values()):
            raise PolicyError("Only the minimal execution environment is allowed")
        timeout = spec.get("timeout", self.runner_limits.deadline_seconds)
        if type(timeout) is not int or not 1 <= timeout <= 604800:
            raise PolicyError("Invalid execution deadline")
        if type(spec.get("tty", False)) is not bool:
            raise PolicyError("Invalid terminal option")
        if kind == "document":
            prefix = ["/opt/fenix-ocr/bin/python", "-I", "-u", "/usr/local/lib/fenix-ocr/worker.py"]
            if (len(argv) != 5 or argv[:4] != prefix or spec.get("tty")
                    or spec.get("resource_paths") or env):
                raise PolicyError("Document execution requires the fixed image-owned worker")
            try:
                request = json.loads(argv[4])
                if (not isinstance(request, dict)
                        or set(request) != {"path", "sha256", "mode", "start_page", "page_count", "dpi"}
                        or not isinstance(request["path"], str)
                        or spec.get("read_paths") != [request["path"]]
                        or not isinstance(request["sha256"], str)
                        or not re.fullmatch(r"[0-9a-f]{64}", request["sha256"])
                        or request["mode"] not in ("auto", "native", "ocr")
                        or any(type(request[k]) is not int or not low <= request[k] <= high
                               for k, low, high in (("start_page", 1, 2000), ("page_count", 1, 20), ("dpi", 100, 300)))):
                    raise ValueError("Invalid document request")
            except (ValueError, TypeError, KeyError) as exc:
                raise PolicyError("Invalid bounded document request") from exc
            timeout = min(timeout, 300)
        cwd, workspace = Path(spec["cwd"]), Path(spec.get("workspace", spec["cwd"]))
        mounts = []
        try:
            resources = spec.get("resource_paths", [])
            if not isinstance(resources, list) or len(resources) > 16 or (resources and kind != "shell"):
                raise PolicyError("Invalid selected skill resources")
            for resource in resources:
                path = Path(resource)
                relative = path.relative_to(root / "skills")
                if len(relative.parts) != 1 or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,79}", relative.name):
                    raise PolicyError("Resources must name a selected skill directory")
                mounts.append((PinnedPath(root, path), True))
            if kind == "browser":
                if "--no-sandbox" in argv or "--disable-setuid-sandbox" in argv:
                    raise PolicyError("The browser sandbox cannot be disabled")
                # Never run an account-configured wrapper that could silently add
                # --no-sandbox. Browser binaries belong to the approved image.
                argv = [self.config.get("browser_executable", "/usr/bin/chromium"), *argv[1:]]
                if not argv[0].startswith(("/usr/", "/opt/")) or ".." in Path(argv[0]).parts:
                    raise PolicyError("Browser executable must belong to the immutable image")
                relative = cwd.relative_to(root / "state" / "browser")
                if (len(relative.parts) != 2 or relative.parts[0] not in ("test", "profiles")
                        or not re.fullmatch(r"[A-Za-z0-9_-]{1,80}", relative.parts[1])):
                    raise PolicyError("Invalid browser profile")
                mounts.append((PinnedPath(root, cwd), False))
                downloads = root / "state" / "browser" / "artifacts" / relative.parts[1] / "downloads"
                mounts.append((PinnedPath(root, downloads), False))
                profile = "web"
                readonly = False
            else:
                relative = workspace.relative_to(root)
                if not relative.parts or relative.parts[0] not in ("workspace", "projects"):
                    raise PolicyError("Controller state is never a task workspace")
                if not cwd.is_relative_to(workspace) or ".." in cwd.parts:
                    raise PolicyError("Command directory escapes the project")
                reads = spec.get("read_paths", [])
                if not isinstance(reads, list) or len(reads) > 48:
                    raise PolicyError("Invalid read-only inputs")
                readonly = bool(reads)
                if readonly and spec.get("tty"):
                    raise PolicyError("Read-only commands cannot be interactive")
                if readonly:
                    for path in reads:
                        path = Path(path)
                        if any(path.is_relative_to(Path(p)) for p in ("/usr", "/bin", "/sbin", "/lib", "/lib64")) or path in (Path("/etc"), Path("/dev/null"), Path("/dev/urandom")):
                            continue  # These come only from the immutable image.
                        if any(path.is_relative_to(Path(resource)) for resource in resources):
                            continue
                        mounts.append((PinnedPath(workspace, path), True))
                    if not mounts:
                        raise PolicyError("Read-only execution requires project inputs")
                    if kind == "document" and (len(mounts) != 1 or not stat.S_ISREG(os.fstat(mounts[0][0].fd).st_mode)):
                        raise PolicyError("Document input must be one project file")
                    profile = "offline"
                else:
                    mounts.append((PinnedPath(root, workspace), kind == "language_server"))
                    profile = "offline" if kind == "language_server" else spec.get("profile", "development")
                    if profile not in ("offline", "development"):
                        raise PolicyError("Invalid task network profile")
            for mount, _ in mounts:
                if any(c in str(mount.path) for c in (",", "\n", "\r", ":")):
                    raise PolicyError("Unsupported mount path")
            identity = self.accounts[owner]
            home = None
            if self.config.get("persistent_environments", False) and kind != "browser" and not readonly:
                from .workspaces import environment
                pinned = environment(root, workspace, identity["uid"], identity["gid"])
                mounts.append((pinned, kind == "language_server"))
                home = pinned.path
            from .egress import effective
            hosts = effective(self.config.get("network_hosts", {}), identity.get("network_hosts", {}), profile)
            return Launch(owner, task, kind, argv, cwd, env, mounts, profile,
                          min(timeout, self.runner_limits.deadline_seconds), spec.get("tty", False),
                          account_fingerprint(account), identity["uid"], identity["gid"],
                          self.runner_limits, Limits(**identity.get("limits", dataclasses.asdict(self.account_limits))), readonly, home, hosts)
        except BaseException:
            for mount, _ in mounts:
                mount.close()
            raise
