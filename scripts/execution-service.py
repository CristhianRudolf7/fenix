#!/usr/bin/env python3
"""Run the host-owned Fenix execution service (root/systemd; never inside a tenant)."""
import argparse
import asyncio
import contextlib
import fcntl
import os
from pathlib import Path
import signal
import stat
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from fenix_isolation import PolicyError, private_json
from fenix_execution.policy import Policy
from fenix_execution.docker import stop_abandoned_cgroups
from fenix_execution.backends import backend as make_backend
from fenix_execution.service import Records, Service

async def serve(config):
    instance = config.get("instance", "default")
    if not isinstance(instance, str) or not instance.isalnum() or len(instance) > 32:
        raise PolicyError("Invalid execution instance")
    instance_lock = open("/run/lock/fenix-execution-" + instance + ".lock", "a")
    fcntl.flock(instance_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    state = Path(config["state_dir"])
    state.mkdir(mode=0o700, parents=True, exist_ok=True)
    if state.is_symlink() or state.stat().st_uid != 0 or state.stat().st_mode & 0o077:
        raise PolicyError("Service state must be a private root-owned directory")
    lock = open(state / "service.lock", "a")
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    stop_abandoned_cgroups(state)
    policy = Policy(config)
    backend = make_backend(config)
    config = {**config, "image": backend.image}
    records = Records(config["state_dir"])
    service = Service(policy, backend, records)
    if config.get("checkpoint_key"):
        from fenix_execution.checkpoints import Checkpoints
        service.snapshots = Checkpoints(policy, records, config)
    if config.get("integrations_file"):
        from fenix_execution.broker import Broker
        service.broker = Broker(policy, records, config)
    await backend.cleanup_orphans()
    socket = Path(config["socket"])
    socket.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    metadata = socket.parent.stat()
    if metadata.st_uid != 0 or metadata.st_mode & 0o022:
        raise PolicyError("Control socket directory must be root-owned and not writable by tenants")
    if socket.exists():
        if not stat.S_ISSOCK(socket.lstat().st_mode):
            raise PolicyError("Refusing to replace a non-socket control path")
        socket.unlink()
    stop = asyncio.Event()
    loop = asyncio.get_running_loop()
    for name in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(name, stop.set)
    server = await asyncio.start_unix_server(service.handle, path=socket, limit=1024 * 1024 + 1)
    # Socket access alone grants nothing: requests need an account credential or
    # a one-use runner attachment capability. The parent directory is read-only.
    os.chmod(socket, 0o666)
    monitor = asyncio.create_task(service.monitor())
    try:
        async with server:
            await stop.wait()
    finally:
        monitor.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await monitor
        await service.shutdown()
        socket.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("config", type=Path)
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise PolicyError("Execution service requires host root for constrained provisioning")
    os.umask(0o077)
    asyncio.run(serve(private_json(args.config)))

if __name__ == "__main__":
    main()
