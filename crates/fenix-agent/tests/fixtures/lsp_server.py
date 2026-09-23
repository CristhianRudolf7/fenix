"""Deterministic stdio LSP fixture: lifecycle, diagnostics, and UTF-16 changes."""
import json
import os
import sys
import time
from pathlib import Path
from urllib.parse import unquote, urlparse

documents = {}
versions = {}
symbol_requests = 0
workspace = None

def send(value):
    data = json.dumps(dict(jsonrpc="2.0", **value)).encode()
    sys.stdout.buffer.write(f"Content-Length: {len(data)}\r\n\r\n".encode() + data)
    sys.stdout.buffer.flush()

while True:
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if line == b"\r\n":
            break
        key, value = line.decode().split(":", 1)
        headers[key.lower()] = value.strip()
    message = json.loads(sys.stdin.buffer.read(int(headers["content-length"])))
    method = message.get("method")
    params = message.get("params", {})
    if method == "initialize":
        workspace = Path(unquote(urlparse(params["rootUri"]).path))
        send({"id": message["id"], "result": {"capabilities": {"textDocumentSync": 2, "definitionProvider": True, "hoverProvider": True, "documentSymbolProvider": True, "diagnosticProvider": {}}}})
    elif method == "textDocument/didOpen":
        doc = params["textDocument"]
        documents[doc["uri"]] = doc["text"]
        versions[doc["uri"]] = doc["version"]
    elif method == "textDocument/didChange":
        doc = params["textDocument"]
        assert "range" in params["contentChanges"][0]
        documents[doc["uri"]] = params["contentChanges"][0]["text"]
        versions[doc["uri"]] = doc["version"]
    elif method == "textDocument/didClose":
        documents.pop(params["textDocument"]["uri"], None)
    elif method == "textDocument/diagnostic":
        uri = params["textDocument"]["uri"]
        items = [] if "BROKEN" not in documents[uri] else [{"severity": 1, "message": "fixture syntax error", "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}}}]
        send({"id": message["id"], "result": {"kind": "full", "items": items}})
    elif method == "textDocument/hover":
        uri = params["textDocument"]["uri"]
        if "SLOW" in documents[uri]:
            (workspace / "slow-started").write_text("started")
            time.sleep(20)
        send({"id": message["id"], "result": {"contents": documents[uri], "pid": os.getpid(), "version": versions[uri], "position": params["position"], "documents": documents, "symbol_requests": symbol_requests}})
    elif method in ("textDocument/definition", "textDocument/references", "textDocument/implementation"):
        send({"id": message["id"], "result": [{"uri": params["textDocument"]["uri"], "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}}]})
    elif method == "textDocument/documentSymbol":
        symbol_requests += 1
        text = documents[params["textDocument"]["uri"]]
        result = []
        if text.startswith("fn run"):
            span = {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": len(text.splitlines()[0].encode("utf-16-le")) // 2}}
            selection = {"start": {"line": 0, "character": 3}, "end": {"line": 0, "character": 6}}
            result = [{"name": "Module", "kind": 2, "range": span, "selectionRange": selection, "children": [{"name": "run", "kind": 12, "range": span, "selectionRange": selection}]}]
        send({"id": message["id"], "result": result})
