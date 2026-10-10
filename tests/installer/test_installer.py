"""Offline installer contract tests; no real downloads or systemd mutations."""
import hashlib
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class InstallerTests(unittest.TestCase):
    def run_install(self, *, corrupt=False, unsafe=False, restart_failure=False, occupied=False, installed=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            home = root / 'home'
            home.mkdir()
            mocks = root / 'bin'
            mocks.mkdir()
            asset = root / 'fenix-linux-x86_64.tar.gz'
            with tarfile.open(asset, 'w:gz') as archive:
                for name, content in [('fenix', b'#!/bin/sh\nexit 0\n'), ('web/index.html', b'<html>Fenix</html>')]:
                    entry = tarfile.TarInfo(name)
                    entry.size = len(content)
                    entry.mode = 0o755 if name == 'fenix' else 0o644
                    archive.addfile(entry, io.BytesIO(content))
                if unsafe:
                    entry = tarfile.TarInfo('web/escape')
                    entry.type = tarfile.SYMTYPE
                    entry.linkname = '/etc/passwd'
                    archive.addfile(entry)
            digest = '0' * 64 if corrupt else hashlib.sha256(asset.read_bytes()).hexdigest()
            checksum = root / 'checksum'
            checksum.write_text(f'{digest}  {asset.name}\n')
            scripts = {
                'curl': '#!/bin/bash\nfor arg in "$@"; do [[ $arg == http://127.0.0.1:* ]] && exit 0; done\nwhile (($#)); do if [[ $1 == -o ]]; then dest=$2; shift 2; else url=$1; shift; fi; done\nif [[ $url == *.sha256 ]]; then cp "$MOCK_CHECKSUM" "$dest"; else cp "$MOCK_ASSET" "$dest"; fi\n',
                'systemctl': '#!/bin/bash\nif [[ "$*" == *is-active* ]]; then exit 1; fi\nif [[ "$*" == *restart* && ${MOCK_RESTART_FAILURE:-0} == 1 ]]; then exit 1; fi\nexit 0\n',
                'ss': '#!/bin/sh\n[ "${MOCK_OCCUPIED:-0}" = 1 ] && echo LISTEN\nexit 0\n',
            }
            for name, content in scripts.items():
                path = mocks / name
                path.write_text(content)
                path.chmod(0o755)
            personal = home / '.local/share/fenix-personal'
            if installed:
                (personal / 'data').mkdir(parents=True)
                (personal / 'data/keep').write_text('user data')
                (personal / 'releases/old').mkdir(parents=True)
                (personal / 'current').symlink_to(personal / 'releases/old')
                unit = home / '.config/systemd/user/fenix-personal.service'
                unit.parent.mkdir(parents=True)
                unit.write_text('# Managed by the Fenix public installer\nold unit\n')
            env = dict(os.environ, HOME=str(home), PATH=f'{mocks}:{os.environ["PATH"]}',
                       MOCK_ASSET=str(asset), MOCK_CHECKSUM=str(checksum), FENIX_NO_OPEN='1',
                       MOCK_RESTART_FAILURE=str(int(restart_failure)), MOCK_OCCUPIED=str(int(occupied)))
            result = subprocess.run(['bash', str(ROOT / 'scripts/install.sh')], env=env,
                                    capture_output=True, text=True, timeout=10)
            return result, (personal / 'current').exists(), (
                (personal / 'data/keep').read_text() if installed else None), (
                os.readlink(personal / 'current') if (personal / 'current').is_symlink() else None)

    def test_installs_without_build_tools(self):
        result, present, _, _ = self.run_install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(present)

    def test_checksum_mismatch_does_not_install(self):
        result, present, _, _ = self.run_install(corrupt=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(present)

    def test_symlink_archive_is_rejected(self):
        result, present, _, _ = self.run_install(unsafe=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(present)

    def test_update_preserves_data(self):
        result, _, data, link = self.run_install(installed=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(data, 'user data')
        self.assertFalse(link.endswith('/old'))

    def test_failed_start_rolls_back(self):
        result, present, data, link = self.run_install(installed=True, restart_failure=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(present)
        self.assertEqual(data, 'user data')
        self.assertTrue(link.endswith('/old'))

    def test_occupied_port_is_not_replaced(self):
        result, present, _, _ = self.run_install(occupied=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(present)


if __name__ == '__main__':
    unittest.main()
