"""Authenticated, owner-bound file checkpoints; never RAM, cookies or controller state."""
from __future__ import annotations
import io
import json
import os
from pathlib import Path, PurePosixPath
import secrets
import shutil
import stat
import sqlite3
import tarfile
import time
import uuid
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from fenix_isolation import PolicyError, account_fingerprint, identifier, private_json, private_write
from .policy import PinnedPath


class BoundedBuffer(io.BytesIO):
    def __init__(self, limit):
        super().__init__()
        self.limit = limit

    def write(self, value):
        if self.tell() + len(value) > self.limit:
            raise PolicyError("Checkpoint exceeds its size limit")
        return super().write(value)


def archive_tree(fd, limit):
    output = BoundedBuffer(limit)
    count = 0
    with tarfile.open(fileobj=output, mode="w|") as archive:
        def visit(directory, prefix, depth):
            nonlocal count
            if depth > 64:
                raise PolicyError("Checkpoint tree is too deep")
            for name in sorted(os.listdir(directory)):
                count += 1
                if count > 10000:
                    raise PolicyError("Checkpoint contains too many entries")
                child = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=directory)
                try:
                    before = os.fstat(child)
                    path = prefix + name
                    info = tarfile.TarInfo(path)
                    info.mtime = int(before.st_mtime)
                    info.mode = before.st_mode & 0o777  # No setuid/setgid bits, owners or extended attributes.
                    if stat.S_ISDIR(before.st_mode):
                        info.type = tarfile.DIRTYPE
                        archive.addfile(info)
                        visit(child, path + "/", depth + 1)
                    elif stat.S_ISREG(before.st_mode) and before.st_nlink == 1:
                        info.size = before.st_size
                        with os.fdopen(os.dup(child), "rb") as stream:
                            archive.addfile(info, stream)
                        after = os.fstat(child)
                        if (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (after.st_size, after.st_mtime_ns, after.st_ctime_ns):
                            raise PolicyError("Project changed during checkpoint creation")
                    else:
                        raise PolicyError("Checkpoints reject links and special files")
                finally:
                    os.close(child)
        visit(fd, "", 0)
    return output.getvalue()


class Checkpoints:
    def __init__(self, policy, records, config):
        self.policy, self.records = policy, records
        self.key = bytes.fromhex(private_json(config["checkpoint_key"])["key"])
        if len(self.key) != 32:
            raise PolicyError("Checkpoint key must contain 32 random bytes")
        self.directory = records.directory / "checkpoints"
        self.directory.mkdir(mode=0o700, exist_ok=True)
        if self.directory.is_symlink() or self.directory.stat().st_mode & 0o077:
            raise PolicyError("Checkpoint directory must be private")
        self.image = config["image"]
        self.limit = config.get("checkpoint_max_bytes", 32 * 1024 * 1024)
        self.quota = config.get("checkpoint_account_bytes", 128 * 1024 * 1024)
        self.global_quota = config.get("checkpoint_total_bytes", 1024 * 1024 * 1024)
        if type(self.limit) is not int or not 10240 <= self.limit <= 128 * 1024 * 1024 or not self.limit <= self.quota <= self.global_quota:
            raise PolicyError("Invalid checkpoint quotas")
        self.db = sqlite3.connect(records.directory / "checkpoints.sqlite", check_same_thread=False)
        os.chmod(records.directory / "checkpoints.sqlite", 0o600)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.db.execute("PRAGMA synchronous=FULL")
        self.db.execute("PRAGMA max_page_count=16384")
        self.db.execute("PRAGMA journal_size_limit=8388608")
        self.db.execute("CREATE TABLE IF NOT EXISTS checkpoints(id TEXT PRIMARY KEY,owner TEXT NOT NULL,metadata TEXT NOT NULL,size INTEGER NOT NULL,expires REAL NOT NULL)")
        self.db.commit()
        # File paths are unguessable but authorization always uses the database owner.
        known = {row[0] for row in self.db.execute("SELECT id FROM checkpoints")}
        for path in self.directory.iterdir():
            if path.name.endswith(".enc") and path.stem not in known:
                path.unlink()  # Incomplete commit from a previous service crash.
        self.expire()

    def expire(self):
        for (key,) in self.db.execute("SELECT id FROM checkpoints WHERE expires<=?", (time.time(),)).fetchall():
            self.delete(key)

    def delete(self, key):
        identifier(key)
        (self.directory / (key + ".enc")).unlink(missing_ok=True)
        self.db.execute("DELETE FROM checkpoints WHERE id=?", (key,))
        self.db.commit()

    def owned(self, owner, key):
        identifier(key)
        row = self.db.execute("SELECT metadata,size FROM checkpoints WHERE owner=? AND id=? AND expires>?", (owner, key, time.time())).fetchone()
        if row is None:
            raise PolicyError("Checkpoint not found")
        return row

    def dispatch(self, account, request):
        # Calls run in a worker thread and are serialized by service admission.
        self.expire()
        owner = account["id"]
        if not self.policy.active(owner, account_fingerprint(account)):
            raise PolicyError("Account revoked")
        operation = request["operation"]
        if operation == "list_checkpoints":
            return {"checkpoints": [json.loads(row[0]) for row in self.db.execute("SELECT metadata FROM checkpoints WHERE owner=? ORDER BY expires DESC LIMIT 32", (owner,))]}
        if operation == "checkpoint":
            return self.create(account, request)
        key = request.get("id", "")
        metadata, size = self.owned(owner, key)
        if operation == "delete_checkpoint":
            self.delete(key)
            return {"deleted": key}
        if operation not in ("restore", "clone"):
            raise PolicyError("Unsupported checkpoint operation")
        info = json.loads(metadata)
        if info["image"] != self.image or info["policy_version"] != 1:
            raise PolicyError("Checkpoint requires the original approved image and policy")
        with open(self.directory / (key + ".enc"), "rb") as stream:
            blob = stream.read(self.limit + 29)
        if len(blob) != size or size > self.limit + 28:
            raise PolicyError("Invalid checkpoint ciphertext size")
        try:
            data = AESGCM(self.key).decrypt(blob[:12], blob[12:], metadata.encode())
        except Exception:
            raise PolicyError("Checkpoint authentication failed") from None
        root = self.policy.storage(owner)
        destination = self.workspace(root, request["destination"])
        if destination.exists() or destination.is_symlink():
            raise PolicyError("Restore requires a new destination; existing files are never overwritten")
        parent = PinnedPath(root, destination.parent)
        identity = self.policy.accounts[owner]
        name = destination.name
        created = False
        try:
            with tarfile.open(fileobj=io.BytesIO(data), mode="r:") as archive:
                members = archive.getmembers()
                names, total = set(), 0
                for member in members:
                    path = PurePosixPath(member.name)
                    if (not member.name or path.is_absolute() or ".." in path.parts or len(path.parts) > 64
                            or not (member.isdir() or member.isfile()) or member.name in names
                            or member.size < 0 or member.mode & ~0o777):
                        raise PolicyError("Invalid checkpoint archive member")
                    names.add(member.name)
                    total += member.size
                if len(members) > 10000 or total > self.limit:
                    raise PolicyError("Checkpoint expansion exceeds quota")
                if not self.policy.active(owner, account_fingerprint(account)):
                    raise PolicyError("Account revoked before restore")
                os.mkdir(name, mode=0o700, dir_fd=parent.fd)
                created = True
                target_fd = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent.fd)
                try:
                    # The destination remains root-owned and private until fully restored.
                    for member in members:
                        parts = PurePosixPath(member.name).parts
                        directory = os.dup(target_fd)
                        try:
                            for part in parts[:-1]:
                                directory_next = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=directory)
                                os.close(directory)
                                directory = directory_next
                            if member.isdir():
                                os.mkdir(parts[-1], 0o700, dir_fd=directory)
                            else:
                                fd = os.open(parts[-1], os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=directory)
                                with os.fdopen(fd, "wb") as output, archive.extractfile(member) as source:
                                    shutil.copyfileobj(source, output, 65536)
                                    os.fchmod(output.fileno(), member.mode & 0o777)
                                    os.fchown(output.fileno(), identity["uid"], identity["gid"])
                                    output.flush()
                                    os.fsync(output.fileno())
                        finally:
                            os.close(directory)
                    # Child ownership changes only once no further untrusted path is opened.
                    for directory, dirs, _ in os.walk(f"/proc/self/fd/{target_fd}", topdown=False):
                        for child in dirs:
                            os.chown(Path(directory) / child, identity["uid"], identity["gid"], follow_symlinks=False)
                    os.fchown(target_fd, identity["uid"], identity["gid"])
                    os.fsync(target_fd)
                finally:
                    os.close(target_fd)
            return {"destination": str(destination), "generation": str(uuid.uuid4()), "type": "files", "external_effects_rolled_back": False}
        except BaseException:
            if created:
                shutil.rmtree(name, dir_fd=parent.fd)
            raise
        finally:
            parent.close()

    def workspace(self, root, value):
        path = Path(value)
        relative = path.relative_to(root)
        if not relative.parts or relative.parts[0] not in ("workspace", "projects") or ".." in path.parts:
            raise PolicyError("Checkpoint scope must be a project workspace")
        return path

    def create(self, account, request):
        owner = account["id"]
        root = self.policy.storage(owner)
        source = self.workspace(root, request["workspace"])
        pinned = PinnedPath(root, source)
        try:
            read_fd = os.open(pinned.source, os.O_RDONLY | os.O_DIRECTORY)
            try:
                data = archive_tree(read_fd, self.limit)
            finally:
                os.close(read_fd)
        finally:
            pinned.close()
        ttl = request.get("ttl_seconds", 86400)
        if type(ttl) is not int or not 60 <= ttl <= 7 * 86400:
            raise PolicyError("Checkpoint retention must be between one minute and seven days")
        size = len(data) + 28
        count, used = self.db.execute("SELECT count(*),coalesce(sum(size),0) FROM checkpoints WHERE owner=?", (owner,)).fetchone()
        total = self.db.execute("SELECT coalesce(sum(size),0) FROM checkpoints").fetchone()[0]
        if count >= 32 or used + size > self.quota or total + size > self.global_quota:
            raise PolicyError("Checkpoint storage quota reached")
        key, now = str(uuid.uuid4()), time.time()
        info = dict(id=key, owner=owner, generation=key, image=self.image, policy_version=1, created=now, expires=now+ttl, type="files", workspace=str(source))
        metadata = json.dumps(info, sort_keys=True, separators=(",", ":"))
        nonce = secrets.token_bytes(12)
        ciphertext = nonce + AESGCM(self.key).encrypt(nonce, data, metadata.encode())
        private_write(self.directory / (key + ".enc"), ciphertext)
        try:
            self.db.execute("INSERT INTO checkpoints VALUES(?,?,?,?,?)", (key, owner, metadata, size, now+ttl))
            self.db.commit()
        except BaseException:
            (self.directory / (key + ".enc")).unlink(missing_ok=True)
            raise
        return {"checkpoint": info, "bytes": size}
