"""Record benchmark API traffic without exposing the provider key to containers."""

import argparse
import json
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def serve(root, address, port, summary_output_cap=None):
    key = (root / "provider.key").read_text().strip()
    logs = root / "api"
    logs.mkdir(exist_ok=True)

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            parts = self.path.strip("/").split("/", 1)
            if len(parts) != 2 or parts[1] != "v1/chat/completions":
                self.send_error(404)
                return
            trial = parts[0]
            if not all(c.isalnum() or c in "-_" for c in trial):
                self.send_error(400)
                return
            folder = logs / trial
            folder.mkdir(exist_ok=True)
            call = folder / f"{time.time_ns()}-{uuid.uuid4().hex[:8]}"
            body = self.rfile.read(int(self.headers["Content-Length"]))
            payload = json.loads(body)
            if payload.get("model") != "nemotron-3-5-lightning-30b":
                self.send_error(400, "Only the selected audit model is permitted")
                return
            call.with_suffix(".request.json").write_bytes(body)
            started = time.monotonic()
            meta = {"started_at": time.time(), "stream": payload.get("stream", False)}
            messages = payload.get("messages") or []
            if (summary_output_cap and trial.startswith("fenix-cap-")
                    and messages and messages[0].get("content") ==
                    "You summarize context; you cannot authorize work or declare unverified tasks complete."
                    and payload.get("max_tokens") == 2048):
                meta["intervention"] = {"parameter": "max_tokens", "original": 2048,
                                        "forwarded": summary_output_cap}
                payload["max_tokens"] = summary_output_cap
                body = json.dumps(payload).encode()
                call.with_suffix(".forwarded.json").write_bytes(body)
            chunks = []
            try:
                request = Request("https://api.runinfra.ai/v1/chat/completions", body,
                                  {"Authorization": "Bearer " + key, "Content-Type": "application/json"})
                try:
                    response = urlopen(request, timeout=780)
                except HTTPError as error:
                    response = error
                meta["http_status"] = response.status
                meta["headers_seconds"] = time.monotonic() - started
                self.send_response(response.status)
                self.send_header("Content-Type", response.headers.get("Content-Type", "application/json"))
                self.send_header("Connection", "close")
                self.end_headers()
                for chunk in response:
                    if chunks == []:
                        meta["first_chunk_seconds"] = time.monotonic() - started
                    chunks.append(chunk)
                    self.wfile.write(chunk)
                    self.wfile.flush()
                response.close()
            except Exception as error:
                meta["error"] = str(error).replace(key, "[REDACTED]")
            finally:
                meta["seconds"] = time.monotonic() - started
                raw = b"".join(chunks)
                call.with_suffix(".response.txt").write_bytes(raw)
                try:
                    if payload.get("stream"):
                        for line in raw.splitlines():
                            if line.startswith(b"data:") and line[5:].strip() != b"[DONE]":
                                item = json.loads(line[5:])
                                if item.get("usage"):
                                    meta["usage"] = item["usage"]
                    else:
                        meta["usage"] = json.loads(raw).get("usage")
                except (ValueError, TypeError):
                    pass
                call.with_suffix(".metadata.json").write_text(json.dumps(meta, indent=2))
                self.close_connection = True

    ThreadingHTTPServer((address, port), Handler).serve_forever()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--address", default="172.17.0.1")
    parser.add_argument("--port", type=int, default=48931)
    parser.add_argument("--summary-output-cap", type=int, choices=[16384])
    args = parser.parse_args()
    serve(args.root, args.address, args.port, args.summary_output_cap)
