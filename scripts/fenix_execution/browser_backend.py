"""gVisor-only Docker checkpoint adapter; never falls back to a weaker runtime."""
import asyncio
import json
import re
from pathlib import Path
from fenix_isolation import PolicyError, private_json, private_write


class BrowserCheckpointBackend:
    async def checkpoint_estimate(self, execution, maximum):
        record = private_json(self.mount_dir / execution / 'container.json')
        group = record.get('sandbox_cgroup', '')
        prefix = '/fenixacct.slice/fenixacct-' + record['owner'].replace('-', '') + '.slice/'
        if not group.startswith(prefix) or '..' in Path(group).parts:
            raise PolicyError('Browser memory accounting is unavailable')
        current = int((Path('/sys/fs/cgroup') / group.lstrip('/') / 'memory.current').read_text())
        return min(maximum, max(64 * 1024**2, current * 2))

    async def checkpoint_directory(self, execution):
        info = await self.inspect(execution)
        container = info['Id']
        if not re.fullmatch(r'[0-9a-f]{64}', container) or info['HostConfig']['Runtime'] != self.runtime:
            raise PolicyError('Unexpected browser checkpoint container')
        return self.docker_root / 'containers' / container / 'checkpoints' / 'fenix-browser'

    async def checkpoint_browser(self, execution):
        if not self.browser_checkpoints:
            raise PolicyError('Browser hibernation requires Docker checkpoint support and gVisor')
        self.bundle.unchanged()
        await self.command('docker', 'checkpoint', 'create', 'fenix-run-' + execution, 'fenix-browser', timeout=120)
        info = await self.inspect(execution)
        if info['State']['Running']:
            raise PolicyError('Browser checkpoint did not stop its processes')
        return await self.checkpoint_directory(execution)

    async def attach_restored(self, execution):
        self.bundle.unchanged()
        self.bundle.check_registration(json.loads(await self.command('docker', 'info', '--format', '{{json .}}')))
        process = await asyncio.create_subprocess_exec('docker', 'start', '--checkpoint', 'fenix-browser',
            '--attach', '--interactive', 'fenix-run-' + execution,
            stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
        try:
            async with asyncio.timeout(60):
                while True:
                    info = await self.inspect(execution)
                    if info['State']['Running']:
                        break
                    if process.returncode is not None:
                        raise PolicyError('Browser checkpoint restore failed')
                    await asyncio.sleep(.1)
            from .gvisor import RUNTIME, sentry_cgroup
            if info['HostConfig']['Runtime'] != RUNTIME:
                raise PolicyError('Restored browser isolation changed')
            record_path = self.mount_dir / execution / 'container.json'
            record = private_json(record_path)
            record['sandbox_cgroup'] = await asyncio.to_thread(sentry_cgroup, record['owner'], record['container'], info['State']['Pid'], self.bundle.sentry)
            private_write(record_path, json.dumps(record).encode())
            self.verified.add(execution)
            return process
        except BaseException:
            if process.returncode is None: process.kill()
            await process.wait()
            raise
