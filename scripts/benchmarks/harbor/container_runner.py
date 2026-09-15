"""Submit an official instruction to the unmodified Fenix HTTP runtime."""

import json
import os
import signal
import select
import socket
import socketserver
import sqlite3
import subprocess
import sys
import threading
import time
from pathlib import Path
from urllib.request import Request, urlopen
from urllib.error import HTTPError

ROOT = Path("/opt/fenix-audit")
LOGS = Path("/logs/agent")
BASE = "http://127.0.0.1:4173"


def api(path, body=None):
    data = None if body is None else json.dumps(body).encode()
    request = Request(BASE + path, data, {"Content-Type": "application/json"})
    try:
        with urlopen(request, timeout=150) as response:
            return json.load(response)
    except HTTPError as error:
        raise RuntimeError(error.read().decode()) from error


class ProxyTunnel(socketserver.BaseRequestHandler):
    def handle(self):
        with socket.create_connection(("172.17.0.1", 48931), timeout=780) as upstream:
            peers = (self.request, upstream)
            while True:
                readable, _, _ = select.select(peers, [], [], 780)
                if not readable:
                    return
                for source in readable:
                    data = source.recv(65536)
                    if not data:
                        return
                    (upstream if source is self.request else self.request).sendall(data)


def save(name, value):
    target = LOGS / name
    temporary = target.with_suffix(".partial")
    temporary.write_text(json.dumps(value, indent=2))
    temporary.replace(target)


def save_checkpoints():
    database = ROOT / "data/fenix.sqlite3"
    if not database.exists():
        return
    with sqlite3.connect("file:" + str(database) + "?mode=ro", uri=True) as connection:
        rows = connection.execute(
            "SELECT kind,id,data,updated_at FROM records "
            "WHERE kind IN ('context_checkpoints','active_context_summaries')"
        ).fetchall()
    save("context-checkpoints.json", [
        {"kind": kind, "id": key, "data": json.loads(data), "updated_at": updated}
        for kind, key, data, updated in rows])


def main():
    LOGS.mkdir(parents=True, exist_ok=True)
    (ROOT / "controller.pid").write_text(str(os.getpid()))

    def interrupted(signum, frame):
        raise InterruptedError("Harbor ended the agent phase")

    signal.signal(signal.SIGTERM, interrupted)
    config = json.loads((ROOT / "run.json").read_text())
    tunnel = socketserver.ThreadingTCPServer(("127.0.0.1", 48932), ProxyTunnel)
    tunnel.daemon_threads = True
    threading.Thread(target=tunnel.serve_forever, daemon=True).start()
    log = (LOGS / "server.log").open("w")
    process = subprocess.Popen(
        [str(ROOT / "ld-linux-x86-64.so.2"), "--library-path", str(ROOT),
         str(ROOT / "fenix"), "--data-dir", str(ROOT / "data"),
         "--skills-dir", str(ROOT / "skills")], stdout=log, stderr=log,
        start_new_session=True,
    )
    started = time.monotonic()
    result = {"started_at": time.time()}
    try:
        for _ in range(120):
            if process.poll() is not None:
                raise RuntimeError("Fenix exited during startup")
            try:
                status = api("/api/status")
                break
            except Exception:
                time.sleep(0.5)
        else:
            raise RuntimeError("Fenix startup timeout")
        settings = status["default_settings"]
        settings.update(provider="openrouter", model="nemotron-3-5-lightning-30b",
                        base_url=config["api_base"].replace("172.17.0.1:48931", "127.0.0.1:48932"), reasoning_effort=None,
                        workspace=os.getcwd(), command_policy="allow", file_write_policy="allow",
                        api_key="audit-proxy-placeholder")
        save("settings.json", settings)
        api("/api/settings", settings)
        result["configured_seconds"] = time.monotonic() - started
        submitted = api("/api/chat", {"text": config["instruction"]})
        result["submitted"] = submitted
        blocked_since = None
        terminal_since = None
        events_by_task = {}
        while True:
            tasks = api("/api/tasks")
            status = api("/api/status")
            save("tasks.json", tasks)
            save("status.json", status)
            save("messages.json", api("/api/messages"))
            save_checkpoints()
            for task in tasks:
                recorded = events_by_task.setdefault(task["id"], [])
                while True:
                    cursor = recorded[-1]["seq"] if recorded else 0
                    detail = api("/api/tasks/" + task["id"] + "?limit=100&after=" + str(cursor))
                    page = detail["events"]
                    recorded.extend(page)
                    if len(page) < 100:
                        break
                detail["events"] = recorded
                save("task-" + task["id"] + ".json", detail)
            running = [t for t in tasks if t["status"] in ("queued", "running")]
            plans = status.get("plans") or []
            running = running or [p for p in plans if p["status"] in ("running", "revising")]
            if not running:
                blocked = any(t["status"] not in ("completed", "failed", "cancelled") for t in tasks)
                blocked = blocked or any(p["status"] in ("paused", "blocked") for p in plans)
                if not blocked:
                    blocked_since = None
                    terminal_since = terminal_since or time.monotonic()
                    if time.monotonic() - terminal_since >= 2:
                        result["stop_reason"] = "all_tasks_terminal"
                        result["terminal_quiescence_seconds"] = time.monotonic() - terminal_since
                        break
                else:
                    terminal_since = None
                    blocked_since = blocked_since or time.monotonic()
                    if time.monotonic() - blocked_since >= 15:
                        result["stop_reason"] = "requires_user_input"
                        break
            else:
                blocked_since = None
                terminal_since = None
            time.sleep(2)
    except Exception as error:
        result["error"] = str(error)
        raise
    finally:
        result["seconds"] = time.monotonic() - started
        # A completed Fenix turn can own persistent services. Keep the runtime
        # alive for the official verifier; exiting this controller closes its
        # inference tunnel, so no new model requests can run during verification.
        preserve_runtime = result.get("stop_reason") in ("all_tasks_terminal", "requires_user_input")
        result["runtime_preserved_for_verifier"] = preserve_runtime
        save("runner.json", result)
        if process.poll() is None and not preserve_runtime:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
        log.close()


if __name__ == "__main__":
    main()
