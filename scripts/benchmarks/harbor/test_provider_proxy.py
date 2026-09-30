"""Check the host-side recording proxy without a real provider or key."""
import json
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from provider_proxy import build, read_key

SECRET = "host-only-fixture-key"


class Upstream(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        authorized = self.headers["Authorization"] == "Bearer " + SECRET
        payload = json.dumps({"model": body["model"], "authorized": authorized,
                              "usage": {"prompt_tokens": 3, "completion_tokens": 1, "cost": 1e-6}}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Retry-After", "2")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


class ProviderProxyTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.upstream = ThreadingHTTPServer(("127.0.0.1", 0), Upstream)
        threading.Thread(target=self.upstream.serve_forever, daemon=True).start()
        url = f"http://127.0.0.1:{self.upstream.server_port}/v1/chat/completions"
        self.proxy = build(self.root, "127.0.0.1", 0, models=("fast-model", "master-model"), key=SECRET, upstream=url)
        threading.Thread(target=self.proxy.serve_forever, daemon=True).start()

    def tearDown(self):
        for server in (self.proxy, self.upstream):
            server.shutdown()
            server.server_close()

    def post(self, model):
        request = Request(f"http://127.0.0.1:{self.proxy.server_port}/trial-1/v1/chat/completions",
                          json.dumps({"model": model, "messages": []}).encode(),
                          {"Authorization": "Bearer placeholder", "Content-Type": "application/json"})
        return urlopen(request, timeout=10)

    def test_forwards_each_permitted_role_with_the_host_key_and_records_usage(self):
        for model in ("fast-model", "master-model"):
            with self.post(model) as response:
                self.assertEqual(json.loads(response.read())["authorized"], True)
                self.assertEqual(response.headers["Retry-After"], "2")
        records = [json.loads(p.read_text()) for p in (self.root / "api" / "trial-1").glob("*.metadata.json")]
        self.assertEqual(sorted(r["model"] for r in records), ["fast-model", "master-model"])
        self.assertTrue(all(r["usage"]["cost"] == 1e-6 for r in records))
        for path in self.root.rglob("*"):
            if path.is_file():
                self.assertNotIn(SECRET, path.read_text())

    def test_rejects_models_outside_the_evaluation(self):
        with self.assertRaises(HTTPError) as error:
            self.post("other-model")
        self.assertEqual(error.exception.code, 400)

    def test_reads_a_named_variable_from_a_host_env_file(self):
        env = self.root / "provider.env"
        env.write_text("OTHER=x\nexport RUNINFRA_GATEWAY_KEY='abc'\n")
        self.assertEqual(read_key(self.root, env, "RUNINFRA_GATEWAY_KEY"), "abc")
        with self.assertRaises(SystemExit):
            read_key(self.root, env, "MISSING")


if __name__ == "__main__":
    unittest.main()
