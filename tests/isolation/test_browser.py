"""Opt-in Chromium sandbox/CDP test through the real execution service."""
import asyncio
import contextlib
import dataclasses
import json
import os
from pathlib import Path
import shutil
import subprocess
import unittest
import uuid
import test_docker as docker_tests

@unittest.skipUnless(docker_tests.ENABLED,"Requires explicit root opt-in and Chromium/AppArmor in the test image")
class BrowserAcceptance(unittest.IsolatedAsyncioTestCase):
    asyncSetUp=docker_tests.DockerAcceptance.asyncSetUp
    asyncTearDown=docker_tests.DockerAcceptance.asyncTearDown
    save_registry=docker_tests.DockerAcceptance.save_registry
    request=docker_tests.DockerAcceptance.request

    async def test_browser_keeps_inner_sandbox_and_cdp_is_owner_scoped(self):
        repo=Path(__file__).resolve().parents[2]
        profile_name="fenix-browser-test-"+uuid.uuid4().hex
        apparmor=self.root/"browser.apparmor"
        apparmor.write_text((repo/"packaging/linux/fenix-browser.apparmor").read_text().replace("fenix-browser-v1",profile_name))
        seccomp=self.root/"browser.json"
        shutil.copyfile(repo/"packaging/linux/seccomp-browser.json",seccomp)
        subprocess.run(["apparmor_parser","-r",str(apparmor)],check=True)
        self.config.update(browser_apparmor=profile_name,browser_seccomp=str(seccomp))
        self.policy.runner_limits=dataclasses.replace(self.policy.runner_limits,memory_mib=512,pids=128)
        root=self.users/self.owners[0]
        profile=root/"state/browser/test/fixture"
        downloads=root/"state/browser/artifacts/fixture/downloads"
        for directory in (profile,downloads):
            directory.mkdir(parents=True);os.chown(directory,61001,61001)
        execution=None
        heartbeat=None
        try:
            grant=await self.request(operation="create",spec=dict(task="browser:fixture",kind="browser",argv=["/untrusted/old-wrapper","--headless=new","--disable-gpu","--disable-dev-shm-usage","--no-first-run","--remote-debugging-port=0","--remote-debugging-address=127.0.0.1","--user-data-dir="+str(profile),"about:blank"],cwd=str(profile),workspace=str(profile),timeout=60))
            self.assertTrue(grant["ok"],grant);execution=grant["id"]
            reader,writer=await asyncio.open_unix_connection(self.socket);self.writers.append(writer)
            writer.write(json.dumps(dict(operation="attach",id=execution,capability=grant["capability"])).encode()+b"\n");await writer.drain()
            self.assertTrue(json.loads(await reader.readline())["ok"])
            async def pulse():
                while True:
                    writer.write(b"\x09\0\0\0\0");await writer.drain();await asyncio.sleep(2)
            heartbeat=asyncio.create_task(pulse())
            endpoint=profile/"DevToolsActivePort"
            for _ in range(300):
                if endpoint.exists() and endpoint.read_text().strip(): break
                await asyncio.sleep(.1)
            if not endpoint.exists():
                diagnostic = ""
                with contextlib.suppress(asyncio.TimeoutError, ConnectionError):
                    diagnostic, code = await asyncio.wait_for(docker_tests.DockerAcceptance.output(self, reader), 3)
                    diagnostic = f"exit={code}: " + diagnostic[:3000]
                self.fail("Chromium did not start with its inner sandbox: " + diagnostic)
            port=int(endpoint.read_text().splitlines()[0])
            denied=await self.request(1,operation="connect",id=execution,port=port)
            self.assertFalse(denied["ok"])
            control,client=await asyncio.open_unix_connection(self.socket)
            client.write(json.dumps(dict(operation="connect",account_id=self.owners[0],credential=self.accounts[0]["upstream_token"],id=execution,port=port)).encode()+b"\n");await client.drain()
            self.assertTrue(json.loads(await control.readline())["ok"])
            client.write(b"GET /json/version HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");await client.drain()
            headers=await asyncio.wait_for(control.readuntil(b"\r\n\r\n"),10)
            length=next(int(line.split(b":",1)[1]) for line in headers.split(b"\r\n") if line.lower().startswith(b"content-length:"))
            response=headers+await asyncio.wait_for(control.readexactly(length),10)
            self.assertIn(b"webSocketDebuggerUrl",response);self.assertIn(b"200 OK",response)
            client.close();await client.wait_closed()
            info=await self.backend.inspect(execution)
            if self.backend.kind in ("microvm", "gvisor"):
                self.assertEqual(info["Path"],"/usr/bin/python3")
                self.assertIn("/usr/bin/chromium",info["Args"])
            else:
                self.assertEqual(info["Path"],"/usr/bin/chromium")
            self.assertNotIn("--no-sandbox",info["Args"])
            self.assertFalse(info["HostConfig"]["Privileged"])
            self.assertEqual(info["HostConfig"]["CapDrop"],["ALL"])
            self.assertFalse(info["HostConfig"]["PortBindings"])
            self.assertNotIn(str(root/"state"),[mount["Destination"] for mount in info["Mounts"]])
        finally:
            if heartbeat:
                heartbeat.cancel()
                with contextlib.suppress(asyncio.CancelledError,ConnectionError): await heartbeat
            if execution and execution in self.service.executions:
                await self.service.destroy(self.service.executions[execution])
            await self.backend.cleanup_orphans()
            subprocess.run(["apparmor_parser","-R",str(apparmor)],check=True)

if __name__=="__main__": unittest.main()
