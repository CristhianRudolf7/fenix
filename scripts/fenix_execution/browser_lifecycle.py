"""Suspend/resume browser leases without treating a saved browser as cancelled."""
import asyncio
import contextlib
import hashlib
import os
import secrets
import shutil
import time
from fenix_isolation import PolicyError
from .browser_checkpoints import BrowserCheckpoints, tree_fingerprint


async def disk_operation(function, *args):
    # Cancellation cannot release the lifecycle lock while a disk write still runs.
    job = asyncio.create_task(asyncio.to_thread(function, *args))
    try:
        return await asyncio.shield(job)
    except asyncio.CancelledError:
        await job
        raise


class BrowserLifecycle:
    def __init__(self, service):
        self.service = service
        self.lock = asyncio.Lock()
        self.store = BrowserCheckpoints(service.records, service.policy.config) if service.policy.config.get('checkpoint_key') else None

    @property
    def supported(self):
        return self.store is not None and getattr(self.service.backend, 'browser_checkpoints', False) is True

    @property
    def runtime_identity(self):
        value = getattr(self.service.backend, 'checkpoint_identity', None)
        return value if isinstance(value, str) else None

    async def capture_metadata(self, execution):
        paths = [m.source for m, readonly in execution.launch.mounts if not readonly]
        return dict(spec=execution.spec, fingerprint=execution.launch.fingerprint,
                    files_fingerprint=await disk_operation(tree_fingerprint, paths),
                    image=self.service.backend.image, runtime=self.runtime_identity,
                    expires=time.time() + self.service.policy.config.get('browser_checkpoint_ttl_seconds', 86400),
                    egress_pins=execution.launch.egress_pins)

    def mark_suspended(self, execution, metadata):
        execution.phase = 'suspended'
        execution.expires = metadata['expires']
        execution.deadline = time.monotonic() + metadata['expires'] - time.time()
        self.service.records.settle(execution)
        self.service.records.status(execution.id, 'suspended')

    async def suspend(self, owner, key):
        service = self.service
        if not self.supported:
            raise PolicyError('Browser hibernation is unavailable; the browser remains active')
        execution = service.owned(owner, key)
        if execution.launch.kind != 'browser':
            raise PolicyError('Only browser executions can hibernate')
        async with execution.lock, self.lock:
            if execution.phase == 'suspended':
                return dict(id=key, suspended=True)
            if execution.phase != 'running' or not execution.attached:
                raise PolicyError('Browser is not ready to hibernate')
            maximum = service.policy.reservation(execution.launch) * 1024**2
            estimate = await service.backend.checkpoint_estimate(key, maximum) if getattr(type(service.backend), 'checkpoint_estimate', None) else maximum
            self.store.ensure_space(owner, estimate)
            execution.phase = 'suspending'
            # Close and join every docker-exec CDP relay before snapshotting it.
            execution.disconnect.set()
            checkpoint_attempted = False
            try:
                if execution.tunnels:
                    await asyncio.wait_for(asyncio.gather(*list(execution.tunnels), return_exceptions=True), 15)
                checkpoint_attempted = True
                source = await service.backend.checkpoint_browser(key)
                execution.raw_checkpoint = True
                metadata = await self.capture_metadata(execution)
                execution.checkpoint_metadata = metadata
                await disk_operation(self.store.seal, owner, key, source, metadata)
                await disk_operation(shutil.rmtree, source)
                execution.raw_checkpoint = False
                self.mark_suspended(execution, metadata)
                service.records.audit('browser_suspend', owner, key)
                return dict(id=key, suspended=True)
            except BaseException:
                # The Docker command can lose its response after committing a snapshot.
                if checkpoint_attempted and not execution.raw_checkpoint:
                    with contextlib.suppress(Exception):
                        info = await service.backend.inspect(key)
                        source = await service.backend.checkpoint_directory(key)
                        if info['State']['Running'] is False and source.is_dir():
                            execution.raw_checkpoint = True
                if execution.raw_checkpoint:
                    if execution.checkpoint_metadata is None:
                        with contextlib.suppress(Exception):
                            execution.checkpoint_metadata = await self.capture_metadata(execution)
                    metadata = execution.checkpoint_metadata or dict(expires=time.time() + 3600)
                    self.mark_suspended(execution, metadata)
                    service.records.audit('browser_suspend', owner, key, 'failed', reason='snapshot retained for recovery')
                # A stopped browser still has the original raw snapshot. Keep it
                # recoverable in this service instead of deleting a user's form.
                execution.phase = 'suspended' if execution.raw_checkpoint else 'running'
                execution.disconnect = asyncio.Event()
                raise

    async def resume(self, account, key):
        service = self.service
        execution = service.owned(account['id'], key)
        async with service.admission:
            if execution.phase != 'suspended':
                raise PolicyError('Browser is not suspended')
            service.check_capacity(execution.launch, exclude=key)
            execution.phase = 'resuming'
        try:
            async with execution.lock, self.lock:
                metadata = execution.checkpoint_metadata or self.store.metadata(account['id'], key)
                paths = [m.source for m, readonly in execution.launch.mounts if not readonly]
                if metadata['image'] != service.backend.image or metadata.get('runtime') != self.runtime_identity or metadata['files_fingerprint'] != await disk_operation(tree_fingerprint, paths):
                    raise PolicyError('Browser files changed while suspended; restore was refused')
                if not execution.raw_checkpoint:
                    directory = await service.backend.checkpoint_directory(key)
                    try:
                        await disk_operation(self.store.unseal, account['id'], key, directory)
                    finally:
                        # A cancelled request can finish its authenticated disk copy.
                        execution.raw_checkpoint = directory.is_dir()
                service.records.reserve_resume(execution, service.policy.accounts[account['id']].get('execution_seconds_per_month'))
                execution.attached = False
                execution.generation += 1
                capability = secrets.token_urlsafe(32)
                execution.capability_hash = hashlib.sha256(capability.encode()).digest()
                execution.attach_deadline = time.monotonic() + 30
                execution.disconnect = asyncio.Event()
                execution.started = time.monotonic()
                execution.deadline = execution.started + execution.launch.timeout
                execution.resume_pending = True
                return dict(id=key, capability=capability, timeout=execution.launch.timeout, generation=execution.generation)
        except BaseException:
            execution.phase = 'suspended'
            execution.attached = True
            execution.deadline = time.monotonic() + max(1, execution.expires - time.time())
            raise

    def release_unattached(self, execution):
        self.service.records.settle(execution)
        execution.phase = 'suspended'
        execution.resume_pending = False
        execution.attached = True
        execution.deadline = time.monotonic() + max(1, execution.expires - time.time())

    async def restored(self, execution):
        # Only called after isolation verification and a successful restore.
        async with self.lock:
            source = await self.service.backend.checkpoint_directory(execution.id)
            if source.exists():
                await disk_operation(shutil.rmtree, source)
            self.store.delete(execution.id)
            execution.raw_checkpoint = False
            execution.checkpoint_metadata = None
            execution.resume_pending = False

    async def recover(self):
        from .service import Execution
        if not self.store:
            return set()
        kept = set()
        for key, owner in self.store.db.execute('SELECT id,owner FROM browser_checkpoints').fetchall():
            launch = None
            try:
                metadata = self.store.metadata(owner, key)
                account = self.service.policy.current()[owner]
                if metadata['expires'] <= time.time() or metadata['image'] != self.service.backend.image or not self.service.policy.active(owner, metadata['fingerprint']):
                    raise PolicyError('Saved browser expired or its authorization changed')
                if metadata.get('runtime') != self.runtime_identity:
                    raise PolicyError('Browser checkpoint runtime changed')
                launch = self.service.policy.launch(account, metadata['spec'])
                if launch.kind != 'browser': raise PolicyError('Invalid saved execution kind')
                info = await self.service.backend.inspect(key)
                if info['State']['Running'] or info['Config']['Labels'].get('fenix.owner') != owner:
                    raise PolicyError('Saved browser container identity mismatch')
                # A crash during decryption can leave an incomplete native copy.
                # Recreate it from authenticated storage on the next admission.
                raw = await self.service.backend.checkpoint_directory(key)
                if raw.exists():
                    await disk_operation(shutil.rmtree, raw)
                for index, (mount, _) in enumerate(launch.mounts):
                    original = (self.service.backend.mount_dir / key / str(index)).stat()
                    current = os.fstat(mount.fd)
                    if (original.st_dev, original.st_ino) != (current.st_dev, current.st_ino):
                        raise PolicyError('Saved browser mount identity changed')
                execution = Execution(launch)
                execution.id = key; execution.spec = metadata['spec']; execution.phase = 'suspended'
                execution.provisioning = False; execution.attached = True
                execution.expires = metadata['expires']; execution.deadline = time.monotonic() + metadata['expires'] - time.time()
                launch.egress_pins = metadata['egress_pins']
                self.service.backend.egress_policies[key] = launch.egress_pins
                self.service.executions[key] = execution
                kept.add(key)
            except Exception:
                if launch: launch.close()
                self.store.delete(key)
                self.service.records.audit('browser_restore_recovery', owner, key, 'failed')
        return kept
