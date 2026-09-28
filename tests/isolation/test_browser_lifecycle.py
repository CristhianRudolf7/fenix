"""Browser memory lifecycle contracts without credentials or production mutation."""
import asyncio
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import time
import unittest
from unittest.mock import AsyncMock, patch
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'scripts'))
from test_execution import Fixture, ALICE, BOB
from fenix_isolation import PolicyError
from fenix_execution.admission import CapacityUnavailable
from fenix_execution.browser_checkpoints import BrowserCheckpoints, tree_fingerprint
from fenix_execution.service import Service


class QueueTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.f = Fixture()
        self.backend = AsyncMock()
        self.backend.runtime = 'test'; self.backend.image = 'test'; self.backend.absent.return_value = True
        self.service = Service(self.f.policy, self.backend, self.f.records)
        self.memory = patch('fenix_execution.service.available_memory_mib', return_value=16384)
        self.memory.start()

    async def asyncTearDown(self):
        await self.service.shutdown(); self.memory.stop(); self.f.close()

    async def tick(self):
        await self.service.queue.tick()
        jobs = [e['job'] for e in self.service.queue.entries.values() if e['job']]
        await asyncio.gather(*jobs)

    async def test_capacity_waits_then_grants_without_duplicate_execution(self):
        self.f.policy.config['max_executions'] = 1
        first = await self.service.create(self.f.accounts[0], self.f.spec())
        ticket = self.service.queue.submit(self.f.accounts[1], self.f.spec(BOB))['ticket']
        await self.tick()
        self.assertTrue((await self.service.queue.poll(BOB, ticket))['queued'])
        self.assertEqual(len(self.service.executions), 1)
        await self.service.destroy(self.service.executions[first['id']])
        await self.tick()
        grant = await self.service.queue.poll(BOB, ticket)
        self.assertEqual(self.service.executions[grant['id']].launch.owner, BOB)
        self.assertEqual(len(self.service.executions), 1)

    async def test_queue_cancellation_and_owner_boundary(self):
        ticket = self.service.queue.submit(self.f.accounts[0], self.f.spec())['ticket']
        with self.assertRaises(PolicyError): await self.service.queue.poll(BOB, ticket)
        await self.service.queue.cancel(ALICE, ticket)
        await self.tick()
        self.backend.create.assert_not_awaited()
        self.assertEqual(self.service.queue.entries, {})

    async def test_low_actual_host_memory_defers_even_with_free_quota(self):
        ticket = self.service.queue.submit(self.f.accounts[0], self.f.spec())['ticket']
        with patch('fenix_execution.service.available_memory_mib', return_value=100):
            await self.tick()
        self.assertTrue((await self.service.queue.poll(ALICE, ticket))['queued'])
        await self.tick()
        self.assertIn('id', await self.service.queue.poll(ALICE, ticket))

    async def test_revoked_queued_account_cannot_start(self):
        ticket = self.service.queue.submit(self.f.accounts[0], self.f.spec())['ticket']
        self.f.accounts[0]['disabled'] = True; self.f.save()
        await self.tick()
        with self.assertRaises(PolicyError): await self.service.queue.poll(ALICE, ticket)
        self.backend.create.assert_not_awaited()

    async def test_unclaimed_grant_is_destroyed(self):
        ticket = self.service.queue.submit(self.f.accounts[0], self.f.spec())['ticket']
        await self.tick()
        self.service.queue.entries[ticket]['touched'] -= 30
        await self.tick()
        self.assertEqual(len(self.service.executions), 0)

    async def test_owner_order_rotates_and_queue_is_bounded(self):
        self.f.policy.config['max_queued_per_account'] = 1
        self.service.queue.submit(self.f.accounts[0], self.f.spec())
        with self.assertRaises(PolicyError): self.service.queue.submit(self.f.accounts[0], self.f.spec())
        self.service.queue.submit(self.f.accounts[1], self.f.spec(BOB))
        order = []
        async def blocked(account, spec):
            order.append(account['id']); raise CapacityUnavailable('occupied')
        self.service.create = blocked
        await self.tick(); await self.tick()
        self.assertEqual(order, [ALICE, BOB, BOB, ALICE])

    async def browser(self):
        profile = self.f.users / ALICE / 'state/browser/profiles/form'
        profile.mkdir(parents=True)
        (profile / 'Cookies').write_text('private-state')
        (self.f.users / ALICE / 'state/browser/artifacts/form/downloads').mkdir(parents=True)
        spec = dict(task='browser:form', kind='browser', argv=['/usr/bin/chromium'], cwd=str(profile), workspace=str(profile), timeout=300)
        grant = await self.service.create(self.f.accounts[0], spec)
        execution = self.service.executions[grant['id']]
        execution.attached = True; execution.phase = 'running'
        return execution, profile

    async def test_suspend_releases_memory_reservation_but_keeps_profile_exclusive(self):
        execution, profile = await self.browser()
        raw = self.f.root / 'raw'; raw.mkdir(); (raw / 'pages.img').write_bytes(b'form-memory')
        self.backend.browser_checkpoints = True
        self.backend.checkpoint_browser.return_value = raw
        self.service.browsers.store.ensure_space = lambda *args: None
        await self.service.browsers.suspend(ALICE, execution.id)
        self.assertEqual(execution.phase, 'suspended')
        self.assertFalse(raw.exists())
        self.assertEqual(self.f.records.usage(ALICE)['reserved_seconds'], 0)
        self.f.policy.config['max_executions'] = 1
        await self.service.create(self.f.accounts[1], self.f.spec(BOB))
        # Removing active capacity does not unlock the owner's saved profile.
        with self.assertRaises(PolicyError): await self.service.create(self.f.accounts[0], execution.spec)
        await self.service.destroy(execution, attachment_generation=0)
        self.assertIn(execution.id, self.service.executions)

    async def test_mutated_profile_refuses_restore_and_keeps_checkpoint(self):
        execution, profile = await self.browser()
        raw = self.f.root / 'raw'; raw.mkdir(); (raw / 'pages.img').write_bytes(b'memory')
        self.backend.browser_checkpoints = True; self.backend.checkpoint_browser.return_value = raw
        self.service.browsers.store.ensure_space = lambda *args: None
        await self.service.browsers.suspend(ALICE, execution.id)
        (profile / 'Cookies').write_text('changed')
        with self.assertRaisesRegex(PolicyError, 'files changed'):
            await self.service.resume(self.f.accounts[0], execution.id)
        self.assertEqual(execution.phase, 'suspended')
        self.assertTrue(self.service.browsers.store.metadata(ALICE, execution.id))

    async def test_resume_rotates_capability_and_old_attachment_cannot_destroy_it(self):
        execution, _ = await self.browser()
        raw = self.f.root / 'raw'; raw.mkdir(); (raw / 'pages.img').write_bytes(b'memory')
        self.backend.browser_checkpoints = True; self.backend.checkpoint_browser.return_value = raw
        self.backend.checkpoint_directory.return_value = raw
        self.service.browsers.store.ensure_space = lambda *args: None
        old_hash = execution.capability_hash
        await self.service.browsers.suspend(ALICE, execution.id)
        grant = await self.service.resume(self.f.accounts[0], execution.id)
        self.assertEqual((raw / 'pages.img').read_bytes(), b'memory')
        self.assertNotEqual(old_hash, execution.capability_hash)
        execution.phase = 'running'
        await self.service.destroy(execution, attachment_generation=0)
        self.assertFalse(execution.cleaned)
        self.assertEqual(grant['generation'], 1)

    async def test_unsupported_checkpoint_keeps_running_browser(self):
        execution, _ = await self.browser()
        self.backend.browser_checkpoints = False
        with self.assertRaises(PolicyError): await self.service.browsers.suspend(ALICE, execution.id)
        self.assertEqual(execution.phase, 'running')
        self.backend.destroy.assert_not_awaited()

    async def saved_browser(self):
        execution, profile = await self.browser()
        raw = self.f.root / 'raw'; raw.mkdir(); (raw / 'pages.img').write_bytes(b'unsent-form')
        self.backend.browser_checkpoints = True
        self.backend.checkpoint_browser.return_value = raw
        self.backend.checkpoint_directory.return_value = raw
        self.service.browsers.store.ensure_space = lambda *args: None
        await self.service.browsers.suspend(ALICE, execution.id)
        return execution, profile, raw

    async def test_abandoned_resume_keeps_saved_form_and_rechecks_mount_content(self):
        execution, profile, raw = await self.saved_browser()
        ticket = self.service.queue.submit(self.f.accounts[0], execution.id, 'resume')['ticket']
        await self.tick()
        self.assertEqual(execution.phase, 'resuming')
        await self.service.queue.cancel(ALICE, ticket)
        self.assertEqual(execution.phase, 'suspended')
        self.assertEqual(self.f.records.usage(ALICE)['reserved_seconds'], 0)
        self.assertEqual((raw / 'pages.img').read_bytes(), b'unsent-form')
        self.backend.destroy.assert_not_awaited()
        (profile / 'Cookies').write_text('changed after admission')
        with self.assertRaisesRegex(PolicyError, 'files changed'):
            await self.service.resume(self.f.accounts[0], execution.id)

    async def test_encryption_failure_keeps_original_snapshot_recoverable(self):
        execution, _ = await self.browser()
        raw = self.f.root / 'raw'; raw.mkdir(); (raw / 'pages.img').write_bytes(b'unsent-form')
        self.backend.browser_checkpoints = True
        self.backend.checkpoint_browser.return_value = raw
        self.backend.checkpoint_directory.return_value = raw
        self.service.browsers.store.ensure_space = lambda *args: None
        with patch.object(self.service.browsers.store, 'seal', side_effect=OSError('disk failure')):
            with self.assertRaises(OSError): await self.service.browsers.suspend(ALICE, execution.id)
        self.assertEqual(execution.phase, 'suspended')
        self.assertEqual(self.f.records.usage(ALICE)['reserved_seconds'], 0)
        self.assertTrue(raw.exists())
        grant = await self.service.resume(self.f.accounts[0], execution.id)
        self.assertEqual(grant['id'], execution.id)

    async def test_restart_recovers_saved_owner_without_reserving_browser_ram(self):
        execution, _, _ = await self.saved_browser()
        key = execution.id
        self.backend.mount_dir = self.f.root / 'mounts'
        (self.backend.mount_dir / key).mkdir(parents=True)
        for index, (mount, _) in enumerate(execution.launch.mounts):
            (self.backend.mount_dir / key / str(index)).symlink_to(mount.path, target_is_directory=True)
        self.backend.inspect.return_value = {'State': {'Running': False}, 'Config': {'Labels': {'fenix.owner': ALICE}}}
        self.backend.egress_policies = {}
        await self.service.shutdown()
        self.service = Service(self.f.policy, self.backend, self.f.records)
        self.assertEqual(await self.service.browsers.recover(), {key})
        self.assertEqual(self.service.executions[key].phase, 'suspended')
        self.assertEqual(self.f.records.usage(ALICE)['reserved_seconds'], 0)
        with self.assertRaises(PolicyError): await self.service.resume(self.f.accounts[1], key)

    async def test_failed_native_checkpoint_leaves_original_execution_running(self):
        execution, _ = await self.browser()
        self.backend.browser_checkpoints = True
        self.service.browsers.store.ensure_space = lambda *args: None
        self.backend.checkpoint_browser.side_effect = PolicyError('unsupported socket')
        self.backend.inspect.return_value = {'State': {'Running': True}}
        with self.assertRaises(PolicyError): await self.service.browsers.suspend(ALICE, execution.id)
        self.assertEqual(execution.phase, 'running')
        self.assertFalse(execution.disconnect.is_set())
        self.backend.destroy.assert_not_awaited()

    async def test_failed_restore_of_a_still_stopped_container_preserves_form(self):
        execution, _, raw = await self.saved_browser()
        grant = await self.service.resume(self.f.accounts[0], execution.id)
        self.backend.attach_restored.side_effect = PolicyError('runtime rejected the snapshot')
        self.backend.inspect.return_value = {'State': {'Running': False}}
        with self.assertRaises(PolicyError):
            await self.service.attach(grant, AsyncMock(), AsyncMock())
        self.assertEqual(execution.phase, 'suspended')
        self.assertFalse(execution.cancelled.is_set())
        self.assertEqual(self.f.records.usage(ALICE)['reserved_seconds'], 0)
        self.assertEqual((raw / 'pages.img').read_bytes(), b'unsent-form')
        self.backend.destroy.assert_not_awaited()


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture(); self.store = BrowserCheckpoints(self.f.records, self.f.config)
        self.source = self.f.root / 'snapshot'; self.source.mkdir()
        self.payload = b'private form value' * 100000
        (self.source / 'pages').write_bytes(self.payload)
        self.metadata = dict(expires=time.time()+3600, spec={}, image='test')
        self.key = '00000000-0000-4000-8000-000000000003'
    def tearDown(self):
        self.store.db.close(); self.f.close()
    def test_streaming_roundtrip_and_owner_checks(self):
        self.store.seal(ALICE, self.key, self.source, self.metadata)
        self.assertNotIn(b'private form value', (self.store.directory / self.key / '0').read_bytes())
        target = self.f.root / 'restore'
        with self.assertRaises(PolicyError): self.store.unseal(BOB, self.key, target)
        self.store.unseal(ALICE, self.key, target)
        self.assertEqual((target / 'pages').read_bytes(), self.payload)
    def test_tamper_is_detected_before_restore_can_use_plaintext(self):
        self.store.seal(ALICE, self.key, self.source, self.metadata)
        blob = self.store.directory / self.key / '0'
        raw = bytearray(blob.read_bytes()); raw[20] ^= 1; blob.write_bytes(raw)
        target = self.f.root / 'restore'
        with self.assertRaises(Exception): self.store.unseal(ALICE, self.key, target)
        self.assertFalse(target.exists())
    def test_fingerprint_detects_same_size_content_change(self):
        before = tree_fingerprint([self.source]); original = (self.source/'pages').stat()
        (self.source/'pages').write_bytes(b'x' * len(self.payload))
        os.utime(self.source/'pages', ns=(original.st_atime_ns, original.st_mtime_ns))
        self.assertNotEqual(before, tree_fingerprint([self.source]))
    def test_disk_quota_preserves_original_source(self):
        self.store.limit = 1
        with self.assertRaises(PolicyError): self.store.seal(ALICE, self.key, self.source, self.metadata)
        self.assertTrue((self.source/'pages').exists())

if __name__ == '__main__': unittest.main()
