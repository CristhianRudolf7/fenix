#!/usr/bin/env python3
"""Reconcile or verify the complete firewall of a dedicated Fenix Docker bridge."""
import argparse
import fcntl
import ipaddress
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
from fenix_isolation import PolicyError, identifier

BLOCKED = ("0.0.0.0/8", "10.0.0.0/8", "100.64.0.0/10", "127.0.0.0/8",
           "169.254.0.0/16", "172.16.0.0/12", "192.0.0.0/24", "192.168.0.0/16",
           "198.18.0.0/15", "224.0.0.0/4", "240.0.0.0/4")

def run(*args, **kwargs):
    return subprocess.run(args, check=True, timeout=15, **kwargs)

def network_info(name):
    match = re.fullmatch(r"fenix-(?:user|run)-([0-9a-f-]{36})", name)
    if not match:
        raise PolicyError("Expected a canonical Fenix network name")
    identifier(match[1])
    network = json.loads(subprocess.check_output(
        ["docker", "network", "inspect", name], timeout=15))[0]
    if network.get("EnableIPv6") or network.get("Driver") != "bridge":
        raise PolicyError("Only dedicated IPv4 bridges are supported")
    if not re.fullmatch(r"[0-9a-f]{64}", network["Id"]):
        raise PolicyError("Invalid network identifier")
    bridge = "br-" + network["Id"][:12]
    if network.get("Options", {}).get("com.docker.network.bridge.name", bridge) != bridge:
        raise PolicyError("Custom bridge names are not supported")
    return bridge, "FX" + network["Id"][:12], "FH" + network["Id"][:12]

def resolvers():
    path = Path("/run/systemd/resolve/resolv.conf")
    if not path.exists():
        path = Path("/etc/resolv.conf")
    result = set()
    for line in path.read_text().splitlines():
        parts = line.split()
        if len(parts) >= 2 and parts[0] == "nameserver":
            address = ipaddress.ip_address(parts[1])
            if address.version == 4 and not address.is_loopback:
                result.add(str(address))
    return sorted(result)

def host_addresses():
    interfaces = json.loads(subprocess.check_output(["ip", "-j", "address", "show"], timeout=15))
    return sorted({str(ipaddress.IPv4Address(a["local"])) for interface in interfaces
                   for a in interface.get("addr_info", []) if a["family"] == "inet" and ipaddress.IPv4Address(a["local"]).is_global})

def policy(chain, host_chain, profile, dns, hosts, allowed=None):
    rules = []
    if profile != "offline" and allowed is None:
        for address in dns:
            for protocol in ("udp", "tcp"):
                rules.append(["-A", chain, "-d", address + "/32", "-p", protocol,
                              "-m", protocol, "--dport", "53", "-j", "RETURN"])
    for address in dict.fromkeys((*BLOCKED, *(a + "/32" for a in hosts))):
        rules.append(["-A", chain, "-d", address, "-j", "REJECT", "--reject-with", "icmp-port-unreachable"])
    if profile == "controller":
        rules.append(["-A", chain, "-j", "RETURN"])
    elif profile in ("web", "development"):
        if allowed is None:
            rules.append(["-A", chain, "-p", "tcp", "-m", "multiport", "--dports", "80,443", "-j", "RETURN"])
        else:
            for address in sorted(set(allowed)):
                ip = ipaddress.IPv4Address(address)
                if not ip.is_global or ip.is_multicast:
                    raise PolicyError("Only public execution destinations may be allowed")
                rules.append(["-A", chain, "-d", str(ip) + "/32", "-p", "tcp", "-m", "multiport", "--dports", "80,443", "-j", "RETURN"])
        rules.append(["-A", chain, "-j", "REJECT", "--reject-with", "icmp-port-unreachable"])
    else:
        rules.append(["-A", chain, "-j", "REJECT", "--reject-with", "icmp-port-unreachable"])
    rules.extend([
        ["-A", host_chain, "-m", "conntrack", "--ctstate", "RELATED,ESTABLISHED", "-j", "RETURN"],
        ["-A", host_chain, "-j", "REJECT", "--reject-with", "icmp-port-unreachable"],
    ])
    return rules

def normalized(line):
    return ["ESTABLISHED,RELATED" if p == "RELATED,ESTABLISHED" else p for p in shlex.split(line)]

def reconcile(name, profile="controller", check=False, extra_hosts=(), allowed=None):
    if allowed is not None and profile not in ("web", "development"):
        raise PolicyError("Destination restrictions require an execution network profile")
    bridge, chain, host_chain = network_info(name)
    rules = policy(chain, host_chain, profile, resolvers(), sorted(set(host_addresses()) | {str(ipaddress.IPv4Address(a)) for a in extra_hosts}), allowed)
    jumps = [["-I", "DOCKER-USER", "1", "-i", bridge, "-j", chain],
             ["-I", "INPUT", "1", "-i", bridge, "-j", host_chain]]
    if not check:
        transaction = ["*filter", f":{chain} - [0:0]", f":{host_chain} - [0:0]",
                       f"-F {chain}", f"-F {host_chain}"]
        transaction.extend(shlex.join(rule) for rule in rules)
        for jump in jumps:
            existing = subprocess.check_output(["iptables", "-w", "-S", jump[1]], text=True, timeout=15)
            for line in existing.splitlines():
                if normalized(line) == ["-A", jump[1], *jump[3:]]:
                    transaction.append(shlex.join(["-D", jump[1], *jump[3:]]))
            transaction.append(shlex.join(jump))
        transaction.append("COMMIT")
        run("iptables-restore", "--wait", "5", "--noflush",
            input="\n".join(transaction) + "\n", text=True, stdout=subprocess.DEVNULL)
    for chain_name in (chain, host_chain):
        observed = subprocess.check_output(["iptables", "-w", "-S", chain_name], text=True, timeout=15)
        actual = [normalized(line) for line in observed.splitlines() if line.startswith("-A ")]
        expected = [normalized(shlex.join(r)) for r in rules if r[1] == chain_name]
        if actual != expected:
            raise PolicyError("Firewall policy verification failed")
    for jump in jumps:
        run("iptables", "-w", "-C", jump[1], *jump[3:], stdout=subprocess.DEVNULL)
        lines = subprocess.check_output(["iptables", "-w", "-S", jump[1]], text=True, timeout=15)
        for line in lines.splitlines():
            parsed = normalized(line)
            if parsed[:2] != ["-A", jump[1]]:
                continue
            if parsed == ["-A", jump[1], *jump[3:]]:
                break
            if not (len(parsed) == 6 and parsed[2] == "-i" and parsed[3].startswith("br-") and parsed[3] != bridge
                    and parsed[4] == "-j" and re.fullmatch(r"F[XH][0-9a-f]{12}", parsed[5])):
                raise PolicyError("An earlier firewall rule bypasses tenant isolation")

def remove(name):
    bridge, chain, host_chain = network_info(name)
    info = json.loads(subprocess.check_output(["docker", "network", "inspect", name], timeout=15))[0]
    if info.get("Containers"):
        raise PolicyError("Cannot remove protection from an occupied network")
    transaction = ["*filter"]
    for parent, owned in (("DOCKER-USER", chain), ("INPUT", host_chain)):
        existing = subprocess.check_output(["iptables", "-w", "-S", parent], text=True, timeout=15)
        for line in existing.splitlines():
            if normalized(line) == ["-A", parent, "-i", bridge, "-j", owned]:
                transaction.append(shlex.join(["-D", parent, "-i", bridge, "-j", owned]))
        result = subprocess.run(["iptables", "-w", "-S", owned], capture_output=True, timeout=15)
        if result.returncode == 0:
            transaction.extend([f"-F {owned}", f"-X {owned}"])
    transaction.append("COMMIT")
    run("iptables-restore", "--wait", "5", "--noflush", input="\n".join(transaction) + "\n", text=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("network")
    parser.add_argument("--profile", choices=["controller", "offline", "web", "development"], default="controller")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--remove", action="store_true")
    parser.add_argument("--deny-ip", action="append", default=[], help="Additional host public/NAT IPv4 address")
    parser.add_argument("--restricted", action="store_true", help="Deny destinations except explicitly allowed IPs; disable DNS egress")
    parser.add_argument("--allow-ip", action="append", default=[])
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise PolicyError("Run the network guard as root")
    with open("/run/lock/fenix-network.lock", "a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if args.remove:
            remove(args.network)
        else:
            if args.allow_ip and not args.restricted:
                raise PolicyError("--allow-ip requires --restricted")
            reconcile(args.network, args.profile, args.check, args.deny_ip, args.allow_ip if args.restricted else None)

if __name__ == "__main__":
    main()
