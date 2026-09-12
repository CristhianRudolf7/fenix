#!/usr/bin/env python3
"""Opt-in, billable Luna smoke test. Requires OPENAI_API_KEY; never prints it."""

import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/debug/fenix")
    parser.add_argument("--report", default="target/verification/live-smoke.json")
    args = parser.parse_args()
    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        raise SystemExit("OPENAI_API_KEY is required. No requests were sent.")
    binary = Path(args.binary).resolve()
    report = {"model": "gpt-5.6-luna", "reasoning_effort": "low", "scenarios": []}
    with tempfile.TemporaryDirectory(prefix="fenix-live-") as temporary:
        root = Path(temporary)
        workspace = root / "workspace"
        outside = root / "outside-workspace"
        workspace.mkdir()
        outside.mkdir()
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        base = f"http://127.0.0.1:{port}/api"
        environment = os.environ.copy()
        environment.pop("OPENAI_API_KEY", None)
        service = subprocess.Popen(
            [str(binary), "--port", str(port), "--data-dir", str(root / "data")],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=environment,
        )

        def api(path, payload=None):
            request = urllib.request.Request(
                base + path,
                data=json.dumps(payload).encode() if payload is not None else None,
                headers={"Content-Type": "application/json"},
            )
            try:
                with urllib.request.urlopen(request, timeout=150) as response:
                    return json.load(response)
            except urllib.error.HTTPError as error:
                # Backend errors are classified, but avoid printing any remote body.
                raise RuntimeError(f"Local API returned HTTP {error.code} at {path}") from None

        def run_scenario(name, prompt):
            task_id = api("/messages", {"text": prompt})["task_id"]
            deadline = time.monotonic() + 240
            approved = set()
            while time.monotonic() < deadline:
                status = api("/status")
                pending = status.get("pending")
                if pending and pending["id"] not in approved:
                    action = pending.get("action") or {}
                    arguments = action.get("arguments") or {}
                    target = arguments.get("path", "")
                    path = Path(target) if Path(target).is_absolute() else workspace / target
                    file_allowed = action.get("tool") in ("file_write", "file_edit") and path.resolve() == workspace / "result.txt"
                    command_allowed = action.get("tool") == "command_run" and arguments.get("command", "").strip() == "printf '%s' 'FENIX_COMMAND_OK'" and Path(arguments.get("cwd", "/")).resolve() == outside
                    if pending["kind"] != "approval" or not (file_allowed or command_allowed):
                        raise RuntimeError(f"Unexpected pending request in scenario {name}; no approval was sent")
                    api("/messages", {"text": f"aprobar {pending['id']}"})
                    approved.add(pending["id"])
                task = api(f"/tasks/{task_id}")
                if task["task"]["status"] in ("completed", "failed", "cancelled", "interrupted"):
                    events = task["events"]
                    result = {
                        "scenario": name, "status": task["task"]["status"],
                        "tools": [event["data"]["tool"] for event in events if event["kind"] == "tool_started"],
                        "model_calls": sum(event["kind"] == "model_response" for event in events),
                        "approvals": len(approved),
                        "reported_tokens": sum((event["data"].get("usage") or {}).get("total_tokens") or 0 for event in events),
                        "tool_errors": [event["data"].get("tool") for event in events if event["kind"] == "tool_result" and event["data"].get("is_error")],
                    }
                    report["scenarios"].append(result)
                    print(json.dumps(result), flush=True)
                    if result["status"] != "completed":
                        raise RuntimeError(f"Scenario {name} did not complete: {task['task'].get('error')}")
                    return task
                time.sleep(0.4)
            raise RuntimeError(f"Scenario {name} timed out")

        try:
            for _ in range(100):
                try:
                    api("/status")
                    break
                except (OSError, RuntimeError):
                    if service.poll() is not None:
                        raise RuntimeError("The local service did not start") from None
                    time.sleep(0.1)
            connection = api("/settings", {
                "provider": "openai", "model": "gpt-5.6-luna", "reasoning_effort": "low",
                "workspace": str(workspace), "command_policy": "ask", "file_write_policy": "ask",
                "api_key": key,
            })
            assert connection["connected"]
            print(json.dumps({"connection_verified": True, "model": connection["model"]}), flush=True)
            run_scenario("explicit_memory", "Guarda explícitamente en memoria este dato ficticio de prueba: mi bebida de prueba favorita es el té verde. Usa el título Bebida de prueba, comprueba duplicados y verifica que se guardó. No necesitas preguntarme nada más.")
            memories = api("/memory?query=verde")
            assert any("verde" in memory["content"].lower() for memory in memories)
            run_scenario("lexical_retrieval", "Usa memory_search para recuperar mi bebida de prueba favorita y dime cuál es, con su ID de memoria.")
            run_scenario("file_creation_and_verification", "Crea el archivo result.txt en la carpeta de archivos con el contenido exacto FENIX_LIVE_OK seguido de un salto de línea. Luego usa file_read para verificarlo. Si el ejecutor pide permiso, lo aprobaré por texto.")
            assert (workspace / "result.txt").read_text() == "FENIX_LIVE_OK\n"
            command_task = run_scenario("command_outside_workspace", f"Ejecuta exactamente el comando printf '%s' 'FENIX_COMMAND_OK' usando cwd {outside}. Comprueba su salida y código de salida con command_status antes de responder. El directorio está fuera de la carpeta de archivos; está permitido usarlo para comandos. Si necesitas aprobación, la daré por texto.")
            assert any(event["kind"] == "tool_result" and event["data"].get("tool") == "command_status" and event["data"]["result"].get("exit_code") == 0 and "FENIX_COMMAND_OK" in event["data"]["result"].get("output", "") for event in command_task["events"])
            run_scenario("reminder_creation", "Crea un recordatorio recurrente ficticio de prueba cada 3600 segundos llamado Revisión de prueba con el texto Revisa los pendientes. Es sólo un recordatorio de texto, no una tarea del agente. Verifica su próxima ejecución y dame su ID.")
            schedules = api("/schedules")
            assert any(schedule["mode"] == "reminder" and schedule["interval_seconds"] == 3600 for schedule in schedules)
            assert key not in json.dumps(api("/status"))
            assert key not in json.dumps(api("/messages"))
            report["success"] = True
        finally:
            service.terminate()
            try:
                service.wait(timeout=15)
            except subprocess.TimeoutExpired:
                service.kill()
                service.wait(timeout=5)
            destination = Path(args.report)
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"success": report.get("success", False), "report": args.report}), flush=True)


if __name__ == "__main__":
    main()
