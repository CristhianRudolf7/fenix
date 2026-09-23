# Self-hosted workspaces with gVisor

[Español](../es/self-hosted-gvisor.md) · [Account and execution policy](execution-isolation.md)

Fenix now defaults to **gVisor `runsc` with the `systrap` platform** for its multi-user execution service. This requires neither `/dev/kvm` nor nested virtualization. It runs on a compatible Linux server, including an ordinary cloud VM. There is no E2B account, sandbox API key or hosted sandbox subscription. Compute, storage, updates and backups remain the operator's responsibility.

This is an application-kernel sandbox, not a hardware VM. The account controller retains credentials, conversations and scheduling; untrusted shell commands, terminals, skill scripts, language servers and Chromium execute in separate gVisor sandboxes. Native file/document operations remain in the trusted Fenix controller. The runtime is upstream gVisor; Fenix supplies its own authorization, workspace lifecycle, mounting policy, quotas, network restrictions and startup verification.

## Runtime selection and upgrades

Use `packaging/linux/execution.example.json`: `backend` is `gvisor`, `runtime` is `fenix-runsc`, and `gvisor_manifest` names the private pinned bundle. Omitting `backend` now selects gVisor; old configurations that intentionally retain the OCI baseline must explicitly use `backend: oci` and `runtime: runc`. This is a fail-closed configuration change, not an automatic migration of live accounts.

The older `backend: microvm` remains an explicit optional Kata adapter for hosts with KVM. It is not selected by default, and gVisor never falls back to it, runc or host execution. A missing runtime, changed binary, unsafe registration or failed startup check rejects the execution.

Preparation pins the complete installed release: `runsc`, every file in the adjacent `gvisor-bin/` directory and the trusted startup gate. The tested candidate is `release-20260914.0`; release files must stay together. Current releases require sidecar binaries, including `gvisor_sentry`. A standalone legacy `runsc` installation is insufficient. Verify upstream release checksums before preparation; Fenix's SHA-256 manifest identifies local bytes, not their origin.

The Docker alias must point to that exact executable and the fixed argument list. Fenix selects systrap, the sandbox network stack, shared bind-mount access and no write overlays. Direct filesystem access from Sentry is disabled: file access passes through the separate Gofer. Host Unix sockets, host FIFOs, raw packet writes, privileged character-device passthrough, debug profiling and annotation flag overrides are disabled. OCI seccomp is explicitly enabled inside the sandbox, in addition to gVisor's host seccomp protections. Strict sidecar presence and release matching prevent legacy embedded fallbacks.

## Files, tools and account boundaries

`persistent_environments: true` gives regular shell work a private tool home under the account's bounded filesystem, distinct for each project path. Python virtualenvs, npm prefixes, Cargo directories, caches and project files survive sandbox removal. Each execution starts a fresh sandbox; arbitrary root filesystem modifications and running processes do not persist. The operator-built image supplies system packages; `Dockerfile.workspace` adds document and multimedia applications when needed.

Bind mounts remain in shared mode so native Fenix file tools and a running sandbox see each other's changes. Read-only exports are remounted read-only on the host as well as marked read-only in the sandbox. Language servers receive the selected project and tool home read-only, with no external network. Browsers receive only their own profile and downloads. The workload receives neither the Docker socket, execution service socket, controller secrets nor another account's files.

The existing account UID/GID, fixed-capacity filesystem, credential broker, monthly execution-time reservation, admission limits and revocation rules remain in force. Persistent files are host-local and require backups; this change does not provide cross-host failover or replicated storage.

## Startup and resource checks

Before user code starts, a host-pinned Python gate checks the gVisor marker and waits for a grant. Fenix verifies Docker's registration and selected runtime, the actual host Sentry executable, its exact account/container cgroup, and host seccomp/no-new-privileges. It then rechecks account authorization and deadline before granting execution. The marker alone is not treated as proof. These checks detect configuration mistakes; they are not cryptographic attestation or an escape-resistance certification.

The hard memory/CPU/PID limits apply to the host sandbox cgroup. Guest cgroup files are virtual and must not be used as evidence of host enforcement. The memory limit includes runtime overhead; it is not a promise that the entire number is available to application memory. Admission additionally reserves `runtime_overhead_mib` (128 MiB by default) as conservative headroom. Account slices cap aggregate workloads and controllers. Limits between individual processes within one gVisor sandbox are not equivalent to independent host containers.

Cancellation kills the whole sandbox. When Docker removal fails, the recorded host cgroup provides an emergency stop; cleanup remains pending until removal is confirmed. Existing public-web-only egress and private/metadata/host destination blocking still apply. There are no published task/CDP ports; authenticated relays connect inside the exact sandbox.

Chromium retains its inner sandbox. The browser's OCI seccomp policy executes inside gVisor; the runc-specific AppArmor browser profile is not applied to Sentry as if it were Chromium. No `--no-sandbox`, privileged workload or host networking fallback is added.

The tested release's [OCI seccomp translator](https://github.com/google/gvisor/blob/release-20260914.0/runsc/specutils/seccomp/seccomp.go) returns EPERM for ERRNO rules even when Docker specifies ENOSYS. This breaks glibc's thread-creation fallback. The trusted gate therefore installs an additional native BPF filter that keeps `clone3` denied with ENOSYS, checks that result, and preserves all other existing restrictions. Internal `docker exec` relays pass through the same filter setup because they do not inherit the initial task's filters. Acceptance tests cover working threads while both `clone3` and unauthorized namespace creation remain denied. Reassess this workaround on runtime upgrades; do not disable OCI seccomp to bypass a compatibility failure.

## Installation

Use Linux 5.6+ on x86_64 or aarch64, systemd, Docker with cgroup v2/systemd, the existing firewall/storage dependencies and Python 3.11+. Run the following after installing a **complete checksum-verified upstream gVisor release** in a root-owned location readable and executable by unprivileged runtime processes. Keep `gvisor-bin/` beside `runsc`. The [upstream installation guide](https://gvisor.dev/docs/user_guide/install/) documents release archives and packages. Installing its Debian package can modify Docker configuration; review that behavior before applying it to an existing host.

```sh
sudo scripts/install-isolation.sh
sudo python3 /opt/fenix/isolation/prepare-gvisor.py \
  --runsc /usr/local/bin/runsc --register-runtime
sudo systemctl reload docker
sudo python3 /opt/fenix/isolation/gvisor-preflight.py
```

`prepare-gvisor.py` does not download binaries or restart Docker. It preserves existing Docker settings and the default runtime, refuses a conflicting `fenix-runsc` alias, validates the proposed daemon configuration and saves a backup. Without `--register-runtime`, it writes only a manifest and configuration fragment. An existing manifest is not overwritten: stop workloads and prepare an explicit upgrade. If preparation fails after writing a valid bundle, inspect its fragment/backups before retrying.

Copy and customize the execution example using the [account migration procedure](execution-isolation.md#installation-and-migration). Preserve existing account UUIDs and volumes. Select a real immutable image digest and size quotas for the host's capacity. Start the service only after preparation; a repository update does not change a running service or migrate existing accounts.

## Verification and scope

The [validation record](../research/self-hosted-gvisor-validation-2026-09-20.json) records 45 passing policy tests, 15 passing real gVisor scenarios and one passing OCI relay regression. The runtime was exercised on this x86_64 host without `/dev/kvm`, then its temporary Docker registration and assets were removed. Existing production accounts were not migrated. The record includes the initial failures, fixes, tested release/image hashes and remaining coverage limits.

Policy tests run without gVisor installed:

```sh
python3 -m unittest discover -s tests/isolation -v
```

Real acceptance uses synthetic accounts, temporary mounts, isolated networks and account cgroups, without KVM:

```sh
sudo env FENIX_GVISOR_TESTS=1 \
  FENIX_GVISOR_MANIFEST=/etc/fenix/gvisor/manifest.json \
  FENIX_TEST_IMAGE=fenix-tenant:local \
  python3 -m unittest discover -s tests/isolation -p test_gvisor_acceptance.py -v
```

Explicit opt-in with a missing runtime is a failure. The suite exercises authorization, real host limits, persistent tools, shared file coherence, read-only exports, terminals, Rust transport, revocation, descendant termination, private network relays and Chromium/CDP. Repeat it on the deployment host and after runtime/image updates. Passing it does not establish arbitrary application compatibility, multi-tenant load capacity or immunity to sandbox escapes. In particular, validate the actual LibreOffice/media image separately; Linux applications relying on unsupported kernel facilities can require adaptation.

RAM snapshots, process suspend/resume, live migration, GPU and Windows desktop support are not implemented by this adapter. Local single-user mode remains a separate mode and is not advertised as a multi-user sandbox.

Primary references: [platforms](https://gvisor.dev/docs/user_guide/platforms/), [filesystem sharing](https://gvisor.dev/docs/user_guide/filesystem/), [application compatibility](https://gvisor.dev/docs/user_guide/compatibility/), [Docker runtime registration](https://docs.docker.com/reference/cli/dockerd/#configure-container-runtimes).
