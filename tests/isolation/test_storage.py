"""Opt-in, fixed-size ext4 migration and exhaustion acceptance test."""
import errno
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import uuid

sys.path.insert(0,str(Path(__file__).resolve().parents[2]/"scripts"))
from fenix_isolation import private_write,verify_storage

@unittest.skipUnless(os.environ.get("FENIX_DOCKER_TESTS")=="1" and os.geteuid()==0,"Requires explicit root opt-in for a disposable loop filesystem")
class StorageAcceptance(unittest.TestCase):
    def test_migration_preserves_data_and_capacity_survives_remount(self):
        directory=Path(tempfile.mkdtemp(prefix="fenix-storage-test-",dir="/var/tmp"))
        owner=str(uuid.uuid4());users=directory/"users";users.mkdir(mode=0o700)
        account=users/owner;account.mkdir()
        (account/"workspace").mkdir();(account/"state").mkdir()
        (account/"workspace/input.txt").write_text("project-data")
        (account/"state/history.txt").write_text("private-history")
        secret=directory/"outside-secret";secret.write_text("outside")
        (account/"workspace/link").symlink_to(secret)
        units=directory/"units";units.mkdir()
        config=directory/"execution.json"
        capacity=64*1024*1024
        private_write(config,json.dumps(dict(users_root=str(users),volume_dir=str(directory/"volumes"),host_disk_reserve_bytes=256*1024*1024,
            accounts={owner:dict(uid=61001,gid=61001,disk_bytes=capacity)})).encode())
        script=Path(__file__).resolve().parents[2]/"scripts/tenant-storage.py"
        def command(action,*args,check=True):
            return subprocess.run([sys.executable,str(script),str(config),owner,action,*args],check=check,capture_output=True,text=True,timeout=60)
        try:
            result=command("migrate","--write-unit",str(units))
            self.assertIn("ready",result.stdout)
            verify_storage(account,capacity)
            self.assertEqual(account.stat().st_uid,61001)
            self.assertEqual((account/"workspace/input.txt").read_text(),"project-data")
            self.assertTrue((account/"workspace/link").is_symlink())
            self.assertEqual(secret.read_text(),"outside")
            backup=users/(owner+".pre-isolation")
            self.assertEqual(backup.stat().st_uid,0)
            self.assertEqual(backup.stat().st_mode & 0o777,0o700)
            self.assertEqual((backup/"state/history.txt").read_text(),"private-history")
            self.assertNotEqual(command("migrate",check=False).returncode,0)
            written=0
            with open(account/"workspace/fill", "wb",buffering=0) as target:
                with self.assertRaises(OSError) as failure:
                    while written<=capacity:
                        written+=target.write(b"x"*(1024*1024))
                self.assertEqual(failure.exception.errno,errno.ENOSPC)
            self.assertLess(written,capacity)
            (account/"workspace/fill").unlink()
            subprocess.run(["umount",str(account)],check=True)
            command("mount");command("check")
            self.assertEqual((account/"state/history.txt").read_text(),"private-history")
            self.assertEqual(len(list(units.glob("*.mount"))),1)
        finally:
            if os.path.ismount(account): subprocess.run(["umount",str(account)],check=True)
            shutil.rmtree(directory)

if __name__=="__main__": unittest.main()
