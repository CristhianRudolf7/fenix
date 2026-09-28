import io
import tarfile
import tempfile
import unittest
from pathlib import Path
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


if __name__ == '__main__':
    unittest.main()
