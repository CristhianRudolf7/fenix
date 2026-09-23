#!/usr/bin/env python3
"""Create, migrate, mount or verify a fixed-capacity account filesystem."""
import argparse
import fcntl
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from fenix_isolation import PolicyError, identifier, private_json, verify_storage


def run(*args):
    subprocess.run(args, check=True, timeout=300)


def inactive(owner):
    running = subprocess.check_output(["docker", "ps", "--format", "{{.Names}}"], text=True, timeout=15).splitlines()
    runners = subprocess.check_output(["docker", "ps", "-q", "--filter", "label=fenix.owner=" + owner], text=True, timeout=15).strip()
    if "fenix-user-" + owner in running or runners:
        raise PolicyError("Stop the account controller and all its runners before changing storage")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("config", type=Path)
    parser.add_argument("account")
    parser.add_argument("action", choices=["create", "migrate", "mount", "check"])
    parser.add_argument("--write-unit", type=Path, help="Write a system mount unit to this existing directory; activation is separate")
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise PolicyError("Account storage provisioning requires root")
    os.umask(0o077)
    config = private_json(args.config)
    owner = identifier(args.account)
    identity = config["accounts"][owner]
    uid, gid, capacity = identity["uid"], identity["gid"], identity["disk_bytes"]
    if type(uid) is not int or type(gid) is not int or min(uid, gid) < 1 or type(capacity) is not int or capacity < 64 * 1024 * 1024:
        raise PolicyError("Invalid storage identity or capacity")
    users = Path(config["users_root"]).resolve(strict=True)
    if users.stat().st_uid != 0 or users.stat().st_mode & 0o022:
        raise PolicyError("The account directory parent must be controlled by root")
    root = users / owner
    images = Path(config["volume_dir"])
    images.mkdir(mode=0o700, parents=True, exist_ok=True)
    if images.is_symlink() or images.stat().st_uid != 0 or images.stat().st_mode & 0o077 or images.is_relative_to(users):
        raise PolicyError("Volume images require private host storage outside account directories")
    image = images / (owner + ".ext4")
    with open(images / (owner + ".lock"), "a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if root.is_symlink():
            raise PolicyError("Account mount point cannot be a symlink")
        if args.action == "check":
            verify_storage(root, capacity)
            if root.stat().st_uid != uid or root.stat().st_gid != gid:
                raise PolicyError("Account filesystem ownership mismatch")
            return
        inactive(owner)
        if args.action == "mount":
            info = image.lstat()
            if not image.is_file() or image.is_symlink() or info.st_uid != 0 or info.st_mode & 0o077 or info.st_size != capacity:
                raise PolicyError("Invalid account volume image")
            root.mkdir(mode=0o700, exist_ok=True)
            if not os.path.ismount(root):
                if any(root.iterdir()):
                    raise PolicyError("Refusing to hide existing files below a mount")
                run("mount", "-o", "loop,rw,nodev,nosuid", str(image), str(root))
            verify_storage(root, capacity)
        else:
            if os.path.ismount(root) or image.exists() or image.is_symlink():
                raise PolicyError("Existing storage is never reformatted")
            if args.action == "create" and root.exists() and any(root.iterdir()):
                raise PolicyError("Use migrate to preserve existing account files")
            if args.action == "migrate" and not root.is_dir():
                raise PolicyError("Migration requires an existing account directory")
            reserve = config.get("host_disk_reserve_bytes", 2 * 1024 * 1024 * 1024)
            if type(reserve) is not int or reserve < 256 * 1024 * 1024 or shutil.disk_usage(images).free < capacity + reserve:
                raise PolicyError("Insufficient reserved host capacity for this account volume")
            fd = os.open(image, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            try:
                os.posix_fallocate(fd, 0, capacity)  # Reserve actual capacity, not a sparse-file promise.
                os.fsync(fd)
            except BaseException:
                image.unlink(missing_ok=True)
                raise
            finally:
                os.close(fd)
            run("mkfs.ext4", "-q", "-F", "-m", "0", str(image))
            stage = Path(tempfile.mkdtemp(prefix=".fenix-volume-", dir=users))
            mounted = False
            backup = users / (owner + ".pre-isolation")
            try:
                run("mount", "-o", "loop,rw,nodev,nosuid", str(image), str(stage))
                mounted = True
                if args.action == "migrate":
                    if backup.exists():
                        raise PolicyError("Existing migration backup must be reviewed first")
                    # Copy symlinks as links; never follow them while copying or changing ownership.
                    shutil.copytree(root, stage, symlinks=True, dirs_exist_ok=True)
                for directory, dirs, files in os.walk(stage, followlinks=False):
                    for name in dirs + files:
                        os.chown(Path(directory) / name, uid, gid, follow_symlinks=False)
                for name in ("workspace", "projects", "state", "skills", "memory", "artifacts", "sessions"):
                    path = stage / name
                    if path.is_symlink():
                        raise PolicyError("Account top-level directories cannot be symlinks")
                    path.mkdir(mode=0o700, exist_ok=True)
                    os.chown(path, uid, gid)
                os.chown(stage, uid, gid)
                os.chmod(stage, 0o700)
                run("sync", "-f", str(stage))
                run("umount", str(stage))
                mounted = False
                if args.action == "migrate":
                    root.rename(backup)
                    # Backups remain host-only, even though the original account UID owned them.
                    os.chown(backup, 0, 0)
                    os.chmod(backup, 0o700)
                root.mkdir(mode=0o700, exist_ok=True)
                run("mount", "-o", "loop,rw,nodev,nosuid", str(image), str(root))
                verify_storage(root, capacity)
                print(f"Account volume ready: {root}. Original migration backup, if any: {backup}")
            finally:
                if mounted:
                    run("umount", str(stage))
                stage.rmdir()
        if args.write_unit:
            unit = subprocess.check_output(["systemd-escape", "--path", "--suffix=mount", str(root)], text=True).strip()
            if any(c in str(root) + str(image) for c in ("\n", "\r", "%", " ")):
                raise PolicyError("Mount unit paths cannot contain whitespace or systemd specifiers")
            destination = args.write_unit / unit
            with destination.open("x") as output:
                output.write(f"[Unit]\nDescription=Fenix account volume {owner}\nBefore=fenix-execution.service\n\n[Mount]\nWhat={image}\nWhere={root}\nType=ext4\nOptions=loop,rw,nodev,nosuid\n\n[Install]\nWantedBy=local-fs.target\n")
            print(f"Mount unit written: {destination}; enable it before restarting the host")


if __name__ == "__main__":
    main()
