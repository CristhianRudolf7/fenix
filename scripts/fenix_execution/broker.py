"""Owner-scoped HTTPS integration broker with pinned DNS and exact operations."""
from __future__ import annotations
import asyncio
import base64
import http.client
import ipaddress
import json
import re
import socket
import ssl
import time
from urllib.parse import urlsplit
from fenix_isolation import PolicyError, account_fingerprint, private_json

MAX_BODY = 256 * 1024
MAX_RESPONSE = 512 * 1024


def public_addresses(host, port, denied=()):
    addresses = list(dict.fromkeys(item[4][0] for item in socket.getaddrinfo(host, port, type=socket.SOCK_STREAM)))
    if not addresses or any(not ipaddress.ip_address(address).is_global for address in addresses):
        raise PolicyError("Integration DNS contains a non-public destination")
    blocked = {ipaddress.ip_address(address) for address in denied}
    if any(ipaddress.ip_address(address) in blocked for address in addresses):
        raise PolicyError("Integration resolves to a configured host or NAT address")
    # Reject host public addresses too. A broker cannot hairpin into host services.
    import subprocess
    interfaces = json.loads(subprocess.check_output(["ip", "-j", "address", "show"], timeout=5))
    local = {a["local"] for item in interfaces for a in item.get("addr_info", [])}
    if any(address in local for address in addresses):
        raise PolicyError("Integration resolves to this host")
    return addresses


class PinnedHTTPS(http.client.HTTPSConnection):
    def __init__(self, hostname, address):
        super().__init__(hostname, port=443, timeout=15, context=ssl.create_default_context())
        self.address = address

    def connect(self):
        raw = socket.create_connection((self.address, 443), self.timeout)
        try:
            self.sock = self._context.wrap_socket(raw, server_hostname=self.host)
        except BaseException:
            raw.close()
            raise


class Broker:
    def __init__(self, policy, records, config):
        self.policy, self.records = policy, records
        self.path = config["integrations_file"]
        self.denied_addresses = tuple(config.get("host_public_ips", ()))
        self.slots = asyncio.Semaphore(8)
        self.account_slots = {}
        self.usage = {}  # Survives requests; lifetime limits also persist in SQLite.
        records.db.execute("CREATE TABLE IF NOT EXISTS broker_usage(owner TEXT, connection TEXT, operation TEXT, count INTEGER NOT NULL, PRIMARY KEY(owner,connection,operation))")
        records.db.commit()

    def connections(self, account):
        # This file belongs to the host; IDs never select another owner's record.
        data = private_json(self.path)
        connections = data.get("accounts", {}).get(account["id"], {})
        if not isinstance(connections, dict) or len(connections) > 64:
            raise PolicyError("Invalid integration configuration")
        return connections

    async def dispatch(self, account, request):
        owner = account["id"]
        connections = self.connections(account)
        if request["operation"] == "integrations":
            return {"integrations": [{"id": name, "operations": [{"id": key, "description": spec.get("description", ""), "method": spec.get("method", "GET")} for key, spec in entry.get("operations", {}).items()]}
                                     for name, entry in connections.items() if not entry.get("disabled", False)]}
        if set(request) - {"operation", "account_id", "credential", "auth_epoch", "connection", "action", "body", "task"}:
            raise PolicyError("Unknown integration fields")
        name, action = request.get("connection"), request.get("action")
        if not isinstance(name, str) or not isinstance(action, str):
            raise PolicyError("Integration operation not found")
        entry = connections.get(name)
        operation = entry.get("operations", {}).get(action) if entry else None
        if not entry or entry.get("disabled", False) or not operation or entry.get("expires", 0) <= time.time():
            raise PolicyError("Integration operation unavailable or expired")
        if not re.fullmatch(r"[A-Za-z0-9:_-]{1,128}", request.get("task", "")):
            raise PolicyError("Integration requires a task identity")
        body = request.get("body")
        # Exact top-level fields and values scope resources and tenant identifiers.
        # No arbitrary URL, query, headers, method, remote account or OAuth refresh input.
        fields = operation.get("body_fields", [])
        fixed = operation.get("fixed_body", {})
        if body is not None and (not isinstance(body, dict) or set(body) - set(fields) or set(body) & set(fixed)):
            raise PolicyError("Integration payload is outside the configured grant")
        payload = {**(body or {}), **fixed} if body is not None or fixed else None
        encoded = json.dumps(payload, separators=(",", ":"), allow_nan=False).encode() if payload is not None else None
        if encoded and len(encoded) > MAX_BODY:
            raise PolicyError("Integration request exceeds size limit")
        key = (owner, name, action)
        maximum = operation.get("max_calls", 100)
        minute_maximum = operation.get("calls_per_minute", 10)
        if type(maximum) is not int or not 1 <= maximum <= 1000000 or type(minute_maximum) is not int or not 1 <= minute_maximum <= 60:
            raise PolicyError("Invalid integration usage limit")
        window, used = self.usage.get(key, (time.monotonic(), 0))
        if time.monotonic() - window > 60:
            window, used = time.monotonic(), 0
        count = self.records.db.execute("SELECT count FROM broker_usage WHERE owner=? AND connection=? AND operation=?", key).fetchone()
        if used >= minute_maximum or (count and count[0] >= maximum):
            raise PolicyError("Integration usage grant exhausted")
        self.usage[key] = (window, used + 1)
        self.records.db.execute("INSERT INTO broker_usage VALUES(?,?,?,1) ON CONFLICT(owner,connection,operation) DO UPDATE SET count=count+1", key)
        self.records.db.commit()  # Failed/ambiguous writes consume their allowance. Never automatically retry.
        fingerprint = account_fingerprint(account)
        slots = self.account_slots.setdefault(owner, asyncio.Semaphore(2))
        async with slots, self.slots:
            if not self.policy.active(owner, fingerprint) or self.connections(account).get(name) != entry:
                raise PolicyError("Integration grant revoked")
            response = await asyncio.to_thread(self.perform, entry, operation, encoded)
            if not self.policy.active(owner, fingerprint) or self.connections(account).get(name) != entry:
                raise PolicyError("Integration grant revoked during request")
            return response

    def perform(self, entry, operation, body):
        target = urlsplit(operation["url"])
        if (target.scheme != "https" or not target.hostname or target.port not in (None, 443)
                or target.username or target.password or target.fragment or "\\" in operation["url"]
                or any(ord(c) < 33 for c in operation["url"])):
            raise PolicyError("Integration operation requires a fixed HTTPS URL")
        method = operation.get("method", "GET")
        if method not in {"GET", "POST", "PUT", "PATCH", "DELETE"} or (method == "GET" and body is not None):
            raise PolicyError("Invalid integration method or body")
        header = entry.get("credential_header", "Authorization")
        if header not in ("Authorization", "X-API-Key"):
            raise PolicyError("Unsupported credential header")
        secret = entry.get("secret", "")
        if not isinstance(secret, str) or not 16 <= len(secret) <= 4096 or any(ord(c) < 32 or ord(c) > 126 for c in secret):
            raise PolicyError("Invalid integration credential")
        host = entry.get("host")
        if target.hostname != host:
            raise PolicyError("Credential host does not match the operation")
        address = public_addresses(host, 443, self.denied_addresses)[0]
        connection = PinnedHTTPS(host, address)
        try:
            headers = {header: secret, "Accept": "application/json", "Accept-Encoding": "identity", "Content-Type": "application/json"}
            path = target.path or "/"
            if target.query:
                path += "?" + target.query
            connection.request(method, path, body=body, headers=headers)
            response = connection.getresponse()
            chunks, size, deadline = [], 0, time.monotonic() + 30
            while True:
                if time.monotonic() >= deadline:
                    raise PolicyError("Integration response deadline exceeded")
                chunk = response.read1(min(16384, MAX_RESPONSE + 1 - size))
                if not chunk:
                    break
                chunks.append(chunk)
                size += len(chunk)
                if size > MAX_RESPONSE:
                    break
            data = b"".join(chunks)
            if len(data) > MAX_RESPONSE or 300 <= response.status < 400:
                raise PolicyError("Integration response too large or redirected")
            # No response headers are forwarded; reject literal or common encoded reflection.
            values = [secret.encode(), secret.removeprefix("Bearer ").encode()]
            if any(v in data or base64.b64encode(v) in data for v in values):
                raise PolicyError("Integration response reflected a credential")
            try:
                value = json.loads(data)
            except (ValueError, UnicodeError):
                raise PolicyError("Integration response must be JSON") from None
            return {"status": response.status, "response": value}
        finally:
            connection.close()
