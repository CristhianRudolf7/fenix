"""Authenticated Unix-socket service with revocable, owner-bound execution leases."""
from __future__ import annotations
import asyncio
import contextlib
from datetime import datetime, timezone
import hashlib
import hmac
import json
import logging
from logging.handlers import RotatingFileHandler
import os
from pathlib import Path
import secrets
import sqlite3
import stat
import struct
import time
import uuid
from fenix_isolation import PolicyError, identifier
from .admission import AdmissionQueue, CapacityUnavailable, available_memory_mib

MAX_CONTROL = 1024 * 1024
MAX_FRAME = 65536


class Records:
    def __init__(self, directory):
        directory = Path(directory)
        directory.mkdir(parents=True, exist_ok=True, mode=0o700)
        if directory.is_symlink() or directory.stat().st_mode & 0o077:
            raise PolicyError("Execution state must be a private host directory")
        self.directory = directory
        self.db = sqlite3.connect(directory / "execution.sqlite")
        os.chmod(directory / "execution.sqlite", 0o600)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.db.execute("PRAGMA synchronous=FULL")
        self.db.execute("PRAGMA max_page_count=16384")
        self.db.execute("PRAGMA journal_size_limit=8388608")
        self.db.execute("CREATE TABLE IF NOT EXISTS executions(id TEXT PRIMARY KEY, owner TEXT NOT NULL, task TEXT NOT NULL, kind TEXT NOT NULL, generation TEXT NOT NULL, status TEXT NOT NULL, created REAL NOT NULL, expires REAL NOT NULL)")
        self.db.execute("CREATE INDEX IF NOT EXISTS executions_owner ON executions(owner,created)")
        self.db.execute("CREATE TABLE IF NOT EXISTS compute_usage(owner TEXT NOT NULL, period TEXT NOT NULL, seconds REAL NOT NULL, PRIMARY KEY(owner,period))")
        self.db.execute("CREATE TABLE IF NOT EXISTS compute_reservations(id TEXT PRIMARY KEY, owner TEXT NOT NULL, period TEXT NOT NULL, seconds REAL NOT NULL)")
        # After a crash the exact stop time may be unknown. Consume outstanding
        # reservations conservatively; do not reset quotas by restarting a service.
        for owner, period, seconds in self.db.execute("SELECT owner,period,SUM(seconds) FROM compute_reservations GROUP BY owner,period").fetchall():
            self.db.execute("INSERT INTO compute_usage VALUES (?,?,?) ON CONFLICT(owner,period) DO UPDATE SET seconds=seconds+excluded.seconds", (owner, period, seconds))
        self.db.execute("DELETE FROM compute_reservations")
        self.db.execute("UPDATE executions SET status='interrupted' WHERE status IN ('created','running')")
        self.db.commit()
        self.log = logging.Logger("fenix.execution." + str(id(self)))
        handler = RotatingFileHandler(directory / "audit.jsonl", maxBytes=4 * 1024 * 1024, backupCount=4)
        os.chmod(directory / "audit.jsonl", 0o600)
        self.log.addHandler(handler)
        self.log.setLevel(logging.INFO)

    def audit(self, action, owner=None, execution=None, outcome="allowed", **extra):
        # Arguments, raw request/response bodies, credentials and URLs are never logged.
        allowed = {k: v for k, v in extra.items() if k in ("kind", "reason", "task", "policy_version")}
        self.log.info(json.dumps(dict(timestamp=time.time(), action=action, owner=owner,
                                      execution=execution, outcome=outcome, **allowed), separators=(",", ":")))

    @staticmethod
    def period():
        return datetime.now(timezone.utc).strftime("%Y-%m")

    def usage(self, owner, maximum=None):
        period = self.period()
        used = self.db.execute("SELECT seconds FROM compute_usage WHERE owner=? AND period=?", (owner, period)).fetchone()
        reserved = self.db.execute("SELECT COALESCE(SUM(seconds),0) FROM compute_reservations WHERE owner=? AND period=?", (owner, period)).fetchone()[0]
        return dict(period=period, used_seconds=used[0] if used else 0, reserved_seconds=reserved, limit_seconds=maximum)

    def insert(self, execution, maximum=None):
        launch = execution.launch
        usage = self.usage(launch.owner, maximum)
        if maximum is not None and usage["used_seconds"] + usage["reserved_seconds"] + launch.timeout > maximum:
            raise PolicyError("Account monthly execution budget exhausted; use a shorter timeout or update its plan")
        with self.db:
            self.db.execute("INSERT INTO compute_reservations VALUES (?,?,?,?)",
                            (execution.id, launch.owner, usage["period"], launch.timeout))
            self.db.execute("INSERT INTO executions VALUES (?,?,?,?,?,?,?,?)",
                        (execution.id, launch.owner, launch.task, launch.kind, execution.id,
                         "created", time.time(), execution.expires))

    def reserve_resume(self, execution, maximum=None):
        usage = self.usage(execution.launch.owner, maximum)
        if maximum is not None and usage['used_seconds'] + usage['reserved_seconds'] + execution.launch.timeout > maximum:
            raise PolicyError('Account execution budget exhausted')
        with self.db:
            self.db.execute('INSERT INTO compute_reservations VALUES (?,?,?,?)',
                            (execution.id, execution.launch.owner, usage['period'], execution.launch.timeout))

    def settle(self, execution):
        row = self.db.execute("SELECT owner,period,seconds FROM compute_reservations WHERE id=?", (execution.id,)).fetchone()
        if row is None:
            return
        owner, period, maximum = row
        elapsed = min(maximum, max(0, time.monotonic() - execution.started))
        with self.db:
            self.db.execute("INSERT INTO compute_usage VALUES (?,?,?) ON CONFLICT(owner,period) DO UPDATE SET seconds=seconds+excluded.seconds", (owner, period, elapsed))
            self.db.execute("DELETE FROM compute_reservations WHERE id=?", (execution.id,))
            self.db.execute("DELETE FROM compute_usage WHERE period < ?", (str(datetime.now(timezone.utc).year - 1) + "-01",))

    def status(self, execution, status):
        self.db.execute("UPDATE executions SET status=? WHERE id=?", (status, execution))
        # Bounded metadata retention. It is not an indefinite task transcript.
        self.db.execute("DELETE FROM executions WHERE status NOT IN ('created','running','suspended') AND created < ?", (time.time() - 7 * 86400,))
        self.db.execute("DELETE FROM executions WHERE id IN (SELECT id FROM executions WHERE status NOT IN ('created','running','suspended') ORDER BY created DESC LIMIT -1 OFFSET 10000)")
        self.db.commit()

    def owned(self, owner, execution):
        return self.db.execute("SELECT id,task,kind,status,created,expires FROM executions WHERE owner=? AND id=?", (owner, execution)).fetchone()

    def list(self, owner):
        columns = ("id", "task", "kind", "status", "created", "expires")
        return [dict(zip(columns, row)) for row in self.db.execute(
            "SELECT id,task,kind,status,created,expires FROM executions WHERE owner=? ORDER BY created DESC LIMIT 100", (owner,))]


class Execution:
    def __init__(self, launch):
        self.id = str(uuid.uuid4())
        self.launch = launch
        self.capability = secrets.token_urlsafe(32)
        self.capability_hash = hashlib.sha256(self.capability.encode()).digest()
        self.expires = time.time() + launch.timeout
        self.started = time.monotonic()
        self.deadline = self.started + launch.timeout
        self.attach_deadline = time.monotonic() + 30
        self.attached = False
        self.process = None
        self.cancelled = asyncio.Event()
        self.lock = asyncio.Lock()
        self.cleaned = False
        self.output_bytes = 0
        self.provisioning = True
        self.phase = 'created'
        self.generation = 0
        self.resume_pending = False
        self.raw_checkpoint = False
        self.checkpoint_metadata = None
        self.disconnect = asyncio.Event()
        self.tunnels = set()
        self.spec = None


class Service:
    def __init__(self, policy, backend, records):
        self.policy, self.backend, self.records = policy, backend, records
        self.executions = {}
        self.admission = asyncio.Lock()
        self.connections = 0
        self.maximum_connections = 128
        self.stopping = False
        self.snapshots = None
        self.broker = None
        self.queue = AdmissionQueue(self)
        from .browser_lifecycle import BrowserLifecycle
        self.browsers = BrowserLifecycle(self)

    async def create(self, account, spec):
        launch = self.policy.launch(account, spec)
        execution = Execution(launch)
        execution.spec = spec
        try:
            async with self.admission:
                if self.stopping:
                    raise PolicyError("Execution service is stopping")
                self.check_capacity(launch)
                owned = [e for e in self.executions.values() if e.launch.owner == launch.owner]
                for other in owned:
                    if (launch.kind == "browser" and other.launch.kind == "browser"
                            and launch.cwd == other.launch.cwd):
                        raise PolicyError("This browser profile is already running")
                    if other.launch.task == launch.task:
                        continue
                    if any(not read_a and not read_b and (a.path.is_relative_to(b.path) or b.path.is_relative_to(a.path))
                           for a, read_a in launch.mounts for b, read_b in other.launch.mounts):
                        raise PolicyError("Another task holds this writable workspace; use a separate worktree")
                if launch.kind == "browser":
                    # Admission owns this exact profile exclusively. Previous
                    # runners have been removed before their lease is released.
                    # Chromium stores container hostname/PID links which cannot
                    # survive a new container; remove only these transient links.
                    profile = next(m for m, _ in launch.mounts if m.path == launch.cwd)
                    names = ("SingletonLock", "SingletonSocket", "SingletonCookie")
                    stale = []
                    for name in names:
                        try:
                            mode = os.stat(name, dir_fd=profile.fd, follow_symlinks=False).st_mode
                        except FileNotFoundError:
                            continue
                        if not stat.S_ISLNK(mode):
                            raise PolicyError("Unexpected browser lock file; profile was not changed")
                        stale.append(name)
                    for name in stale:
                        with contextlib.suppress(FileNotFoundError):
                            os.unlink(name, dir_fd=profile.fd)
                self.records.insert(execution, self.policy.accounts[launch.owner].get("execution_seconds_per_month"))
                self.executions[execution.id] = execution
            async with execution.lock:
                await self.backend.create(execution.id, launch)
                execution.provisioning = False
                execution.attach_deadline = time.monotonic() + 30
            self.records.audit("create", launch.owner, execution.id, kind=launch.kind, task=launch.task, policy_version=1)
            capability = execution.capability
            execution.capability = None
            return {"id": execution.id, "capability": capability, "timeout": launch.timeout, "generation": execution.id}
        except BaseException:
            execution.provisioning = False
            if execution.id in self.executions:
                await self.destroy(execution, "failed")
            else:
                launch.close()
            raise

    def check_capacity(self, launch, exclude=None):
        if self.stopping:
            raise PolicyError('Execution service is stopping')
        active = [e for e in self.executions.values() if e.id != exclude and e.phase != 'suspended']
        owned = [e for e in active if e.launch.owner == launch.owner]
        config = self.policy.config
        if len(owned) >= launch.account_limits.concurrent or len(active) >= config.get('max_executions', 32):
            raise CapacityUnavailable('Execution admission quota reached')
        requested = self.policy.reservation(launch)
        if sum(self.policy.reservation(e.launch) for e in active) + requested > config.get('max_runner_memory_mib', 4096):
            raise CapacityUnavailable('Host execution memory reservation exhausted')
        if sum(self.policy.reservation(e.launch) for e in owned) + requested > launch.account_limits.memory_mib:
            raise CapacityUnavailable('Account execution memory reservation exhausted')
        provisioning = sum(self.policy.reservation(e.launch) for e in active if e.provisioning or e.phase in ('created', 'resuming'))
        if available_memory_mib() < config.get('host_memory_reserve_mib', 1024) + provisioning + requested:
            raise CapacityUnavailable('Waiting for available host memory')

    async def resume(self, account, key):
        return await self.browsers.resume(account, key)

    def owned(self, owner, execution):
        identifier(execution)
        value = self.executions.get(execution)
        if not value or value.launch.owner != owner or value.cancelled.is_set():
            raise PolicyError("Execution not found")
        if not self.policy.active(owner, value.launch.fingerprint) or value.deadline <= time.monotonic():
            raise PolicyError("Execution revoked or expired")
        return value

    async def destroy(self, execution, status="cancelled", attachment_generation=None):
        async with execution.lock:
            if attachment_generation is not None and (execution.generation != attachment_generation or execution.phase in ('suspending', 'suspended', 'resuming')):
                return
            if execution.cleaned:
                return
            execution.cancelled.set()
            failed = False
            try:
                await self.backend.destroy(execution.id)
            except Exception:
                # Creation may have failed before the container existed. Confirm
                # absence rather than claiming successful cleanup on an API error.
                try:
                    await self.backend.inspect(execution.id)
                    failed = True
                except Exception:
                    # A backend-wide outage is retried by the monitor, not ignored.
                    failed = True
                    with contextlib.suppress(Exception):
                        failed = not await self.backend.absent(execution.id)
            if failed:
                self.records.audit("cleanup", execution.launch.owner, execution.id, "failed", reason="backend unavailable")
                return
            try:
                await self.backend.cleanup_mounts(execution.id)
            except Exception:
                self.records.audit("mount_cleanup", execution.launch.owner, execution.id, "failed")
                return
            if execution.launch.profile != "offline":
                try:
                    await self.backend.remove_network(execution.id)
                except Exception:
                    self.records.audit("network_cleanup", execution.launch.owner, execution.id, "failed")
                    return  # Keep ownership and retry instead of forgetting orphan resources.
            if execution.process and execution.process.returncode is None:
                execution.process.kill()
                await execution.process.wait()
            if self.browsers.store:
                async with self.browsers.lock:
                    await asyncio.to_thread(self.browsers.store.delete, execution.id)
            execution.launch.close()
            execution.cleaned = True
            self.executions.pop(execution.id, None)
            self.records.settle(execution)
            self.records.status(execution.id, status)
            self.records.audit("destroy", execution.launch.owner, execution.id, reason=status)

    async def monitor(self):
        while not self.stopping:
            await self.queue.tick()
            for execution in list(self.executions.values()):
                if execution.provisioning or execution.phase == 'suspending':
                    continue
                try:
                    expired = execution.deadline <= time.monotonic() or (not execution.attached and time.monotonic() > execution.attach_deadline)
                    if expired and execution.phase == 'resuming' and not execution.attached:
                        self.browsers.release_unattached(execution)
                        continue
                    if expired or execution.cancelled.is_set() or not self.policy.active(execution.launch.owner, execution.launch.fingerprint):
                        await self.destroy(execution, "expired" if expired else "revoked")
                        continue
                    if execution.phase not in ('suspended', 'resuming'):
                        await self.backend.guard_check(execution.id, execution.launch.profile)
                except Exception:
                    # On policy drift, terminate before any reconciliation attempt.
                    await self.destroy(execution, "policy_failed")
            try:
                await self.backend.watch_controllers(self.policy)
            except Exception:
                self.records.audit("controller_guard", outcome="failed")
            await asyncio.sleep(1)

    async def shutdown(self):
        self.stopping = True
        await self.queue.shutdown()
        async def close_active(execution):
            # A checkpoint already in flight may finish during shutdown. Wait for
            # its commit before deciding whether this execution should be stopped.
            async with execution.lock:
                suspended = execution.phase == 'suspended'
            if not suspended:
                await self.destroy(execution, 'interrupted')
        await asyncio.gather(*(close_active(e) for e in list(self.executions.values())), return_exceptions=True)
        for execution in self.executions.values():
            execution.launch.close()
        if self.browsers.store:
            self.browsers.store.db.close()

    async def reply(self, writer, **values):
        encoded = json.dumps(values, separators=(",", ":")).encode()
        if len(encoded) > MAX_CONTROL:
            raise PolicyError("Control response exceeds limit")
        writer.write(encoded + b"\n")
        await asyncio.wait_for(writer.drain(), 10)

    async def handle(self, reader, writer):
        if self.connections >= self.maximum_connections or self.stopping:
            writer.close()
            return
        self.connections += 1
        authenticated = None
        try:
            raw = await asyncio.wait_for(reader.readline(), 10)
            if len(raw) > MAX_CONTROL or not raw.endswith(b"\n"):
                raise PolicyError("Invalid control request")
            request = json.loads(raw)
            operation = request.get("operation")
            if operation == "attach":
                await self.attach(request, reader, writer)
                return
            account = self.policy.authenticate(request)
            authenticated = account["id"]
            if operation == "health":
                # Successful authentication also verifies current registry readability.
                await self.reply(writer, ok=True, protocol=1, backend=getattr(type(self.backend), "kind", "oci"), runtime=self.backend.runtime,
                                 image=self.backend.image, snapshots=self.snapshots is not None,
                                 platform=getattr(type(self.backend), "platform", None),
                                 persistent_environments=self.policy.config.get("persistent_environments", False), ram_snapshots=self.browsers.supported, execution_queue=True,
                                 profiles=["offline", "development", "web"], policy_version=1)
            elif operation == "create":
                result = self.queue.submit(account, request['spec']) if request.get('enqueue') else await self.create(account, request['spec'])
                await self.reply(writer, ok=True, **result)
            elif operation == 'admission_poll':
                result = await self.queue.poll(authenticated, request['ticket'])
                await self.reply(writer, ok=True, **result)
            elif operation == 'admission_cancel':
                await self.queue.cancel(authenticated, request['ticket'])
                await self.reply(writer, ok=True)
            elif operation == 'browser_suspend':
                result = await self.browsers.suspend(authenticated, request['id'])
                await self.reply(writer, ok=True, **result)
            elif operation == 'browser_resume':
                self.owned(authenticated, request['id'])
                result = self.queue.submit(account, request['id'], 'resume')
                await self.reply(writer, ok=True, **result)
            elif operation == "list":
                await self.reply(writer, ok=True, executions=self.records.list(authenticated),
                                 usage=self.records.usage(authenticated, self.policy.accounts[authenticated].get("execution_seconds_per_month")))
            elif operation in ("cancel", "release", "status"):
                row = self.records.owned(authenticated, request["id"])
                if row is None:
                    raise PolicyError("Execution not found")
                execution = self.executions.get(request["id"])
                if operation == 'release' and execution and execution.phase == 'suspended':
                    pass  # Dropping a failed restore client must not discard its saved page.
                elif operation == 'release' and execution and execution.phase == 'resuming' and not execution.attached:
                    self.browsers.release_unattached(execution)
                elif operation in ('cancel', 'release') and execution:
                    await self.destroy(self.owned(authenticated, request["id"]))
                    if not execution.cleaned:
                        raise PolicyError("Cancellation is pending infrastructure recovery")
                await self.reply(writer, ok=True, status=execution.phase if execution else self.records.owned(authenticated, request["id"])[3])
            elif operation == "connect":
                execution = self.owned(authenticated, request["id"])
                port = request.get("port")
                if type(port) is not int or not 1024 <= port <= 65535 or execution.launch.profile == "offline":
                    raise PolicyError("Runner port unavailable")
                if execution.phase != 'running':
                    raise PolicyError('Browser execution is not running')
                task = asyncio.current_task()
                execution.tunnels.add(task)
                try:
                    upstream_reader, upstream_writer = await self.backend.connect(execution.id, port)
                    await self.reply(writer, ok=True)
                    await self.tunnel(execution, reader, writer, upstream_reader, upstream_writer)
                finally:
                    execution.tunnels.discard(task)
            elif operation == "resize":
                execution = self.owned(authenticated, request["id"])
                rows, columns = request.get("rows"), request.get("columns")
                if not execution.launch.tty or any(type(value) is not int or not 1 <= value <= 500 for value in (rows, columns)):
                    raise PolicyError("Invalid terminal dimensions")
                await self.backend.resize(execution.id, rows, columns)
                await self.reply(writer, ok=True)
            elif operation in ("checkpoint", "restore", "clone", "delete_checkpoint", "list_checkpoints"):
                if self.snapshots is None:
                    raise PolicyError("Encrypted checkpoint storage is not configured")
                async with self.admission:
                    if operation in ("checkpoint", "restore", "clone"):
                        target = Path(request["workspace"] if operation == "checkpoint" else request["destination"])
                        if any(e.launch.owner == authenticated and any(not readonly and (mount.path.is_relative_to(target) or target.is_relative_to(mount.path))
                               for mount, readonly in e.launch.mounts) for e in self.executions.values()):
                            raise PolicyError("Stop writable executions in this project before checkpoint operations")
                    result = await asyncio.to_thread(self.snapshots.dispatch, account, request)
                self.records.audit(operation, authenticated)
                await self.reply(writer, ok=True, **result)
            elif operation in ("integrations", "integration_request"):
                if self.broker is None:
                    raise PolicyError("Integration broker is not configured")
                result = await self.broker.dispatch(account, request)
                self.records.audit(operation, authenticated)
                await self.reply(writer, ok=True, **result)
            else:
                raise PolicyError("Unsupported execution operation")
        except (Exception, asyncio.CancelledError) as error:
            self.records.audit("request", authenticated, outcome="denied", reason=type(error).__name__)
            with contextlib.suppress(Exception):
                await self.reply(writer, ok=False, error=str(error) if isinstance(error, PolicyError) else "Execution service rejected the operation")
        finally:
            self.connections -= 1
            writer.close()
            with contextlib.suppress(Exception):
                await writer.wait_closed()

    async def attach(self, request, reader, writer):
        execution = self.executions.get(request.get("id"))
        capability = request.get("capability", "")
        if not execution or not isinstance(capability, str) or not hmac.compare_digest(hashlib.sha256(capability.encode()).digest(), execution.capability_hash):
            raise PolicyError("Invalid execution capability")
        self.owned(execution.launch.owner, execution.id)
        async with execution.lock:
            if execution.attached or execution.cleaned:
                raise PolicyError("Execution capability already consumed")
            execution.attached = True
            restoring = execution.resume_pending
            try:
                await self.backend.guard_check(execution.id, execution.launch.profile)
                execution.process = await (self.backend.attach_restored(execution.id) if restoring else self.backend.attach(execution.id))
                # Sandbox startup can take time. Re-check authorization and the
                # deadline before the adapter releases its startup gate.
                self.owned(execution.launch.owner, execution.id)
                if restoring:
                    await self.browsers.restored(execution)
                else:
                    await self.backend.release(execution.id, execution.process)
                execution.phase = 'running'
                self.records.status(execution.id, "running")
            except BaseException:
                if restoring:
                    with contextlib.suppress(Exception):
                        info = await self.backend.inspect(execution.id)
                        if info['State']['Running'] is False and execution.raw_checkpoint:
                            self.browsers.release_unattached(execution)
                    if execution.phase == 'suspended':
                        raise
                execution.cancelled.set()  # Monitor retries complete cleanup.
                raise
        process = execution.process
        generation = execution.generation
        try:
            await self.reply(writer, ok=True)
        except BaseException:
            await self.destroy(execution, "interrupted")
            raise
        write_lock = asyncio.Lock()

        async def frame(kind, payload):
            async with write_lock:
                writer.write(bytes([kind]) + struct.pack("!I", len(payload)) + payload)
                await asyncio.wait_for(writer.drain(), 10)

        async def output(stream, kind):
            while chunk := await stream.read(16384):
                execution.output_bytes += len(chunk)
                if execution.output_bytes > execution.launch.limits.log_mib * 1024 * 1024:
                    raise PolicyError("Execution output quota reached")
                await frame(kind, chunk)

        async def input_stream():
            while True:
                header = await asyncio.wait_for(reader.readexactly(5), 10)
                kind, size = header[0], struct.unpack("!I", header[1:])[0]
                if size > MAX_FRAME:
                    raise PolicyError("Input frame exceeds limit")
                payload = await asyncio.wait_for(reader.readexactly(size), 10)
                if kind == 0:
                    if not process.stdin.is_closing():
                        process.stdin.write(payload)
                        await asyncio.wait_for(process.stdin.drain(), 10)
                elif kind == 5 and not payload:
                    process.stdin.close()
                elif kind != 9 or payload:
                    raise PolicyError("Invalid input frame")

        outputs = [asyncio.create_task(output(process.stdout, 1)), asyncio.create_task(output(process.stderr, 2))]
        inputs = asyncio.create_task(input_stream())
        cancelled = asyncio.create_task(execution.cancelled.wait())
        exited = asyncio.create_task(process.wait())
        status = "cancelled"
        try:
            done, _ = await asyncio.wait([inputs, cancelled, exited, *outputs], return_when=asyncio.FIRST_COMPLETED)
            # stdout can close before the process exits. Only errors should cancel it.
            while exited not in done and inputs not in done and cancelled not in done and all(t.exception() is None for t in done):
                pending = [t for t in [inputs, cancelled, exited, *outputs] if not t.done()]
                done, _ = await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)
            if exited in done and not execution.cancelled.is_set():
                await asyncio.gather(*outputs)
                info = await self.backend.inspect(execution.id)
                code = info["State"]["ExitCode"]
                await frame(3, json.dumps(code).encode())
                status = "completed"
            else:
                with contextlib.suppress(Exception):
                    await frame(4, b"")
        finally:
            for task in [inputs, cancelled, exited, *outputs]:
                task.cancel()
            await asyncio.gather(inputs, cancelled, exited, *outputs, return_exceptions=True)
            await self.destroy(execution, status, attachment_generation=generation)

    async def tunnel(self, execution, reader, writer, upstream_reader, upstream_writer):
        async def copy(source, target):
            while chunk := await source.read(65536):
                target.write(chunk)
                await asyncio.wait_for(target.drain(), 15)
        tasks = [asyncio.create_task(copy(reader, upstream_writer)), asyncio.create_task(copy(upstream_reader, writer)),
                 asyncio.create_task(execution.cancelled.wait()), asyncio.create_task(execution.disconnect.wait())]
        try:
            await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
        finally:
            for task in tasks:
                task.cancel()
            await asyncio.gather(*tasks, return_exceptions=True)
            upstream_writer.close()
            with contextlib.suppress(Exception):
                await upstream_writer.wait_closed()
