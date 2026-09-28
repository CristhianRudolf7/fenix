"""Isolated evaluation gateway: scoped OpenAI quotas and optional Jev recording.

Keys stay on the host. This is not a production credential or billing service.
"""
import argparse
import gzip
import json
import re
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import requests


def seconds(value):
    try:
        return max(0.0, float(value))
    except (ValueError, TypeError):
        units = {"ms": .001, "s": 1, "m": 60, "h": 3600}
        return sum(float(n) * units[u] for n, u in re.findall(r"([\d.]+)(ms|s|m|h)", str(value)))


class Quotas:
    """Accounts with the same quota group share reservations and cooldowns."""
    def __init__(self, groups, clock=time.monotonic):
        self.clock = clock
        self.groups = groups
        self.states = {g: {"limit": 200000, "remaining": 200000,
                           "at": clock(), "cooldown": 0} for g in groups}
        self.next = 0
        self.lock = threading.RLock()
        self.admission = threading.Lock()

    def reserve(self, tokens):
        with self.lock:
            now = self.clock()
            choices = []
            for slot, group in enumerate(self.groups):
                state = self.states[group]
                available = min(state["limit"], state["remaining"] + (now-state["at"])*state["limit"]/60)
                if tokens > state["limit"]:
                    continue
                delay = max(0, state["cooldown"]-now, (tokens-available)*60/state["limit"])
                choices.append((delay, (slot-self.next) % len(self.groups), slot, available))
            if not choices:
                raise ValueError("Request exceeds the per-request token allowance of every quota group")
            delay, _, slot, available = min(choices)
            if delay > 0:
                return None, delay
            state = self.states[self.groups[slot]]
            state.update(remaining=available-tokens, at=now)
            self.next = (slot+1) % len(self.groups)
            return slot, 0

    def reserve_estimate(self, estimated_tokens):
        # Serialized JSON (including opaque reasoning state) is not a tokenizer.
        # An estimate may reserve a whole bucket, but cannot prove the provider
        # would reject this request. The provider remains the quota authority.
        with self.lock:
            reservation = min(estimated_tokens, max(s["limit"] for s in self.states.values()))
            slot, delay = self.reserve(reservation)
            return slot, delay, reservation

    def acquire_estimate(self, estimated_tokens, sleep=time.sleep):
        # Keep later, small requests from consuming every refill while a large
        # request waits. Release admission before the network request so API
        # calls that have already reserved their budget still run concurrently.
        with self.admission:
            while True:
                slot, delay, reservation = self.reserve_estimate(estimated_tokens)
                if slot is not None:
                    return slot, reservation
                sleep(min(delay, 5))

    def observe(self, slot, headers, status):
        with self.lock:
            state = self.states[self.groups[slot]]
            now = self.clock()
            limit = headers.get("x-ratelimit-limit-tokens")
            remaining = headers.get("x-ratelimit-remaining-tokens")
            if limit and remaining:
                try:
                    cap = int(limit)
                    available = min(state["limit"], state["remaining"]+(now-state["at"])*state["limit"]/60)
                    if cap > 0:
                        state.update(limit=cap, remaining=min(available, max(0, int(remaining))), at=now)
                except ValueError:
                    pass
            if status == 429:
                state["cooldown"] = max(state["cooldown"], now+max(1, seconds(headers.get("retry-after", "60")), seconds(headers.get("x-ratelimit-reset-tokens", "0"))))


def serve(root, configuration, address, port):
    root.mkdir(parents=True, exist_ok=True)
    root.chmod(0o700)
    config = json.loads(configuration.read_text())
    keys = [Path(k["file"]).read_text().strip() for k in config["openai"]]
    jev_key = Path(config["jev_key_file"]).read_text().strip() if config.get("jev_key_file") else None
    quotas = Quotas([k["quota_group"] for k in config["openai"]])

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            parts = self.path.strip("/").split("/", 1)
            if len(parts) != 2 or not re.fullmatch(r"[\w-]+", parts[0]) or parts[1] not in ("v1/responses", "v1/chat/completions", "v1/jev"):
                self.send_error(404)
                return
            length = int(self.headers.get("Content-Length", "0"))
            if length > 5_000_000:
                self.send_error(413)
                return
            started = time.monotonic()
            record = {"started_at": time.time(), "path": parts[1], "trial": parts[0]}
            folder = root / parts[0]
            folder.mkdir(exist_ok=True)
            prefix = folder / f"{time.time_ns()}-{uuid.uuid4().hex[:6]}"
            sent = False
            response = None
            try:
                body = self.rfile.read(length)
                if self.headers.get("Content-Encoding") == "gzip":
                    body = gzip.decompress(body)
                payload = json.loads(body)
                jev = parts[1] == "v1/jev"
                allowed_model = "typesafe-ai/jev" if jev else "gpt-6-luna"
                if payload.get("model") != allowed_model:
                    self.send_error(400, "Unexpected evaluation model")
                    return
                prefix.with_suffix(".request.json").write_bytes(body)
                record["model"] = allowed_model
                if jev and not jev_key:
                    self.send_error(403, "Decision model disabled for this evaluation")
                    return
                if jev:
                    endpoint = "https://ai-gateway.vercel.sh/typesafe/v1/systemone"
                    key = jev_key
                else:
                    endpoint = "https://api.openai.com/" + parts[1]
                    # Reserve conservatively, then honor provider remaining/reset headers.
                    tokens = len(body)//3 + int(payload.get("max_output_tokens", payload.get("max_completion_tokens", 16384))) + 1000
                    slot, reservation = quotas.acquire_estimate(tokens)
                    record.update(credential_slot=slot+1, estimated_tokens=tokens, reserved_tokens=reservation)
                    key = keys[slot]
                record["throttle_seconds"] = round(time.monotonic()-started, 3)
                response = requests.post(endpoint, headers={"Authorization": "Bearer "+key, "Content-Type": "application/json"}, data=body, stream=True, timeout=(15, 240))
                record["http_status"] = response.status_code
                record["rate_limits"] = {k:v for k,v in response.headers.items() if k.lower().startswith("x-ratelimit-")}
                if not jev:
                    quotas.observe(slot, response.headers, response.status_code)
                self.send_response(response.status_code)
                self.send_header("Content-Type", response.headers.get("Content-Type", "application/json"))
                for k,v in response.headers.items():
                    if k.lower() == "retry-after" or k.lower().startswith("x-ratelimit-"):
                        self.send_header(k,v)
                self.send_header("Connection", "close")
                self.end_headers()
                sent = True
                raw = bytearray()
                for chunk in response.iter_content(1024):
                    if chunk:
                        raw.extend(chunk)
                        self.wfile.write(chunk)
                        self.wfile.flush()
                prefix.with_suffix(".response.txt").write_bytes(raw)
                objects = []
                if payload.get("stream"):
                    for line in raw.decode(errors="replace").splitlines():
                        if line.startswith("data: "):
                            try:
                                objects.append(json.loads(line[6:]))
                            except ValueError:
                                pass
                else:
                    objects = [json.loads(raw)]
                for obj in objects:
                    result = obj.get("response", obj)
                    if result.get("usage"):
                        record["usage"] = result["usage"]
                    if result.get("provider_metadata"):
                        record["provider_metadata"] = result["provider_metadata"]
                    if result.get("error"):
                        record["error"] = result["error"]
            except Exception as error:
                record["error"] = str(error)
                if not sent:
                    self.send_error(502, "Evaluation gateway failed")
            finally:
                if response is not None:
                    response.close()
                self.close_connection = True
                record["seconds"] = time.monotonic()-started
                encoded = json.dumps(record, indent=2)
                for secret in keys + ([jev_key] if jev_key else []):
                    encoded = encoded.replace(secret, "[REDACTED]")
                prefix.with_suffix(".meta.json").write_text(encoded)

    print("Evaluation gateway ready", flush=True)
    ThreadingHTTPServer((address, port), Handler).serve_forever()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("configuration", type=Path)
    parser.add_argument("--address", default="172.17.0.1")
    parser.add_argument("--port", type=int, default=48966)
    args = parser.parse_args()
    serve(args.root, args.configuration, args.address, args.port)
