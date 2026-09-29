"""Verify private skill CRUD against the real server binary without provider calls.

Run: python3 apps/server/tests/skill_management_smoke.py [path/to/fenix]
"""
import concurrent.futures
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

BINARY = str(Path(sys.argv[1] if len(sys.argv) > 1 else "target/release/fenix").resolve())

class Account:
    def __init__(self, root):
        self.root = root
        self.data = root / "data"
        self.skills = root / "skills"
        self.data.mkdir(parents=True)
        (self.data / "web-search.json").write_text('{"enabled":false}')
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            self.port = sock.getsockname()[1]
        self.base = f"http://127.0.0.1:{self.port}"
        self.log = (root / "server.log").open("w+")
        self.process = None
        self.start()

    def start(self):
        self.process = subprocess.Popen([BINARY, "--port", str(self.port), "--data-dir", str(self.data), "--skills-dir", str(self.skills)], stdout=self.log, stderr=subprocess.STDOUT)
        for _ in range(120):
            if self.process.poll() is not None:
                raise RuntimeError("Test server exited before becoming ready")
            try:
                if self.call("GET", "/api/status")[0] == 200:
                    return
            except OSError:
                pass
            time.sleep(0.25)
        raise RuntimeError("Test server startup timed out")

    def stop(self):
        if self.process and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()

    def call(self, method, path, body=None, origin=None):
        request = urllib.request.Request(self.base + path, method=method,
            headers={"content-type": "application/json", "origin": origin or self.base},
            data=None if body is None else json.dumps(body).encode())
        try:
            with urllib.request.urlopen(request, timeout=20) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            text = error.read().decode()
            try:
                value = json.loads(text)
            except ValueError:
                value = {"error": text}
            return error.code, value

with tempfile.TemporaryDirectory(prefix="fenix-private-skills-") as directory:
    accounts = []
    try:
        alice = Account(Path(directory) / "alice"); accounts.append(alice)
        bob = Account(Path(directory) / "bob"); accounts.append(bob)
        name = "memory-and-reminders"
        path = f"/api/skills/{name}/source"
        status, source = alice.call("GET", path)
        assert status == 200, (status, source)
        original = source["content"]
        changed = original + "\n\nKeep this account's summary concise.\n"
        payload = {"content": changed, "revision": source["revision"]}
        assert alice.call("PUT", path, payload, "https://untrusted.example")[0] == 403
        status, saved = alice.call("PUT", path, payload)
        assert status == 200, (status, saved)
        assert alice.call("GET", path)[1]["content"] == changed
        assert bob.call("GET", path)[1]["content"] == original
        assert alice.call("PUT", path, payload)[0] == 400
        for content in ["", "x" * 32001, "---\ndescription: [\n---\nInvalid"]:
            assert alice.call("PUT", path, {"content": content, "revision": saved["revision"]})[0] == 400
        assert alice.call("GET", path)[1]["content"] == changed
        print("PASS: account isolation, origin guard, validation and stale writes")

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            responses = list(pool.map(lambda suffix: alice.call("PUT", path, {"content": changed + suffix, "revision": saved["revision"]}), ["\nFirst edit.", "\nSecond edit."]))
        assert sorted(status for status, _ in responses) == [200, 400], responses
        latest = alice.call("GET", path)[1]
        alice.stop(); alice.start()
        assert alice.call("GET", path)[1] == latest
        assert alice.call("DELETE", path, {"revision": source["revision"]})[0] == 400
        assert alice.call("DELETE", path, {"revision": latest["revision"]})[0] == 200
        assert not (alice.skills / name).exists()
        legacy = alice.data / "skills" / name
        legacy.mkdir(parents=True)
        (legacy / "SKILL.md").write_text(original)
        alice.stop(); alice.start()
        assert not (alice.skills / name).exists()
        assert all(skill["name"] != name for skill in alice.call("GET", "/api/skills")[1])
        assert bob.call("GET", path)[1]["content"] == original
        print("PASS: concurrent edits, save durability and deletion across restart/legacy migration")

        (alice.skills / "unsafe-link").symlink_to(bob.skills / name, target_is_directory=True)
        for method in ["GET", "PUT", "DELETE"]:
            body = payload if method == "PUT" else {"revision": source["revision"]} if method == "DELETE" else None
            assert alice.call(method, "/api/skills/unsafe-link/source", body)[0] == 400
        linked = alice.skills / "linked-source"
        linked.mkdir()
        os.link(bob.skills / name / "SKILL.md", linked / "SKILL.md")
        assert alice.call("GET", "/api/skills/linked-source/source")[0] == 400
        (linked / "SKILL.md").unlink()
        assert bob.call("GET", path)[1]["content"] == original
        print("PASS: symlinks and hard links cannot expose another account's files")
    finally:
        for account in accounts:
            account.stop()
            account.log.close()
