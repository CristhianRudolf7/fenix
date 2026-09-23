"""Opt-in, bounded real Linux/Docker acceptance tests using disposable accounts only.
Run as root with FENIX_DOCKER_TESTS=1 and a local FENIX_TEST_IMAGE.
No image pull, production controller, existing network or existing account is touched.
"""
import asyncio
import contextlib
import dataclasses
import json
import os
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch
import uuid

sys.path.insert(0,str(Path(__file__).resolve().parents[2]/"scripts"))
from fenix_isolation import private_write, PolicyError
from fenix_execution.policy import Policy
from fenix_execution.docker import Docker
from fenix_execution.service import Records,Service

ENABLED=os.environ.get("FENIX_DOCKER_TESTS")=="1" and os.geteuid()==0

@unittest.skipUnless(ENABLED,"Requires explicit root opt-in and disposable Docker resources")
class DockerAcceptance(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.root=Path(tempfile.mkdtemp(prefix="fenix-isolation-test-",dir="/var/tmp"))
        self.users=self.root/"users";self.users.mkdir()
        self.owners=[str(uuid.uuid4()),str(uuid.uuid4())]
        self.accounts=[]
        identities={}
        for i,owner in enumerate(self.owners):
            volume=self.users/owner;volume.mkdir()
            subprocess.run(["mount","-t","tmpfs","-o","size=64m,nodev,nosuid,mode=700","fenix-test",str(volume)],check=True)
            os.chown(volume,61001+i,61001+i)
            for name in ("workspace","projects","state","skills"):
                directory=volume/name;directory.mkdir(mode=0o700);os.chown(directory,61001+i,61001+i)
            (volume/"state"/"synthetic-secret").write_text("CONTROLLER_ONLY_TEST_SECRET")
            identities[owner]=dict(uid=61001+i,gid=61001+i,disk_bytes=64*1024*1024)
            self.accounts.append(dict(id=owner,email=f"test{i}@example.test",upstream=f"http://127.0.0.1:{51001+i}",upstream_token=str(i)*64))
        self.config=dict(backend="oci", runtime="runc", max_runner_memory_mib=512,host_memory_reserve_mib=256,controller_limits=dict(memory_mib=128,tmp_mib=16),registry=str(self.root/"accounts.json"),users_root=str(self.users),accounts=identities,
                         state_dir=str(self.root/"service"),image=os.environ.get("FENIX_TEST_IMAGE","fenix-tenant:local"),instance="test"+uuid.uuid4().hex[:12],
                         runner_limits=dict(memory_mib=256,tmp_mib=32,pids=64,cpus=.5,deadline_seconds=60),
                         account_limits=dict(memory_mib=512,tmp_mib=32,pids=128,cpus=1,concurrent=3))
        self.save_registry()
        getattr(self, "configure_fixture", lambda: None)()
        self.policy=Policy(self.config);self.backend=getattr(type(self), "backend_class", Docker)(self.config);self.records=Records(self.root/"service")
        self.service=Service(self.policy,self.backend,self.records)
        self.socket=self.root/"control.sock"
        self.server=await asyncio.start_unix_server(self.service.handle,path=self.socket)
        self.monitor=asyncio.create_task(self.service.monitor())
        self.writers=[]
        self.heartbeats=[]

    def save_registry(self):
        private_write(Path(self.config["registry"]),json.dumps(dict(accounts=self.accounts)).encode())

    async def asyncTearDown(self):
        for heartbeat in self.heartbeats: heartbeat.cancel()
        await asyncio.gather(*self.heartbeats,return_exceptions=True)
        for writer in self.writers: writer.close()
        self.monitor.cancel()
        with contextlib.suppress(asyncio.CancelledError): await self.monitor
        self.server.close();await self.server.wait_closed()
        await self.service.shutdown()
        await self.backend.cleanup_orphans()
        self.records.db.close()
        for handler in self.records.log.handlers: handler.close()
        for owner in self.owners:
            subprocess.run(["umount",str(self.users/owner)],check=True)
            name="fenixacct-"+owner.replace("-","")+".slice"
            subprocess.run(["systemctl","stop",name],check=True)
            properties=Path("/run/systemd/system")/(name+".d")
            if properties.exists(): shutil.rmtree(properties)
        subprocess.run(["systemctl","daemon-reload"],check=True)
        shutil.rmtree(self.root)

    async def request(self,owner_index=0,**request):
        account=self.accounts[owner_index]
        reader,writer=await asyncio.open_unix_connection(self.socket)
        writer.write(json.dumps(dict(account_id=account["id"],credential=account["upstream_token"],**request)).encode()+b"\n")
        await writer.drain();response=json.loads(await asyncio.wait_for(reader.readline(),60));writer.close();await writer.wait_closed()
        return response

    async def start(self,command,profile="offline",timeout=30,tty=False,kind="shell",resource_paths=None):
        workspace=self.users/self.owners[0]/"workspace"
        grant=await self.request(operation="create",spec=dict(task="test-task",kind=kind,argv=["/bin/bash","--noprofile","--norc","-c",command],cwd=str(workspace),workspace=str(workspace),timeout=timeout,profile=profile,tty=tty,resource_paths=resource_paths or []))
        self.assertTrue(grant.get("ok"),grant)
        reader,writer=await asyncio.open_unix_connection(self.socket);self.writers.append(writer)
        writer.write(json.dumps(dict(operation="attach",id=grant["id"],capability=grant["capability"])).encode()+b"\n");await writer.drain()
        response=json.loads(await reader.readline());self.assertTrue(response.get("ok"),response)
        async def heartbeat():
            try:
                while not writer.is_closing():
                    writer.write(b"\x09\0\0\0\0");await writer.drain();await asyncio.sleep(2)
            except (OSError,ConnectionError): pass
        self.heartbeats.append(asyncio.create_task(heartbeat()))
        return grant,reader,writer

    async def output(self,reader):
        chunks=[];code=None
        while True:
            try: header=await asyncio.wait_for(reader.readexactly(5),40)
            except asyncio.IncompleteReadError: break
            kind,size=header[0],struct.unpack("!I",header[1:])[0]
            chunk=await reader.readexactly(size)
            if kind in (1,2): chunks.append(chunk)
            elif kind==3: code=json.loads(chunk);break
            elif kind==4: break
        return b"".join(chunks).decode(errors="replace"),code

    async def test_runner_files_environment_limits_and_owner_bound_handles(self):
        root=self.users/self.owners[0]
        command=f"set -eu; id -u; cat /sys/fs/cgroup/memory.max /sys/fs/cgroup/pids.max; test ! -e {root}/state/synthetic-secret; test ! -e /var/run/docker.sock; test -z \"${{FENIX_UPSTREAM_TOKEN-}}\"; test ! -e {self.users/self.owners[1]}/workspace; printf verified"
        grant,reader,writer=await self.start(command)
        denied=await self.request(1,operation="status",id=grant["id"]);self.assertFalse(denied["ok"])
        output,code=await self.output(reader)
        self.assertEqual(code,0,output);self.assertIn("61001",output)
        self.assertIn(str(self.policy.runner_limits.memory_mib*1024*1024),output)
        self.assertIn(str(self.policy.runner_limits.pids),output);self.assertIn("verified",output)
        writer.close()

    async def test_controller_provisioning_and_preflight_bind_current_identity(self):
        scripts = Path(__file__).resolve().parents[2] / "scripts"
        config = dict(self.config, socket=str(self.socket))
        config_path = self.root / "execution.json"
        private_write(config_path, json.dumps(config).encode())
        owner = self.owners[0]
        name = "fenix-user-" + owner
        async def run(*args):
            process = await asyncio.create_subprocess_exec(*map(str, args), stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
            output, error = await asyncio.wait_for(process.communicate(), 75)
            return process.returncode, output, error
        try:
            code, output, error = await run(sys.executable, scripts/"tenant-container.py", config["registry"], self.users,
                                           self.accounts[0]["email"], "--execution-config", config_path, "--image", config["image"])
            self.assertEqual(code, 0, error.decode())
            code, output, error = await run(sys.executable, scripts/"tenant-preflight.py", config_path, owner)
            self.assertEqual(code, 0, error.decode())
            self.accounts[0]["auth_epoch"] = 1
            self.save_registry()
            code, output, error = await run(sys.executable, scripts/"tenant-preflight.py", config_path, owner)
            self.assertNotEqual(code, 0)
            self.assertIn("authorization revision changed", error.decode())
        finally:
            await run("docker", "rm", "--force", name)
            await run(sys.executable, scripts/"tenant-network-guard.py", name, "--remove")
            await run("docker", "network", "rm", name)

    async def test_language_server_and_selected_skill_mounts_are_read_only(self):
        root = self.users/self.owners[0]
        command = "if touch forbidden 2>/dev/null; then exit 1; fi; test ! -e /var/run/docker.sock; printf readonly"
        grant, reader, writer = await self.start(command, kind="language_server")
        info = await self.backend.inspect(grant["id"])
        self.assertEqual(info["HostConfig"]["NetworkMode"], "none")
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("readonly", output)
        writer.close()
        await self.request(operation="cancel", id=grant["id"])
        skill = root/"skills"/"selected-helper"
        skill.mkdir(mode=0o755); os.chown(skill, 61001, 61001)
        (skill/"helper.sh").write_text("printf skill-helper-ok")
        command = f"bash {skill}/helper.sh; if touch {skill}/forbidden 2>/dev/null; then exit 1; fi"
        grant, reader, writer = await self.start(command, resource_paths=[str(skill)])
        output, code = await self.output(reader)
        self.assertEqual(code, 0, output); self.assertIn("skill-helper-ok", output)
        self.assertFalse((skill/"forbidden").exists())
        writer.close()

    async def test_cancel_removes_detached_descendants_and_one_use_attach(self):
        grant,reader,writer=await self.start("setsid /bin/sleep 120 </dev/null >/dev/null 2>&1 & echo running; wait")
        duplicate_reader,duplicate_writer=await asyncio.open_unix_connection(self.socket)
        duplicate_writer.write(json.dumps(dict(operation="attach",id=grant["id"],capability=grant["capability"])).encode()+b"\n");await duplicate_writer.drain()
        self.assertFalse(json.loads(await duplicate_reader.readline())["ok"]);duplicate_writer.close();await duplicate_writer.wait_closed()
        response=await self.request(operation="cancel",id=grant["id"]);self.assertTrue(response["ok"],response)
        self.assertTrue(await self.backend.absent(grant["id"]))
        self.assertFalse(self.service.executions)
        writer.close()

    async def test_revocation_removes_running_lease(self):
        grant,reader,writer=await self.start("echo ready; sleep 120")
        self.accounts[0]["disabled"]=True;self.save_registry()
        for _ in range(60):
            if grant["id"] not in self.service.executions: break
            await asyncio.sleep(.1)
        self.assertNotIn(grant["id"],self.service.executions)
        self.assertTrue(await self.backend.absent(grant["id"]))
        writer.close()

    async def test_resize_and_terminal_input_reach_the_guest(self):
        grant,reader,writer=await self.start("read -r answer; stty size; printf 'answer:%s' \"$answer\"",tty=True)
        for _ in range(50):
            if (await self.backend.inspect(grant["id"]))["State"]["Running"]: break
            await asyncio.sleep(.05)
        resized=await self.request(operation="resize",id=grant["id"],rows=40,columns=120)
        self.assertTrue(resized["ok"],resized)
        payload=b"hello\n";writer.write(b"\x00"+struct.pack("!I",len(payload))+payload);await writer.drain()
        output,code=await self.output(reader)
        self.assertEqual(code,0,output);self.assertIn("40 120",output);self.assertIn("answer:hello",output)

    async def test_docker_api_failure_still_kills_recorded_cgroup(self):
        grant,reader,writer=await self.start("echo ready; sleep 120")
        header=await reader.readexactly(5);await reader.readexactly(struct.unpack("!I",header[1:])[0])
        command=self.backend.command
        async def unavailable(*args,**kwargs):
            if args[:3]==("docker","rm","--force"): raise PolicyError("Synthetic Docker API outage")
            return await command(*args,**kwargs)
        with patch.object(self.backend,"command",side_effect=unavailable):
            cancelled=await self.request(operation="cancel",id=grant["id"])
            self.assertFalse(cancelled["ok"])
            for _ in range(50):
                if not (await self.backend.inspect(grant["id"]))["State"]["Running"]: break
                await asyncio.sleep(.05)
            self.assertFalse((await self.backend.inspect(grant["id"]))["State"]["Running"])
        for _ in range(60):
            if grant["id"] not in self.service.executions: break
            await asyncio.sleep(.1)
        self.assertTrue(await self.backend.absent(grant["id"]))

    async def test_compiled_rust_relay_round_trip(self):
        binary=Path(__file__).resolve().parents[2]/"target/debug/fenix"
        if not binary.is_file(): self.skipTest("Build the Fenix binary to test the Rust relay")
        workspace=self.users/self.owners[0]/"workspace"
        grant=await self.request(operation="create",spec=dict(task="relay-test",kind="shell",argv=["/bin/bash","-c","read -r answer; printf 'relay:%s' \"$answer\""],cwd=str(workspace),workspace=str(workspace),timeout=30,profile="offline"))
        self.assertTrue(grant["ok"],grant)
        relay=await asyncio.create_subprocess_exec(str(binary),"internal-runner",env=dict(FENIX_RUNNER_SOCKET=str(self.socket),FENIX_RUNNER_ID=grant["id"],FENIX_RUNNER_CAPABILITY=grant["capability"]),stdin=asyncio.subprocess.PIPE,stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.PIPE)
        try:
            output,error=await asyncio.wait_for(relay.communicate(b"hello\n"),30)
            self.assertEqual(relay.returncode,0,error.decode());self.assertEqual(output,b"relay:hello")
        finally:
            if relay.returncode is None: relay.kill();await relay.wait()

    async def test_guard_repair_and_private_guest_loopback_tunnel(self):
        command="exec python3 -m http.server 8123 --bind 127.0.0.1"
        grant,reader,writer=await self.start(command,profile="development")
        for _ in range(50):
            try:
                upstream_reader,upstream_writer=await self.backend.connect(grant["id"],8123);break
            except Exception: await asyncio.sleep(.1)
        else: self.fail("Guest service did not start")
        upstream_writer.write(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n");await upstream_writer.drain()
        self.assertIn(b"200 OK",await asyncio.wait_for(upstream_reader.read(1024),5));upstream_writer.close();await upstream_writer.wait_closed()
        await self.backend.guard_check(grant["id"],"development",repair=True)
        await self.backend.guard_check(grant["id"],"development")
        info=await self.backend.inspect(grant["id"])
        self.assertFalse(info["NetworkSettings"]["Ports"])
        gateway = next(iter(info["NetworkSettings"]["Networks"].values()))["Gateway"]
        # A controlled host listener makes denial observable without probing real
        # metadata services or another user's applications.
        with socket.socket() as listener:
            listener.bind(("0.0.0.0", 0)); listener.listen(1)
            port = listener.getsockname()[1]
            probe = "import socket,sys; s=socket.socket(); s.settimeout(2); sys.exit(0 if s.connect_ex((sys.argv[1],int(sys.argv[2]))) else 1)"
            await self.backend.command("docker", "exec", "fenix-run-"+grant["id"], "python3", "-c", probe, gateway, str(port))
        await self.request(operation="cancel",id=grant["id"])
        writer.close()

if __name__=="__main__": unittest.main()
