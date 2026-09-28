"""Bounded, owner-fair admission tickets. Waiting never reserves runner memory."""
import asyncio
import time
import uuid
from collections import OrderedDict
from fenix_isolation import PolicyError


class CapacityUnavailable(PolicyError):
    pass


def available_memory_mib():
    with open('/proc/meminfo', encoding='ascii') as source:
        for line in source:
            if line.startswith('MemAvailable:'):
                return int(line.split()[1]) // 1024
    raise PolicyError('Host memory availability cannot be measured')


class AdmissionQueue:
    def __init__(self, service):
        self.service = service
        self.entries = OrderedDict()
        self.last_owner = None
        self.lock = asyncio.Lock()

    def submit(self, account, payload, operation='create'):
        owner = account['id']
        config = self.service.policy.config
        if len(self.entries) >= config.get('max_queued_executions', 128) or sum(e['owner'] == owner for e in self.entries.values()) >= config.get('max_queued_per_account', 8):
            raise PolicyError('Execution waiting queue is full')
        key = str(uuid.uuid4())
        self.entries[key] = dict(owner=owner, account=account, payload=payload, operation=operation,
                                 touched=time.monotonic(), created=time.monotonic(), result=None, error=None, job=None)
        return dict(queued=True, ticket=key, retry_after_ms=1000)

    async def poll(self, owner, key):
        entry = self.entries.get(key)
        if entry is None or entry['owner'] != owner:
            raise PolicyError('Admission ticket not found')
        entry['touched'] = time.monotonic()
        if entry['error']:
            self.entries.pop(key)
            raise PolicyError(entry['error'])
        if entry['result']:
            self.entries.pop(key)
            return entry['result']
        return dict(queued=True, ticket=key, retry_after_ms=1000)

    async def cancel(self, owner, key):
        entry = self.entries.get(key)
        if entry is None:
            return
        if entry['owner'] != owner:
            raise PolicyError('Admission ticket not found')
        self.entries.pop(key)
        if entry['job'] and not entry['job'].done():
            entry['job'].cancel()
            await asyncio.gather(entry['job'], return_exceptions=True)
        if entry['result']:
            execution = self.service.executions.get(entry['result']['id'])
            if execution:
                if entry['operation'] == 'resume' and execution.phase == 'resuming' and not execution.attached:
                    self.service.browsers.release_unattached(execution)
                else:
                    await self.service.destroy(execution)

    async def attempt(self, entry):
        try:
            account = self.service.policy.authenticate(dict(account_id=entry['owner'],
                credential=entry['account']['upstream_token'], auth_epoch=entry['account'].get('auth_epoch', 0)))
            if entry['operation'] == 'resume':
                entry['result'] = await self.service.resume(account, entry['payload'])
            else:
                entry['result'] = await self.service.create(account, entry['payload'])
        except CapacityUnavailable:
            pass
        except PolicyError as error:
            entry['error'] = str(error)
        except asyncio.CancelledError:
            raise
        except Exception:
            entry['error'] = 'Execution could not be started'

    async def tick(self):
        async with self.lock:
            now = time.monotonic()
            for key, entry in list(self.entries.items()):
                if now - entry['touched'] > 20 or now - entry['created'] > self.service.policy.config.get('admission_wait_seconds', 900):
                    await self.cancel(entry['owner'], key)
            pending = [e for e in self.entries.values() if not e['result'] and not e['error'] and (not e['job'] or e['job'].done())]
            # One request per owner per sweep; rotate the starting owner.
            owners = list(dict.fromkeys(e['owner'] for e in pending))
            if self.last_owner in owners:
                at = owners.index(self.last_owner) + 1
                owners = owners[at:] + owners[:at]
            for owner in owners:
                entry = next(e for e in pending if e['owner'] == owner)
                entry['job'] = asyncio.create_task(self.attempt(entry))
            if owners: self.last_owner = owners[0]

    async def shutdown(self):
        for key, entry in list(self.entries.items()):
            await self.cancel(entry['owner'], key)
