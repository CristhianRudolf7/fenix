#!/usr/bin/env python3
"""Guest startup gate. Mounted from the pinned host bundle, never user supplied.

The workload starts only after the host validates this fresh guest and the VMM
cgroup. This is a deployment check, not cryptographic remote attestation.
"""
import json
import os
from pathlib import Path
import signal
import sys
import termios


def main():
    signal.alarm(70)
    terminal = os.isatty(0)
    previous = termios.tcgetattr(0) if terminal else None
    if terminal:
        current = termios.tcgetattr(0)
        current[3] &= ~termios.ECHO
        termios.tcsetattr(0, termios.TCSANOW, current)
    payload = dict(boot_id=Path("/proc/sys/kernel/random/boot_id").read_text().strip(), kernel=os.uname().release)
    os.write(1, b"FENIX-MICROVM/1 " + json.dumps(payload).encode() + b"\n")
    expected = b"FENIX-START\n"
    for byte in expected:
        if os.read(0, 1) != bytes([byte]):
            raise SystemExit(125)
    if terminal:
        termios.tcsetattr(0, termios.TCSANOW, previous)
    signal.alarm(0)
    if len(sys.argv) < 2:
        raise SystemExit(125)
    os.execvpe(sys.argv[1], sys.argv[1:], os.environ)


if __name__ == "__main__":
    main()
