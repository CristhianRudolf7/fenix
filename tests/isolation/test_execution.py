"""Security contract tests without Docker, network, credentials or production accounts."""
import asyncio
import dataclasses
import importlib.util
import json
import os
from pathlib import Path
import socket
import sys
import tempfile
import time
import unittest
from unittest.mock import AsyncMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from fenix_isolation import Limits, PolicyError, account_fingerprint, private_write, registry
from fenix_execution.policy import Policy, PinnedPath
from fenix_execution.service import Records, Service
from fenix_execution.checkpoints import Checkpoints
from fenix_execution.broker import Broker, public_addresses
from fenix_execution.docker import Docker

ALICE = "00000000-0000-4000-8000-000000000001"
BOB = "00000000-0000-4000-8000-000000000002"


class Fixture:
    def __init__(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.users = self.root / "users"
        self.users.mkdir(mode=0o700)
        self.accounts = [dict(id=owner, email=name+"@example.test", upstream="http://127.0.0.1:"+str(4001+i), upstream_token=name[0]*64, disabled=False)
                         for i, (owner, name) in enumerate(((ALICE, "alice"), (BOB, "bob")))]
        self.config = dict(backend="oci", runtime="runc", max_runner_memory_mib=512, host_memory_reserve_mib=256, controller_limits=dict(memory_mib=128,tmp_mib=16),registry=str(self.root / "accounts.json"), users_root=str(self.users), image="sha256:"+"a"*64,
                           accounts={owner: dict(uid=61001, gid=61001, disk_bytes=64*1024*1024) for owner in (ALICE,BOB)},
                           runner_limits=dict(memory_mib=128,tmp_mib=16,concurrent=2), account_limits=dict(memory_mib=256,tmp_mib=16,concurrent=2),
                           checkpoint_key=str(self.root / "key.json"), integrations_file=str(self.root / "integrations.json"))
        self.config["accounts"][BOB]["uid"] += 1
        self.config["accounts"][BOB]["gid"] += 1
        for owner in (ALICE, BOB):
            for directory in ("workspace", "projects", "state"):
                (self.users / owner / directory).mkdir(parents=True)
        self.save()
        private_write(Path(self.config["checkpoint_key"]), json.dumps({"key":"13"*32}).encode())
        self.policy = Policy(self.config)
        # These are policy unit tests, not a claim that tempdirs enforce disk quotas.
        self.policy.storage = lambda owner: self.users / owner
        self.records = Records(self.root / "state")

    def save(self):
        private_write(Path(self.config["registry"]), json.dumps(dict(accounts=self.accounts)).encode())

    def spec(self, owner=ALICE, **overrides):
        cwd = self.users / owner / "workspace"
        return dict(task="task-1",kind="shell",argv=["/bin/true"],cwd=str(cwd),workspace=str(cwd),profile="offline",**overrides)

    def close(self):
        self.records.db.close()
        for handler in self.records.log.handlers:
            handler.close()
        self.temp.cleanup()


class PolicyTests(unittest.TestCase):
    def setUp(self): self.f = Fixture()
    def tearDown(self): self.f.close()

    def test_credentials_do_not_allow_owner_spoofing_or_revoked_access(self):
        p = self.f.policy
        self.assertEqual(p.authenticate(dict(account_id=ALICE,credential="a"*64))["id"], ALICE)
        for request in (dict(account_id=BOB,credential="a"*64),dict(account_id=ALICE,credential="")):
            with self.assertRaises(PolicyError): p.authenticate(request)
        old = account_fingerprint(self.f.accounts[0])
        self.f.accounts[0]["disabled"] = True
        self.f.save()
        self.assertFalse(p.active(ALICE,old))
        with self.assertRaises(PolicyError): p.authenticate(dict(account_id=ALICE,credential="a"*64))

    def test_duplicate_mapping_and_insecure_registry_rejected(self):
        self.f.accounts[1]["upstream"] = self.f.accounts[0]["upstream"]
        self.f.save()
        with self.assertRaises(PolicyError): registry(Path(self.f.config["registry"]))
        Path(self.f.config["registry"]).chmod(0o644)
        with self.assertRaises(PolicyError): registry(Path(self.f.config["registry"]))

    def test_controller_state_foreign_workspace_environment_and_links_rejected(self):
        for path in (self.f.users/BOB/"workspace",self.f.users/ALICE/"state"):
            spec = self.f.spec(); spec.update(cwd=str(path),workspace=str(path))
            with self.assertRaises((PolicyError,ValueError)): self.f.policy.launch(self.f.accounts[0],spec)
        spec = self.f.spec(environment={"BASH_ENV":"/evil"})
        with self.assertRaises(PolicyError): self.f.policy.launch(self.f.accounts[0],spec)
        path=self.f.users/ALICE/"workspace"/"link"
        path.symlink_to(self.f.users/ALICE/"state",target_is_directory=True)
        spec=self.f.spec(); spec.update(cwd=str(path),workspace=str(path))
        with self.assertRaises(PolicyError): self.f.policy.launch(self.f.accounts[0],spec)

    def test_pinned_inode_survives_path_swap_and_denies_hard_links(self):
        base=self.f.users/ALICE/"workspace"
        source=base/"project";source.mkdir();(source/"input").write_text("safe")
        mount=PinnedPath(base,source)
        source.rename(base/"saved"); source.symlink_to(self.f.users/BOB/"workspace",target_is_directory=True)
        try: self.assertEqual((Path(mount.source)/"input").read_text(),"safe")
        finally: mount.close()
        original=base/"original";original.write_text("a");os.link(original,base/"hard")
        with self.assertRaises(PolicyError): PinnedPath(base,base/"hard")

    def test_lsp_is_read_only_offline_and_docker_arguments_are_constrained(self):
        spec=self.f.spec();spec["kind"]="language_server"
        launch=self.f.policy.launch(self.f.accounts[0],spec)
        backend=object.__new__(Docker);backend.instance="test";backend.runtime="runc";backend.image="sha256:"+"a"*64
        try:
            self.assertTrue(all(ro for _,ro in launch.mounts));self.assertEqual(launch.profile,"offline")
            args=backend.create_args(BOB,launch,"test.slice")
            self.assertIn("--read-only",args);self.assertIn("--pids-limit",args);self.assertIn("no-new-privileges",args)
            self.assertNotIn("--privileged",args);self.assertNotIn("/var/run/docker.sock"," ".join(args))
            self.assertEqual(args[args.index("--network")+1],"none")
        finally: launch.close()

    def test_empty_firewall_is_not_valid_and_private_destinations_precede_public_allow(self):
        path=Path(__file__).resolve().parents[2]/"scripts/tenant-network-guard.py"
        spec=importlib.util.spec_from_file_location("guard",path);guard=importlib.util.module_from_spec(spec);spec.loader.exec_module(guard)
        rules=guard.policy("FXtest","FHtest","web",["1.1.1.1"],["203.0.113.10"])
        public=next(i for i,r in enumerate(rules) if "--dports" in r)
        private=next(i for i,r in enumerate(rules) if "169.254.0.0/16" in r)
        self.assertLess(private,public)
        with patch.object(guard,"network_info",return_value=("br-test","FXtest","FHtest")),patch.object(guard,"resolvers",return_value=[]),patch.object(guard,"host_addresses",return_value=[]),patch.object(guard.subprocess,"check_output",return_value="-N FXtest\n"):
            with self.assertRaises(PolicyError): guard.reconcile("ignored",check=True)


class LifecycleTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.f=Fixture()
        self.backend=AsyncMock();self.backend.runtime="test";self.backend.image="test";self.backend.absent.return_value=True
        self.service=Service(self.f.policy,self.backend,self.f.records)
    async def asyncTearDown(self):
        await self.service.shutdown();self.f.close()

    async def test_owner_scope_revocation_and_idempotent_cleanup(self):
        grant=await self.service.create(self.f.accounts[0],self.f.spec())
        with self.assertRaises(PolicyError): self.service.owned(BOB,grant["id"])
        self.assertEqual(self.f.records.list(BOB),[])
        current=self.service.owned(ALICE,grant["id"])
        self.f.accounts[0]["disabled"]=True;self.f.save()
        with self.assertRaises(PolicyError): self.service.owned(ALICE,grant["id"])
        await self.service.destroy(current,"revoked");await self.service.destroy(current,"revoked")
        self.backend.destroy.assert_awaited_once();self.assertNotIn(grant["id"],self.service.executions)
        self.assertEqual(self.f.records.owned(ALICE,grant["id"])[3],"revoked")

    async def test_workspace_conflicts_and_concurrency_are_bounded(self):
        first=await self.service.create(self.f.accounts[0],self.f.spec())
        second=self.f.spec();second["task"]="task-2"
        with self.assertRaises(PolicyError): await self.service.create(self.f.accounts[0],second)
        await self.service.create(self.f.accounts[0],self.f.spec())
        with self.assertRaises(PolicyError): await self.service.create(self.f.accounts[0],self.f.spec())
        self.assertEqual(len(self.service.executions),2)
        self.assertEqual(self.f.records.list(ALICE)[0]["status"],"created")

    async def test_provisioning_is_not_monitored_until_ready(self):
        started,release=asyncio.Event(),asyncio.Event()
        async def create(*args): started.set();await release.wait()
        self.backend.create.side_effect=create
        creation=asyncio.create_task(self.service.create(self.f.accounts[0],self.f.spec()))
        await started.wait()
        monitor=asyncio.create_task(self.service.monitor())
        await asyncio.sleep(.03)
        self.backend.guard_check.assert_not_awaited()
        release.set();await creation
        monitor.cancel()
        with self.assertRaises(asyncio.CancelledError): await monitor

    async def test_real_socket_rejects_foreign_cancel_without_touching_runner(self):
        grant=await self.service.create(self.f.accounts[0],self.f.spec())
        path=self.f.root/"control.sock"
        server=await asyncio.start_unix_server(self.service.handle,path=path)
        async with server:
            reader,writer=await asyncio.open_unix_connection(path)
            writer.write(json.dumps(dict(operation="cancel",id=grant["id"],account_id=BOB,credential="b"*64)).encode()+b"\n");await writer.drain()
            response=json.loads(await reader.readline());self.assertFalse(response["ok"])
            writer.close();await writer.wait_closed()
        self.backend.destroy.assert_not_awaited()

    async def test_checkpoint_blocks_project_writers_but_allows_readers_and_listing(self):
        from unittest.mock import Mock
        self.service.snapshots = Mock()
        self.service.snapshots.dispatch.return_value = {"checkpoints": []}
        path = self.f.root / "checkpoint.sock"
        server = await asyncio.start_unix_server(self.service.handle, path=path)
        async def request(operation):
            reader, writer = await asyncio.open_unix_connection(path)
            writer.write(json.dumps(dict(operation=operation, account_id=ALICE, credential="a"*64,
                                         workspace=str(self.f.users/ALICE/"workspace"))).encode()+b"\n")
            await writer.drain()
            response = json.loads(await reader.readline())
            writer.close(); await writer.wait_closed()
            return response
        async with server:
            writable = await self.service.create(self.f.accounts[0], self.f.spec())
            self.assertFalse((await request("checkpoint"))["ok"])
            self.service.snapshots.dispatch.assert_not_called()
            self.assertTrue((await request("list_checkpoints"))["ok"])
            await self.service.destroy(self.service.owned(ALICE, writable["id"]))
            spec = self.f.spec(); spec["kind"] = "language_server"
            await self.service.create(self.f.accounts[0], spec)
            self.assertTrue((await request("checkpoint"))["ok"])


class CheckpointTests(unittest.TestCase):
    def setUp(self): self.f=Fixture();self.snapshots=Checkpoints(self.f.policy,self.f.records,self.f.config)
    def tearDown(self): self.snapshots.db.close();self.f.close()

    def create(self):
        workspace=self.f.users/ALICE/"workspace"
        (workspace/"sample.txt").write_text("private-project-content")
        return self.snapshots.dispatch(self.f.accounts[0],dict(operation="checkpoint",workspace=str(workspace)))["checkpoint"]

    def test_encryption_owner_binding_and_fresh_restore(self):
        item=self.create();key=item["id"]
        blob=(self.snapshots.directory/(key+".enc")).read_bytes()
        self.assertNotIn(b"private-project-content",blob)
        with self.assertRaises(PolicyError): self.snapshots.dispatch(self.f.accounts[1],dict(operation="delete_checkpoint",id=key))
        destination=self.f.users/ALICE/"workspace"/"restored"
        with patch("fenix_execution.checkpoints.os.fchown"), patch("fenix_execution.checkpoints.os.chown"):
            result=self.snapshots.dispatch(self.f.accounts[0],dict(operation="restore",id=key,destination=str(destination)))
        self.assertNotEqual(result["generation"],key)
        self.assertEqual((destination/"sample.txt").read_text(),"private-project-content")
        with self.assertRaises(PolicyError): self.snapshots.dispatch(self.f.accounts[0],dict(operation="restore",id=key,destination=str(destination)))

    def test_tampering_and_revocation_prevent_restore_before_writes(self):
        key=self.create()["id"];path=self.snapshots.directory/(key+".enc")
        data=bytearray(path.read_bytes());data[-1]^=1;path.write_bytes(data)
        destination=self.f.users/ALICE/"workspace"/"restored"
        with self.assertRaises(PolicyError): self.snapshots.dispatch(self.f.accounts[0],dict(operation="restore",id=key,destination=str(destination)))
        self.assertFalse(destination.exists())
        self.f.accounts[0]["disabled"]=True;self.f.save()
        with self.assertRaises(PolicyError): self.snapshots.dispatch(self.f.accounts[0],dict(operation="list_checkpoints"))

    def test_checkpoint_rejects_symlinks_and_controller_state(self):
        workspace=self.f.users/ALICE/"workspace";(workspace/"secret").symlink_to(self.f.users/ALICE/"state")
        with self.assertRaises((OSError,PolicyError)): self.snapshots.dispatch(self.f.accounts[0],dict(operation="checkpoint",workspace=str(workspace)))
        with self.assertRaises(PolicyError): self.snapshots.dispatch(self.f.accounts[0],dict(operation="checkpoint",workspace=str(self.f.users/ALICE/"state")))


class BrokerTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.f=Fixture()
        self.entry=dict(host="api.example.test",secret="Bearer synthetic-secret-123456",expires=time.time()+3600,operations={"read":dict(url="https://api.example.test/v1/owner-resource",method="GET",max_calls=1)})
        private_write(Path(self.f.config["integrations_file"]),json.dumps({"accounts":{ALICE:{"example":self.entry}}}).encode())
        self.broker=Broker(self.f.policy,self.f.records,self.f.config)
    async def asyncTearDown(self): self.f.close()

    async def test_foreign_connection_and_payload_override_rejected_without_request(self):
        request=dict(operation="integration_request",connection="example",action="read",task="task-1")
        with patch.object(self.broker,"perform") as send:
            with self.assertRaises(PolicyError): await self.broker.dispatch(self.f.accounts[1],request)
            with self.assertRaises(PolicyError): await self.broker.dispatch(self.f.accounts[0],{**request,"body":{"account_id":BOB}})
            send.assert_not_called()

    async def test_usage_limit_is_durable_and_no_implicit_retry(self):
        request=dict(operation="integration_request",connection="example",action="read",task="task-1")
        with patch.object(self.broker,"perform",return_value={"status":200,"response":{"ok":True}}) as send:
            await self.broker.dispatch(self.f.accounts[0],request)
            reopened=Broker(self.f.policy,self.f.records,self.f.config)
            with self.assertRaises(PolicyError): await reopened.dispatch(self.f.accounts[0],request)
            send.assert_called_once()

    async def test_private_and_mixed_dns_results_rejected(self):
        for addresses in (["127.0.0.1"],["169.254.169.254"],["::1"],["8.8.8.8","10.0.0.1"]):
            with patch("socket.getaddrinfo",return_value=[(socket.AF_INET,socket.SOCK_STREAM,6,"",(a,443)) for a in addresses]):
                with self.assertRaises(PolicyError): public_addresses("api.example.test",443)
        with patch("socket.getaddrinfo", return_value=[(socket.AF_INET, socket.SOCK_STREAM, 6, "", ("8.8.8.8", 443))]):
            with self.assertRaises(PolicyError): public_addresses("api.example.test", 443, ["8.8.8.8"])


if __name__ == "__main__": unittest.main()
