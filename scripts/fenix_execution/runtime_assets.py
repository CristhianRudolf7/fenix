"""Protected, locally pinned runtime assets shared by isolation adapters."""
import hashlib
from pathlib import Path
import stat
from fenix_isolation import PolicyError


def protected_path(value, owner=0):
    path = Path(value)
    if not path.is_absolute() or ".." in path.parts or any(c in str(path) for c in (",", "\n", "\r", ":", "\0")):
        raise PolicyError("Runtime assets require absolute protected paths")
    for parent in reversed(path.parents):
        info = parent.lstat()
        sticky = info.st_uid == 0 and bool(info.st_mode & stat.S_ISVTX)
        if not stat.S_ISDIR(info.st_mode) or info.st_uid not in (0, owner) or (info.st_mode & 0o022 and not sticky):
            raise PolicyError("Runtime asset parent is writable or untrusted")
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid != owner or info.st_mode & 0o022:
        raise PolicyError("Runtime asset must be a protected regular file")
    return path


def digest(path):
    with open(path, "rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def stamp(path):
    info = Path(path).lstat()
    return (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
            info.st_size, info.st_mtime_ns, info.st_ctime_ns)
