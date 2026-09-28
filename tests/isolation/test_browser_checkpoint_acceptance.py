"""Real gVisor browser hibernation qualification using disposable accounts only."""
import asyncio
import contextlib
import json
import os
from pathlib import Path
import secrets
import struct
import time
import unittest
from test_gvisor_acceptance import GVisorFixture, ENABLED
import test_docker as docker_tests
from fenix_execution.service import Service
from fenix_isolation import private_write


class CDP:
    """Minimal test-only WebSocket client over the authenticated broker relay."""
    def __init__(self, reader, writer):
        self.reader, self.writer, self.sequence = reader, writer, 0

    @classmethod
    async def open(cls, fixture, key, endpoint):
        port, path = endpoint.read_text().splitlines()[:2]
        reader, writer = await asyncio.open_unix_connection(fixture.socket)
        writer.write(json.dumps(dict(operation='connect', account_id=fixture.owners[0],
            credential=fixture.accounts[0]['upstream_token'], id=key, port=int(port))).encode()+b'\n')
        await writer.drain()
        reply = json.loads(await asyncio.wait_for(reader.readline(), 20))
        if not reply.get('ok'): raise AssertionError(reply)
        writer.write((f'GET {path} HTTP/1.1\r\nHost: localhost:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n').encode())
        await writer.drain()
        headers = await asyncio.wait_for(reader.readuntil(b'\r\n\r\n'), 20)
        if not headers.startswith(b'HTTP/1.1 101 '): raise AssertionError(headers)
        return cls(reader, writer)

    async def call(self, method, params=None, session=None):
        self.sequence += 1
        request = dict(id=self.sequence, method=method, params=params or {})
        if session: request['sessionId'] = session
        data = json.dumps(request).encode(); mask = secrets.token_bytes(4)
        header = bytes([0x81, 0x80 | len(data)]) if len(data) < 126 else b'\x81\xfe' + struct.pack('!H', len(data))
        self.writer.write(header+mask+bytes(v ^ mask[i % 4] for i,v in enumerate(data)))
        await self.writer.drain()
        async with asyncio.timeout(30):
            while True:
                first, second = await self.reader.readexactly(2)
                length = second & 127
                if length == 126: length = struct.unpack('!H', await self.reader.readexactly(2))[0]
                if length == 127: length = struct.unpack('!Q', await self.reader.readexactly(8))[0]
                if length > 1024*1024: raise AssertionError('Oversized test response')
                data = await self.reader.readexactly(length)
                if first & 15 == 8: raise AssertionError('Browser closed WebSocket')
                if first & 15 != 1: continue
                value = json.loads(data)
                if value.get('id') == self.sequence:
                    if 'error' in value: raise AssertionError(value['error'])
                    return value['result']

    async def page(self):
        targets = await self.call('Target.getTargets')
        key = next(t['targetId'] for t in targets['targetInfos'] if t['type'] == 'page')
        return (await self.call('Target.attachToTarget', dict(targetId=key, flatten=True)))['sessionId']

    async def evaluate(self, expression, session):
        value = await self.call('Runtime.evaluate', dict(expression=expression, returnByValue=True), session)
        if 'exceptionDetails' in value: raise AssertionError(value['exceptionDetails'])
        return value['result'].get('value')

    async def close(self):
        self.writer.close()
        await self.writer.wait_closed()


@unittest.skipUnless(ENABLED, 'Requires root opt-in and a pinned gVisor runtime')
class BrowserCheckpointAcceptance(GVisorFixture, unittest.IsolatedAsyncioTestCase):
    asyncTearDown = docker_tests.DockerAcceptance.asyncTearDown
    save_registry = docker_tests.DockerAcceptance.save_registry
    request = docker_tests.DockerAcceptance.request

    def configure_fixture(self):
        super().configure_fixture()
        self.config['runner_limits'].update(memory_mib=1536, pids=512, deadline_seconds=300)
        self.config['browser_seccomp'] = os.environ.get('FENIX_BROWSER_SECCOMP', '/opt/fenix/isolation/seccomp-browser.json')
        self.config['checkpoint_key'] = str(self.root / 'checkpoint-key.json')
        self.config['browser_checkpoint_max_bytes'] = 2*1024**3
        private_write(Path(self.config['checkpoint_key']), json.dumps(dict(key=secrets.token_hex(32))).encode())

    async def attach(self, grant):
        reader, writer = await asyncio.open_unix_connection(self.socket)
        self.writers.append(writer)
        writer.write(json.dumps(dict(operation='attach',id=grant['id'],capability=grant['capability'])).encode()+b'\n')
        await writer.drain()
        reply = json.loads(await asyncio.wait_for(reader.readline(), 100))
        self.assertTrue(reply.get('ok'), reply)
        async def pulse():
            try:
                while True:
                    writer.write(b'\x09\0\0\0\0'); await writer.drain(); await asyncio.sleep(2)
            except (ConnectionError, OSError): pass
        self.heartbeats.append(asyncio.create_task(pulse()))

    async def test_real_form_js_memory_and_restart_survive_two_hibernations(self):
        self.assertTrue(self.service.browsers.supported, 'Enable Docker checkpoint support first')
        profile = self.users/self.owners[0]/'state/browser/test/checkpoint'
        downloads = self.users/self.owners[0]/'state/browser/artifacts/checkpoint/downloads'
        for path in (profile, downloads):
            path.mkdir(parents=True); os.chown(path,61001,61001)
        spec = dict(task='browser:checkpoint',kind='browser',argv=['/usr/bin/chromium','--headless=new',
            '--no-first-run','--no-default-browser-check','--disable-background-mode','--disable-sync',
            '--remote-debugging-port=0','--remote-debugging-address=127.0.0.1',
            '--user-data-dir='+str(profile),'about:blank'],cwd=str(profile),workspace=str(profile),timeout=300)
        grant = await self.request(operation='create',spec=spec)
        self.assertTrue(grant.get('ok'),grant); key=grant['id']
        await self.attach(grant)
        endpoint = profile/'DevToolsActivePort'
        for _ in range(300):
            if endpoint.exists() and endpoint.stat().st_size: break
            await asyncio.sleep(.1)
        self.assertTrue(endpoint.exists())
        cdp = await CDP.open(self,key,endpoint)
        session = await cdp.page()
        self.assertTrue(await cdp.evaluate("document.body.innerHTML='<form><input id=answer><button>Send</button></form>'; document.querySelector('input').value='An unsent form'; window.privateDraft={nonce:314159,submitted:0}; document.querySelector('form').onsubmit=e=>{e.preventDefault();window.privateDraft.submitted++}; true",session))
        before = await self.backend.checkpoint_estimate(key,2*1024**3)
        for cycle in range(2):
            await cdp.close()
            started = time.monotonic()
            saved = await self.service.browsers.suspend(self.owners[0],key)
            self.assertTrue(saved['suspended'])
            self.assertFalse((await self.backend.inspect(key))['State']['Running'])
            self.assertEqual(self.records.usage(self.owners[0])['reserved_seconds'],0)
            native = await self.backend.checkpoint_directory(key)
            self.assertFalse(native.exists(), 'Only encrypted snapshot should remain at rest')
            self.assertFalse((await self.request(1,operation='browser_resume',id=key))['ok'])
            if cycle == 0:
                self.monitor.cancel()
                with contextlib.suppress(asyncio.CancelledError): await self.monitor
                self.server.close(); await self.server.wait_closed()
                await self.service.shutdown()
                self.service = Service(self.policy,self.backend,self.records)
                self.assertEqual(await self.service.browsers.recover(),{key})
                self.server = await asyncio.start_unix_server(self.service.handle,path=self.socket)
                self.monitor = asyncio.create_task(self.service.monitor())
            grant = await self.service.resume(self.accounts[0],key)
            await self.attach(grant)
            cdp = await CDP.open(self,key,endpoint);session=await cdp.page()
            value = await cdp.evaluate("({text:document.querySelector('input').value,nonce:window.privateDraft.nonce,submitted:window.privateDraft.submitted})",session)
            self.assertEqual(value,dict(text='An unsent form',nonce=314159,submitted=0))
            print(json.dumps(dict(cycle=cycle+1,roundtrip_seconds=round(time.monotonic()-started,2),memory_estimate_bytes=before,form_retained=True,js_retained=True,duplicate_submissions=0)),flush=True)
        await cdp.close()
        self.assertTrue((await self.request(operation='cancel',id=key))['ok'])
        self.assertTrue(await self.backend.absent(key))


if __name__ == '__main__': unittest.main()
