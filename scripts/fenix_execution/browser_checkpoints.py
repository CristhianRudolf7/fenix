"""Private, streaming-encrypted gVisor memory checkpoints, bound to one owner.

Snapshots are resumed once, never cloned. Writable browser mounts must match
exactly; process state is not permission to replay a network operation.
"""
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import stat
import sqlite3
import time
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from fenix_isolation import PolicyError, identifier, private_json, private_write

CHUNK = 1024 * 1024


def tree_fingerprint(paths):
    digest = hashlib.sha256()
    count = 0
    for root in paths:
        root = Path(root)
        for base, dirs, files in os.walk(root, followlinks=False):
            dirs.sort(); files.sort()
            for name in sorted(dirs + files):
                count += 1
                if count > 100000:
                    raise PolicyError('Browser profile has too many entries to checkpoint')
                path = Path(base) / name
                info = path.lstat()
                digest.update(str(path.relative_to(root)).encode())
                digest.update(str((info.st_mode, info.st_size, info.st_mtime_ns)).encode())
                if stat.S_ISLNK(info.st_mode):
                    digest.update(os.readlink(path).encode())
                elif stat.S_ISREG(info.st_mode):
                    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
                    with os.fdopen(fd, 'rb') as source:
                        for chunk in iter(lambda: source.read(CHUNK), b''):
                            digest.update(chunk)
                elif not stat.S_ISDIR(info.st_mode) and not stat.S_ISSOCK(info.st_mode):
                    raise PolicyError('Unsupported browser profile entry')
    return digest.hexdigest()


class BrowserCheckpoints:
    def __init__(self, records, config):
        self.db = sqlite3.connect(records.directory / "browser-checkpoints.sqlite", check_same_thread=False)
        os.chmod(records.directory / "browser-checkpoints.sqlite", 0o600)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.db.execute("PRAGMA synchronous=FULL")
        self.key = bytes.fromhex(private_json(config['checkpoint_key'])['key'])
        if len(self.key) != 32:
            raise PolicyError('Invalid browser checkpoint key')
        self.directory = records.directory / 'browser-checkpoints'
        self.directory.mkdir(mode=0o700, exist_ok=True)
        if self.directory.is_symlink() or self.directory.stat().st_mode & 0o077:
            raise PolicyError('Browser checkpoints require private host storage')
        self.limit = config.get('browser_checkpoint_max_bytes', 2 * 1024**3)
        self.quota = config.get('browser_checkpoint_total_bytes', 8 * 1024**3)
        self.owner_quota = config.get('browser_checkpoint_account_bytes', 4 * 1024**3)
        if not 1024**2 <= self.limit <= self.owner_quota <= self.quota:
            raise PolicyError('Invalid browser checkpoint storage quotas')
        self.db.execute('CREATE TABLE IF NOT EXISTS browser_checkpoints(id TEXT PRIMARY KEY, owner TEXT NOT NULL, data BLOB NOT NULL, bytes INTEGER NOT NULL, expires REAL NOT NULL)')
        self.db.commit()

    def metadata(self, owner, key):
        identifier(key)
        row = self.db.execute('SELECT data FROM browser_checkpoints WHERE id=? AND owner=?', (key, owner)).fetchone()
        if row is None:
            raise PolicyError('Browser checkpoint not found')
        raw = row[0]
        try:
            return json.loads(AESGCM(self.key).decrypt(raw[:12], raw[12:], (owner + ':' + key).encode()))
        except Exception as error:
            raise PolicyError('Browser checkpoint authentication failed') from error

    def items(self):
        return [(key, owner, self.metadata(owner, key)) for key, owner in self.db.execute('SELECT id,owner FROM browser_checkpoints').fetchall()]

    def ensure_space(self, owner, expected):
        total, own = self.db.execute('SELECT COALESCE(SUM(bytes),0), COALESCE(SUM(CASE WHEN owner=? THEN bytes ELSE 0 END),0) FROM browser_checkpoints', (owner,)).fetchone()
        if expected > self.limit or total + expected > self.quota or own + expected > self.owner_quota:
            raise PolicyError('Browser checkpoint disk quota reached; session remains active')
        if shutil.disk_usage(self.directory).free < expected * 2 + 256 * 1024**2:
            raise PolicyError('Insufficient disk space to safely save this browser')

    def seal(self, owner, key, source, metadata):
        identifier(key)
        source = Path(source)
        files = sorted(source.rglob('*'))
        if not files or len(files) > 10000 or any(p.is_symlink() or not (p.is_file() or p.is_dir()) for p in files):
            raise PolicyError('Invalid browser checkpoint files')
        total = sum(p.stat().st_size for p in files if p.is_file())
        self.ensure_space(owner, total)
        destination = self.directory / key
        destination.mkdir(mode=0o700)
        manifest = []
        try:
            for number, path in enumerate(p for p in files if p.is_file()):
                aad = f'{owner}:{key}:{number}'.encode()
                nonce = secrets.token_bytes(12)
                cipher = Cipher(algorithms.AES(self.key), modes.GCM(nonce)).encryptor()
                cipher.authenticate_additional_data(aad)
                target = destination / str(number)
                with path.open('rb') as src, target.open('xb') as out:
                    os.chmod(target, 0o600)
                    out.write(nonce)
                    for chunk in iter(lambda: src.read(CHUNK), b''):
                        out.write(cipher.update(chunk))
                    out.write(cipher.finalize()); out.write(cipher.tag)
                    out.flush(); os.fsync(out.fileno())
                manifest.append(str(path.relative_to(source)))
            metadata = dict(metadata, files=manifest)
            nonce = secrets.token_bytes(12)
            data = nonce + AESGCM(self.key).encrypt(nonce, json.dumps(metadata).encode(), (owner + ':' + key).encode())
            for directory in (destination, self.directory):
                fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
                try: os.fsync(fd)
                finally: os.close(fd)
            self.db.execute('INSERT INTO browser_checkpoints VALUES (?,?,?,?,?)', (key, owner, data, total, metadata['expires']))
            self.db.commit()
        except BaseException:
            shutil.rmtree(destination, ignore_errors=True)
            raise

    def unseal(self, owner, key, target):
        metadata = self.metadata(owner, key)
        target = Path(target)
        if target.exists():
            raise PolicyError('Browser restore directory must be new')
        target.mkdir(mode=0o700, parents=True)
        try:
            for number, name in enumerate(metadata['files']):
                relative = Path(name)
                if relative.is_absolute() or '..' in relative.parts:
                    raise PolicyError('Invalid checkpoint path')
                source = self.directory / key / str(number)
                with source.open('rb') as src:
                    nonce = src.read(12); src.seek(-16, 2); tag = src.read(16); src.seek(12)
                    cipher = Cipher(algorithms.AES(self.key), modes.GCM(nonce, tag)).decryptor()
                    cipher.authenticate_additional_data(f'{owner}:{key}:{number}'.encode())
                    destination = target / relative
                    destination.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                    remaining = source.stat().st_size - 28
                    with destination.open('xb') as out:
                        os.chmod(destination, 0o600)
                        while remaining:
                            chunk = src.read(min(CHUNK, remaining))
                            if not chunk: raise PolicyError('Truncated browser checkpoint')
                            remaining -= len(chunk); out.write(cipher.update(chunk))
                        out.write(cipher.finalize())
                        out.flush(); os.fsync(out.fileno())
        except BaseException:
            shutil.rmtree(target, ignore_errors=True)
            raise
        return metadata

    def delete(self, key):
        identifier(key)
        self.db.execute('DELETE FROM browser_checkpoints WHERE id=?', (key,)); self.db.commit()
        shutil.rmtree(self.directory / key, ignore_errors=True)
