#!/usr/bin/env python3
"""Trusted gVisor startup gate, mounted read-only from the operator's bundle.

This reports a deployment marker, not cryptographic attestation. The host also
checks the actual runtime, Sentry and resource cgroup before granting execution.
"""
import ctypes
import errno
import json
import os
from pathlib import Path
import signal
import sys
import termios


def install_clone3_fallback():
    """Keep clone3 denied with ENOSYS so glibc uses the filtered clone syscall.

    runsc release-20260914.0's OCI translator ignores errnoRet and returns EPERM
    for every ERRNO rule. Its native BPF engine does preserve the errno data.
    This extra inherited filter restores Docker's intended ENOSYS behavior;
    it does not allow clone3 or remove any existing filter. Fail closed if the
    installed runtime cannot enforce the expected result.
    """
    architecture = {"x86_64": 0xC000003E, "aarch64": 0xC00000B7}.get(os.uname().machine)
    if architecture is None:
        raise SystemExit(125)

    class Filter(ctypes.Structure):
        _fields_ = [("code", ctypes.c_ushort), ("jt", ctypes.c_ubyte),
                    ("jf", ctypes.c_ubyte), ("k", ctypes.c_uint)]

    class Program(ctypes.Structure):
        _fields_ = [("length", ctypes.c_ushort), ("filter", ctypes.POINTER(Filter))]

    instructions = (Filter * 7)(
        Filter(0x20, 0, 0, 4),                 # Load seccomp_data.arch.
        Filter(0x15, 1, 0, architecture),
        Filter(0x06, 0, 0, 0),                 # Kill on an unexpected ABI.
        Filter(0x20, 0, 0, 0),                 # Load seccomp_data.nr.
        Filter(0x15, 0, 1, 435),               # clone3 on both supported ABIs.
        Filter(0x06, 0, 0, 0x00050000 | errno.ENOSYS),
        Filter(0x06, 0, 0, 0x7FFF0000),        # Other existing filters still apply.
    )
    program = Program(len(instructions), instructions)
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(38, 1, 0, 0, 0) != 0 or libc.prctl(22, 2, ctypes.byref(program), 0, 0) != 0:
        raise SystemExit(125)
    if libc.syscall(ctypes.c_long(435), ctypes.c_void_p(), ctypes.c_size_t(0)) != -1 or ctypes.get_errno() != errno.ENOSYS:
        raise SystemExit(125)


def main():
    signal.alarm(70)
    marker = Path("/proc/gvisor/kernel_is_gvisor").is_file()
    if not marker:
        raise SystemExit(125)
    install_clone3_fallback()
    # Docker exec starts a fresh task instead of inheriting the initial task's
    # filters. Owner-authorized internal relays use this same compatibility gate.
    if len(sys.argv) > 2 and sys.argv[1] == "--exec":
        signal.alarm(0)
        os.execvpe(sys.argv[2], sys.argv[2:], os.environ)
    terminal = os.isatty(0)
    previous = termios.tcgetattr(0) if terminal else None
    if terminal:
        current = termios.tcgetattr(0)
        current[3] &= ~termios.ECHO
        termios.tcsetattr(0, termios.TCSANOW, current)
    payload = dict(backend="gvisor", kernel=os.uname().release)
    os.write(1, b"FENIX-SANDBOX/1 " + json.dumps(payload).encode() + b"\n")
    for byte in b"FENIX-START\n":
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
