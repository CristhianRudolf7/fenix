"""Operator-owned destination policy. Workloads cannot grant network permissions."""
import ipaddress
import re
import socket
from fenix_isolation import PolicyError


def validate(policy):
    if not isinstance(policy, dict) or set(policy) - {"web", "development"}:
        raise PolicyError("network_hosts must map web/development profiles to host lists")
    for hosts in policy.values():
        if not isinstance(hosts, list) or len(hosts) > 128:
            raise PolicyError("Expected at most 128 exact network hosts per profile")
        for host in hosts:
            if (not isinstance(host, str) or len(host) > 253 or host != host.lower()
                    or not re.fullmatch(r"[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?", host)
                    or any(not label or len(label) > 63 or label.startswith("-") or label.endswith("-") for label in host.split("."))):
                raise PolicyError("Network hosts must be exact lowercase ASCII names, without wildcards, ports or URLs")


def effective(global_policy, account_policy, profile):
    validate(global_policy)
    validate(account_policy)
    if profile == "offline":
        return ()
    global_hosts, account_hosts = global_policy.get(profile), account_policy.get(profile)
    if global_hosts is None and account_hosts is None:
        return None  # Compatibility: public web ports, with private/host IP denial.
    if global_hosts is None:
        return tuple(sorted(set(account_hosts)))
    if account_hosts is None:
        return tuple(sorted(set(global_hosts)))
    return tuple(sorted(set(global_hosts) & set(account_hosts)))


def pin(hosts, denied=()):
    """Freeze public IPv4 destinations for one runner; no guest DNS is needed."""
    if hosts is None:
        return None
    forbidden = {ipaddress.ip_address(ip) for ip in denied}
    result = {}
    for host in hosts:
        try:
            addresses = {ipaddress.ip_address(row[4][0]) for row in socket.getaddrinfo(host, 443, socket.AF_INET, socket.SOCK_STREAM)}
        except OSError as exc:
            raise PolicyError("Could not resolve an approved execution destination") from exc
        if not addresses or any(not ip.is_global or ip.is_multicast or ip in forbidden for ip in addresses):
            raise PolicyError("Execution destination resolves to a forbidden address")
        result[host] = tuple(sorted(str(ip) for ip in addresses))
    return result
