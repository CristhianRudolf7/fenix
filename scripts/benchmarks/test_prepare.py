import io
import os
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest import mock
from prepare import prepare, unpack


class DatasetIsolationTests(unittest.TestCase):
    def test_archive_cannot_write_outside_task_or_follow_symlink(self):
        for name, kind in [('../outside', tarfile.REGTYPE), ('/absolute', tarfile.REGTYPE), ('link', tarfile.SYMTYPE)]:
            with tempfile.TemporaryDirectory() as directory:
                stream = io.BytesIO()
                with tarfile.open(fileobj=stream, mode='w:gz') as archive:
                    member = tarfile.TarInfo(name)
                    member.type = kind
                    member.linkname = '/etc/passwd'
                    archive.addfile(member, io.BytesIO())
                with self.assertRaises(ValueError):
                    unpack(stream.getvalue(), Path(directory))

    def test_mutable_revision_and_existing_trial_are_rejected_before_network(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ValueError):
                prepare('terminalworld', 'main', Path(directory) / 'new')
            with self.assertRaises(ValueError):
                prepare('terminalworld', 'a' * 40, Path(directory))

    def test_official_file_names_with_spaces_are_url_encoded(self):
        tree = (b'{"tree": [{"path": "tasks/a/task.toml", "type": "blob", "mode": "100644"}, '
                b'{"path": "tasks/a/data/x y.xls", "type": "blob", "mode": "100644"}, '
                b'{"path": "tasks/a/environment/run.sh", "type": "blob", "mode": "100755"}]}')
        requested = []
        def fetch(url):
            requested.append(url)
            return tree if 'api.github.com' in url else b'content'
        previous = os.umask(0o077)
        try:
            with tempfile.TemporaryDirectory() as directory, mock.patch('prepare.fetch', fetch):
                manifest = prepare('terminal-bench-4.0', 'a' * 40, Path(directory) / 'task')
                task = Path(directory) / 'task/tasks/a'
                self.assertEqual((task / 'data/x y.xls').read_bytes(), b'content')
                # Task images and non-root verifiers must see the official modes.
                self.assertEqual((task / 'data/x y.xls').stat().st_mode & 0o777, 0o644)
                self.assertEqual((task / 'environment/run.sh').stat().st_mode & 0o777, 0o755)
                self.assertEqual((task / 'data').stat().st_mode & 0o777, 0o755)
        finally:
            os.umask(previous)
        self.assertIn('tasks/a/data/x y.xls', manifest['files_sha256'])
        self.assertTrue(any(url.endswith('/tasks/a/data/x%20y.xls') for url in requested))


if __name__ == '__main__':
    unittest.main()
