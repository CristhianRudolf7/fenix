"""Persistent per-workspace tool homes within an account's bounded filesystem.

Only descriptor-relative directory creation is permitted. A tenant may rename
their directories, but cannot turn provisioning into an operation on another
account, a symlink target or a host control directory.
"""
import hashlib
import os
from pathlib import Path
import stat
from fenix_isolation import PolicyError


def environment(root: Path, workspace: Path, uid: int, gid: int):
    from .policy import PinnedPath
    relative = workspace.relative_to(root)
    if not relative.parts or relative.parts[0] not in ("workspace", "projects") or ".." in relative.parts:
        raise PolicyError("Environment must belong to a project")
    key = hashlib.sha256(str(relative).encode()).hexdigest()
    fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        for part in ("environments", key):
            created = False
            try:
                os.mkdir(part, mode=0o700, dir_fd=fd)
                created = True
            except FileExistsError:
                pass
            nxt = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
            os.close(fd)
            fd = nxt
            if created:
                os.fchown(fd, uid, gid)
                os.fsync(fd)
            info = os.fstat(fd)
            if info.st_uid != uid or info.st_gid != gid or stat.S_IMODE(info.st_mode) & 0o077:
                raise PolicyError("Workspace environment must be private to its account")
        # Transfer this exact inode; do not re-open a path the user may replace.
        pinned = object.__new__(PinnedPath)
        pinned.fd, pinned.path = fd, root / "environments" / key
        fd = -1
        return pinned
    finally:
        if fd >= 0:
            os.close(fd)


def variables(home):
    """Platform-owned defaults; callers cannot inject loader or startup hooks."""
    home = str(home)
    return {
        "HOME": home,
        "PATH": home + "/.local/bin:" + home + "/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "XDG_CACHE_HOME": home + "/.cache",
        "XDG_CONFIG_HOME": home + "/.config",
        "XDG_DATA_HOME": home + "/.local/share",
        "PYTHONUSERBASE": home + "/.local",
        "NPM_CONFIG_PREFIX": home + "/.local",
        "NPM_CONFIG_CACHE": home + "/.cache/npm",
        "CARGO_HOME": home + "/.cargo",
        "RUSTUP_HOME": home + "/.rustup",
    }
