"""OCI adapter. Policy is enforced outside containers, never by workload scripts."""
from __future__ import annotations
import asyncio
import http.client
import json
import os
import re
import socket
from pathlib import Path
import subprocess
import sys
import stat
from fenix_isolation import PolicyError, account_slice, image_id, identifier, private_json, private_write


def stop_recorded_cgroup(directory):
    """Emergency stop independent of the Docker API, scoped to a recorded cgroup."""
    metadata = directory / "container.json"
    if not metadata.exists():
        return
    info = private_json(metadata)
    owner = identifier(info["owner"]).replace("-", "")
    container = info["container"]
    if not re.fullmatch(r"[0-9a-f]{64}", container):
        raise PolicyError("Invalid recorded container identity")
    if "vm_cgroup" in info or "sandbox_cgroup" in info:
        group = info.get("sandbox_cgroup", info.get("vm_cgroup"))
        prefix = "/fenixacct.slice/fenixacct-" + owner + ".slice/"
        if not isinstance(group, str) or not group.startswith(prefix) or ".." in Path(group).parts or container not in group:
            raise PolicyError("Invalid recorded isolation cgroup")
        if "sandbox_cgroup" in info and group != prefix + "docker-" + container + ".scope":
            raise PolicyError("Invalid recorded gVisor cgroup")
        control = Path("/sys/fs/cgroup") / group.lstrip("/") / "cgroup.kill"
    else:
        control = Path("/sys/fs/cgroup/fenixacct.slice") / ("fenixacct-" + owner + ".slice") / ("docker-" + container + ".scope") / "cgroup.kill"
    try:
        fd = os.open(control, os.O_WRONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    except FileNotFoundError:
        return
    try:
        os.write(fd, b"1")
    finally:
        os.close(fd)


def stop_abandoned_cgroups(state):
    directory = Path(state) / "mounts"
    if directory.exists():
        for entry in directory.iterdir():
            identifier(entry.name)
            stop_recorded_cgroup(entry)

CONNECT_RELAY = '''import os,socket,sys,threading
s=socket.create_connection(("127.0.0.1",int(sys.argv[1])),5)
s.settimeout(None)
os.write(1,b"FENIX-CONNECTED\\n")
def receive():
    try:
        while True:
            data=s.recv(65536)
            if not data: break
            view=memoryview(data)
            while view: view=view[os.write(1,view):]
    finally: os._exit(0)
threading.Thread(target=receive,daemon=True).start()
try:
    while True:
        data=os.read(0,65536)
        if not data: break
        s.sendall(data)
finally: s.close()
'''

class TunnelWriter:
    def __init__(self, process):
        self.process = process
        self.closing = None
    def write(self, data):
        self.process.stdin.write(data)
    async def drain(self):
        await self.process.stdin.drain()
    def close(self):
        if self.closing is None:
            self.process.stdin.close()
            self.closing = asyncio.create_task(self.finish())
    async def wait_closed(self):
        self.close()
        await self.closing
    async def finish(self):
        try:
            await asyncio.wait_for(self.process.wait(), 5)
        except asyncio.TimeoutError:
            self.process.kill()
            await self.process.wait()


class Docker:
    kind = "oci"
    default_runtime = "runc"
    def __init__(self, config):
        self.config = config
        self.egress_policies = {}
        endpoint = subprocess.check_output(["docker", "context", "inspect", "--format", "{{.Endpoints.docker.Host}}"], text=True, timeout=15).strip()
        if endpoint != "unix:///var/run/docker.sock" or os.environ.get("DOCKER_HOST", endpoint) != "unix:///var/run/docker.sock":
            raise PolicyError("The execution service requires the local Docker Unix socket")
        self.image = image_id(config["image"])
        self.runtime = config.get("runtime", self.default_runtime)
        info = json.loads(subprocess.check_output(["docker", "info", "--format", "{{json .}}"], timeout=20))
        if info.get("CgroupVersion") != "2" or info.get("CgroupDriver") != "systemd":
            raise PolicyError("Execution requires Docker with systemd and cgroup v2")
        if self.runtime not in info.get("Runtimes", {}):
            raise PolicyError("Configured isolation runtime is unavailable; no fallback is permitted")
        options = info.get("SecurityOptions", [])
        if not any("seccomp" in option for option in options):
            raise PolicyError("Docker seccomp enforcement is required")
        self.guard = Path(__file__).resolve().parents[1] / "tenant-network-guard.py"
        self.deny_args = [part for address in config.get("host_public_ips", []) for part in ("--deny-ip", address)]
        self.instance = config.get("instance", "default")
        if not self.instance.isalnum() or len(self.instance) > 32:
            raise PolicyError("Invalid service instance identifier")
        state_dir = Path(config["state_dir"])
        state_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
        if state_dir.is_symlink() or state_dir.stat().st_uid != 0 or state_dir.stat().st_mode & 0o077:
            raise PolicyError("Execution state must be private and root-owned")
        self.mount_dir = state_dir / "mounts"
        self.mount_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
        if self.mount_dir.is_symlink() or self.mount_dir.stat().st_uid != 0 or self.mount_dir.stat().st_mode & 0o077:
            raise PolicyError("Mount staging must be private and root-owned")

    def browser_security_args(self):
        profile = self.config.get("browser_seccomp")
        apparmor = self.config.get("browser_apparmor")
        if not profile or not isinstance(apparmor, str) or not re.fullmatch(r"fenix-browser-[a-z0-9-]{1,64}", apparmor):
            raise PolicyError("Install and configure the dedicated browser sandbox profiles")
        path = Path(profile)
        metadata = path.lstat()
        if not path.is_absolute() or not stat.S_ISREG(metadata.st_mode) or metadata.st_uid != 0 or metadata.st_mode & 0o022:
            raise PolicyError("Browser seccomp policy must be a protected root-owned file")
        return ["--security-opt", "seccomp=" + str(path), "--security-opt", "apparmor=" + apparmor]

    async def command(self, *args, timeout=30):
        creation = asyncio.create_task(asyncio.create_subprocess_exec(*args, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE))
        try:
            process = await asyncio.shield(creation)
        except asyncio.CancelledError:
            process = await creation
            if process.returncode is None:
                process.kill()
            await process.communicate()
            raise
        try:
            output, error = await asyncio.wait_for(process.communicate(), timeout)
        except BaseException:
            if process.returncode is None:
                process.kill()
            await process.communicate()
            raise
        if process.returncode:
            # Never propagate command arguments, environment values or image output.
            raise PolicyError(f"Execution infrastructure command failed: {args[0]}")
        if len(output) > 1024 * 1024:
            raise PolicyError("Infrastructure response exceeds limit")
        return output

    def create_args(self, execution, launch, parent, sources=None):
        name = "fenix-run-" + execution
        network = "none" if launch.profile == "offline" else name
        args = ["docker", "create", "--name", name, "--label", "fenix.execution=" + self.instance,
                "--label", "fenix.owner=" + launch.owner, "--label", "fenix.generation=" + execution,
                "--restart", "no", "--init", "--read-only", "--cap-drop", "ALL",
                "--security-opt", "no-new-privileges", "--user", f"{launch.uid}:{launch.gid}",
                "--runtime", self.runtime, "--cgroup-parent", parent, "--network", network,
                "--sysctl", "net.ipv6.conf.all.disable_ipv6=1", "--sysctl", "net.ipv6.conf.default.disable_ipv6=1",
                "--ipc", "private", "--shm-size", "128m", "--interactive",
                "--tmpfs", f"/tmp:rw,nosuid,nodev,size={launch.limits.tmp_mib}m,mode=1777",
                "--env", "HOME=/tmp", "--env", "TMPDIR=/tmp",
                "--env", "XDG_CACHE_HOME=/tmp/cache", "--env", "XDG_CONFIG_HOME=/tmp/config",
                "--env", "PATH=/usr/local/bin:/usr/bin:/bin", "--env", "LANG=C.UTF-8",
                *launch.limits.docker()]
        if launch.tty:
            args.append("--tty")
        if launch.egress_hosts is not None and launch.profile != "offline":
            if launch.egress_pins is None:
                raise PolicyError("Restricted execution requires pinned network destinations")
            # Do not let Docker's embedded resolver forward arbitrary guest queries.
            args.extend(["--dns", "127.0.0.1"])
            for host, addresses in launch.egress_pins.items():
                for address in addresses:
                    args.extend(["--add-host", f"{host}:{address}"])
        if launch.kind == "browser":
            args.extend(self.browser_security_args())
        if launch.readonly and all(m.path != launch.cwd for m, _ in launch.mounts):
            args.extend(["--tmpfs", f"{launch.cwd}:rw,nosuid,nodev,noexec,size=1m,uid={launch.uid},gid={launch.gid},mode=700"])
        for index, (mount, readonly) in enumerate(launch.mounts):
            source = sources[index] if sources is not None else mount.source
            args.extend(["--mount", f"type=bind,src={source},dst={mount.path},bind-propagation=rprivate" + (",readonly" if readonly else "")])
        for key, value in launch.environment.items():
            args.extend(["--env", f"{key}={value}"])
        if launch.home is not None:
            from .workspaces import variables
            for key, value in variables(launch.home).items():
                args.extend(["--env", f"{key}={value}"])
        args.extend(["--workdir", str(launch.cwd), "--entrypoint", launch.argv[0], self.image, *launch.argv[1:]])
        return args

    async def create(self, execution, launch):
        from .egress import pin
        launch.egress_pins = await asyncio.wait_for(asyncio.to_thread(
            pin, launch.egress_hosts, self.config.get("host_public_ips", [])), 15)
        if launch.profile != "offline":
            self.egress_policies[execution] = launch.egress_pins
        parent = await asyncio.to_thread(account_slice, launch.owner, launch.account_limits)
        if launch.profile != "offline":
            await self.command("docker", "network", "create", "--opt", "com.docker.network.bridge.enable_icc=false",
                               "--label", "fenix.execution=" + self.instance, "fenix-run-" + execution)
            await self.guard_check(execution, launch.profile, repair=True)
        # runc does not accept procfs file-descriptor paths as bind sources on
        # every supported version. Pin them into a root-only mount namespace
        # directory visible to dockerd before handing over stable paths.
        directory = self.mount_dir / execution
        directory.mkdir(mode=0o700)
        sources = []
        for index, (mount, readonly) in enumerate(launch.mounts):
            target = directory / str(index)
            if stat.S_ISDIR(os.fstat(mount.fd).st_mode):
                target.mkdir(mode=0o700)
            else:
                target.touch(mode=0o600, exist_ok=False)
            await self.command("mount", "--bind", mount.source, str(target))
            if readonly:
                # Enforce export permissions in the host as well as the guest.
                await self.command("mount", "-o", "remount,bind,ro,nosuid,nodev", str(target))
            sources.append(str(target))
        container = (await self.command(*self.create_args(execution, launch, parent, sources))).decode().strip()
        if not re.fullmatch(r"[0-9a-f]{64}", container):
            raise PolicyError("Docker did not return a canonical container identity")
        private_write(directory / "container.json", json.dumps(dict(owner=launch.owner, container=container)).encode())

    async def guard_check(self, execution, profile, repair=False):
        if profile != "offline":
            if execution not in self.egress_policies:
                raise PolicyError("Execution network policy is missing")
            pins = self.egress_policies[execution]
            restrictions = [] if pins is None else ["--restricted", *[
                part for ip in sorted({ip for addresses in pins.values() for ip in addresses})
                for part in ("--allow-ip", ip)]]
            await self.command(sys.executable, str(self.guard), "fenix-run-" + execution,
                               "--profile", profile, *self.deny_args, *restrictions, *([] if repair else ["--check"]), timeout=60)

    async def attach(self, execution):
        return await asyncio.create_subprocess_exec("docker", "start", "--attach", "--interactive", "fenix-run-" + execution,
                                                    stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
                                                    stderr=asyncio.subprocess.PIPE)

    async def release(self, execution, process):
        """OCI starts on attachment; stronger adapters have a startup gate."""
        return

    async def inspect(self, execution):
        return json.loads(await self.command("docker", "inspect", "fenix-run-" + execution))[0]

    async def resize(self, execution, rows, columns):
        identifier(execution)
        if any(type(value) is not int or not 1 <= value <= 500 for value in (rows, columns)):
            raise PolicyError("Invalid terminal dimensions")
        def request():
            connection = http.client.HTTPConnection("localhost", timeout=5)
            stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            stream.settimeout(5)
            try:
                stream.connect("/var/run/docker.sock")
                connection.sock = stream
                connection.request("POST", f"/containers/fenix-run-{execution}/resize?h={rows}&w={columns}", body=b"")
                response = connection.getresponse()
                if response.status not in (200, 204):
                    raise PolicyError("Runner terminal resize failed")
                response.read(4096)
            finally:
                connection.close()
                stream.close()
        await asyncio.to_thread(request)

    async def connect(self, execution, port):
        info = await self.inspect(execution)
        if not info["State"]["Running"]:
            raise PolicyError("Runner is not running")
        # Connect inside this exact guest's loopback namespace. No debugging or
        # development port is published on the host or exposed to another runner.
        process = await asyncio.create_subprocess_exec("docker", "exec", "--interactive", "fenix-run-" + execution,
                    *self.exec_argv(["/usr/bin/python3", "-I", "-c", CONNECT_RELAY, str(port)]),
                    stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.DEVNULL)
        writer = TunnelWriter(process)
        try:
            if await asyncio.wait_for(process.stdout.readline(), 8) != b"FENIX-CONNECTED\n":
                raise PolicyError("Runner service is not listening")
            return process.stdout, writer
        except BaseException:
            writer.close()
            raise

    def exec_argv(self, argv):
        return argv

    async def destroy(self, execution):
        name = "fenix-run-" + execution
        # rm --force terminates the complete container/cgroup, including setsid children.
        try:
            await self.command("docker", "rm", "--force", name)
        except Exception:
            stop_recorded_cgroup(self.mount_dir / execution)
            raise  # Keep metadata until Docker confirms removal; the monitor retries.

    async def absent(self, execution):
        names = (await self.command("docker", "ps", "--all", "--format", "{{.Names}}")).decode().splitlines()
        return "fenix-run-" + execution not in names

    async def remove_network(self, execution):
        name = "fenix-run-" + execution
        names = (await self.command("docker", "network", "ls", "--format", "{{.Name}}")).decode().splitlines()
        if name not in names:
            self.egress_policies.pop(execution, None)
            return
        await self.command(sys.executable, str(self.guard), name, "--remove", timeout=60)
        await self.command("docker", "network", "rm", name)
        self.egress_policies.pop(execution, None)

    async def cleanup_mounts(self, execution):
        directory = self.mount_dir / execution
        if not directory.exists():
            return
        for target in directory.iterdir():
            if target.name == "container.json":
                continue
            if not target.name.isdigit() or target.is_symlink():
                raise PolicyError("Unexpected execution mount entry")
            if os.path.ismount(target):
                await self.command("umount", str(target))
            if target.is_dir():
                target.rmdir()
            else:
                target.unlink()
        (directory / "container.json").unlink(missing_ok=True)
        directory.rmdir()

    async def cleanup_orphans(self):
        names = (await self.command("docker", "ps", "--all", "--filter", "label=fenix.execution=" + self.instance,
                                    "--format", "{{.Names}}")).decode().splitlines()
        from fenix_isolation import identifier
        for name in names:
            if not name.startswith("fenix-run-"):
                raise PolicyError("Unexpected container with execution service label")
            execution = identifier(name.removeprefix("fenix-run-"))
            await self.destroy(execution)
        networks = (await self.command("docker", "network", "ls", "--filter", "label=fenix.execution=" + self.instance,
                                       "--format", "{{.Name}}")).decode().splitlines()
        for name in networks:
            execution = identifier(name.removeprefix("fenix-run-"))
            await self.remove_network(execution)
        for directory in self.mount_dir.iterdir():
            await self.cleanup_mounts(identifier(directory.name))

    async def watch_controllers(self, policy):
        names = (await self.command("docker", "ps", "--filter", "label=fenix.controller=" + self.instance,
                                    "--format", "{{.Names}}")).decode().splitlines()
        try:
            accounts = policy.current()
        except Exception:
            accounts = {}  # Registry failure revokes active controllers too.
        for name in names:
            from fenix_isolation import identifier
            owner = identifier(name.removeprefix("fenix-user-"))
            try:
                account = accounts.get(owner)
                if not account or account.get("disabled", False) or owner not in policy.accounts:
                    raise PolicyError("Account disabled")
                info = json.loads(await self.command("docker", "inspect", name))[0]
                env = dict(value.split("=", 1) for value in info["Config"].get("Env", []) if "=" in value)
                if env.get("FENIX_AUTH_EPOCH") != str(account.get("auth_epoch", 0)) or env.get("FENIX_UPSTREAM_TOKEN") != account["upstream_token"]:
                    raise PolicyError("Controller authorization revoked")
                await self.command(sys.executable, str(self.guard), name, "--check", *self.deny_args, timeout=30)
            except Exception:
                await self.command("docker", "stop", "--time", "1", name)
