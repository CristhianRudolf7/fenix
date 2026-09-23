# Self-hosted virtual workspaces

[Español](../es/self-hosted-microvms.md) · [Execution policy](execution-isolation.md)

**Optional backend.** The default is now [gVisor with systrap](self-hosted-gvisor.md), which does not require KVM. This page describes the separately selected Kata backend.

Fenix can run shell commands, terminals, language servers and Chromium in hardware-isolated Linux VMs using **Kata Containers and Cloud Hypervisor**. This backend uses local Docker Engine/containerd to invoke Kata's shim; Docker's presence does not imply that the workload shares the host kernel. There is no E2B Cloud account, subscription, API key or hosted sandbox dependency.

Implementation and deployment are different states. The adapter and acceptance suite are implemented. This development host has no usable `/dev/kvm`, so **real VM boot, Kata compatibility, VMM resource accounting and Chromium inside Kata remain unvalidated here**. Do not label an existing runc installation as a microVM deployment. The backend fails closed when its requirements or guest checks fail.

## Why this implementation

We retain an upstream virtualization engine and guest supervisor instead of writing a hypervisor, filesystem server or guest process protocol. Kata/Cloud Hypervisor's virtio-fs integration fits a workspace where both the agent's native file tools and guest applications need to see current files. Firecracker itself does not provide this complete workspace-management layer. E2B remains a useful architecture reference; no E2B code or service is required by this implementation.

Primary references: [Kata architecture](https://github.com/kata-containers/kata-containers/blob/main/docs/design/architecture/README.md), [Kata hypervisors](https://github.com/kata-containers/kata-containers/blob/main/docs/hypervisors.md), [Docker containerd shims](https://docs.docker.com/engine/daemon/alternative-runtimes/), [Kata Cloud Hypervisor settings](https://github.com/kata-containers/kata-containers/blob/main/src/runtime/config/configuration-clh.toml.in). Validate the exact installed release: upstream `main` is not a release pin.

## Workspace behavior

- The controller retains account identity, conversations, schedules and provider credentials outside workload VMs. Existing authorization, credential broker, network firewall and private port tunnels remain in force.
- Each execution lease gets a fresh VM. A workspace can have multiple executions, subject to ownership, write-conflict, memory and concurrency policy. There is no shared live VM across customers and no warm VM pool.
- Files remain on the account's fixed-capacity filesystem. With `persistent_environments: true`, regular shell work also gets a private tool home at `ACCOUNT/environments/SHA256(relative_workspace_path)`. Its stable paths preserve user-installed tools, virtualenvs and caches between executions. The environment counts against the account disk quota.
- Fenix supplies `HOME`, user package prefixes, cache locations and `PATH`. Use project virtualenvs for Python and the provided user prefix for npm. System packages belong to the operator-built image. This is not unrestricted host administration or a persistent root login.
- Language servers receive the project and environment read-only, with offline networking. Restricted read-only shell commands do not receive the persistent home. Browser profiles and downloads retain their existing per-profile mounts.
- When an execution ends, its VM is removed; files and the tool home remain. Active terminal/process leases keep running until completion, cancellation, disconnect, revocation or deadline. A new execution cold-starts another VM. RAM snapshots, suspended process restoration and live migration are **not implemented**.
- DOCX/XLSX/PPTX and PDF tools can be included with `Dockerfile.workspace`. These are Linux tools, not native Windows Microsoft Office. Windows desktops and GPUs require another explicitly supported backend.

Persistent does not mean backed up. The account filesystem is still host-local. Back up account volumes and environment directories using the existing stopped-writer procedure. File checkpoints cover the selected project, not the separate tool home. Replicated storage and automatic multi-host failover are not supplied by this adapter.

## Checks before user code

`backend: microvm` selects only `fenix-kata`. The service requires a functioning KVM API and a private bundle manifest with SHA-256 hashes for the shim, wrapper, configuration, VMM, guest kernel/image, virtiofsd and startup gate. Modified assets require stopping workloads and validating an explicit upgrade.

The prepared configuration selects only Cloud Hypervisor, enables guest and VMM seccomp, disables runtime debug consoles and VM caches, rejects hypervisor annotations and additional host bind mounts, and uses uncached virtio-fs with host block-device passthrough disabled. Read-only selected mounts are also remounted read-only in the host staging area. Tenant commands cannot select another image, runtime, host mount or guest kernel.

Before executing a user command, the mounted gate reports its guest boot identity and waits. Fenix checks that it differs from the host and active guests, identifies the actual Cloud Hypervisor process for this container, verifies that its cgroup belongs to the account, and records that cgroup for emergency termination. Fenix then rechecks account authorization and deadline before releasing the gate. This detects deployment mistakes; it is **not cryptographic attestation** or a proof of resistance to every VM escape.

The host runc AppArmor browser profile is not applied as a guest profile. The VM path uses the browser seccomp policy inside the guest and retains Chromium's inner namespace sandbox. Failure to start that sandbox fails the workload; no `--no-sandbox` fallback exists.

## Resources and account budgets

The service reserves runner memory plus `microvm_overhead_mib` before provisioning. The configured overhead must cover Kata's default guest memory. Host and account admission caps apply, in addition to the account systemd cgroup and guest workload limits. Test actual VMM and virtiofsd accounting on the intended host.

An account may set `execution_seconds_per_month`. The service reserves each execution's maximum duration atomically before launch, settles actual monotonic elapsed time on cleanup, and returns unused time. Simultaneous leases cannot spend the same quota. An unclean service restart consumes outstanding reservations conservatively because the exact stop time is unknown. Periods use UTC calendar months and executions charge the period in which they were admitted. This is an execution-time quota, not monetary billing or model-token accounting.

`execution_status` reports account usage alongside task-filtered executions. Different account credentials cannot access that usage. Removing the optional monthly limit disables only that quota; memory, process, deadline, disk and concurrency limits still apply.

## Prepare an execution host

Use a dedicated Linux host or a VM provider exposing supported nested virtualization. A machine without `/dev/kvm` cannot run this backend. Install a verified Kata release, including Cloud Hypervisor, its supported guest kernel/image and virtiofsd, at the release's compiled installation prefix (normally `/opt/kata`). Verify upstream release checksums before installation. Fenix's manifest pins local bytes; it does not independently establish their upstream provenance.

The ordinary [isolation installation](execution-isolation.md#installation-and-migration) still supplies account volumes, controllers and the execution service. Stop account work and the execution service before changing the runtime. Use a dedicated Kata installation: preparation replaces its shipped default `configuration.toml` with a hardened Cloud Hypervisor configuration and preserves `configuration.before-fenix.toml`.

```sh
sudo scripts/install-isolation.sh
sudo python3 /opt/fenix/isolation/microvm-preflight.py
sudo python3 /opt/fenix/isolation/prepare-microvm.py --register-runtime
sudo systemctl reload docker
sudo python3 /opt/fenix/isolation/microvm-preflight.py \
  --manifest /etc/fenix/microvm/manifest.json
```

Preparation preserves existing Docker settings and its default runtime, validates the merged daemon JSON, and writes a backup. It does not reload Docker itself. Without `--register-runtime`, it only writes `docker.fragment.json` for separate configuration management. If preparation fails after writing a valid bundle, inspect that fragment and the preserved backups before retrying; an existing manifest is not overwritten automatically.

Build a workspace image from a named immutable tenant image digest, with sufficient free disk space:

```sh
docker build -f packaging/linux/Dockerfile.workspace \
  --build-arg FENIX_TENANT_IMAGE=YOUR_REGISTRY/TENANT_IMAGE@sha256:YOUR_DIGEST \
  -t fenix-workspace:local .
```

For a local-only base, use its verified image tag instead; BuildKit does not accept a bare local image ID as a `FROM` reference. The workspace build includes the [YouTube caption environment](youtube-summary.md).

Create `/etc/fenix/execution.json` from `packaging/linux/execution.microvm.example.json`, replacing account identities, registry owner, image and budgets. The example requires considerably more RAM and disk than a minimal local setup; choose real capacity. Keep manifests/control files outside all user roots. Preserve existing account UUIDs and volumes; do not reformat populated storage. Recreate controllers with the matching configuration, then run the acceptance suite before enabling customer traffic.

## Verification

The [recorded validation](../research/self-hosted-microvms-validation-2026-09-20.json) separates policy tests, real OCI regressions and unexecuted VM acceptance tests. The combined OCI run had one Chromium startup failure; its isolated retry passed without weakening the browser sandbox. The full workspace image was not built on this space-constrained host; package dependency resolution passed in a disposable container. These results do not establish VM compatibility or browser stability under load.

Policy and protocol tests require no virtualization:

```sh
python3 -m unittest discover -s tests/isolation -v
```

On the intended disposable KVM host, the following explicitly opts into real VMs, synthetic account mounts, temporary networks and cgroups. It requires a built workspace image and a configured runtime; missing KVM is a failure when opted in.

```sh
sudo env FENIX_MICROVM_TESTS=1 \
  FENIX_MICROVM_MANIFEST=/etc/fenix/microvm/manifest.json \
  FENIX_TEST_IMAGE=fenix-workspace:local \
  python3 -m unittest discover -s tests/isolation -p test_microvm_acceptance.py -v
```

Coverage includes guest identity, project/tool persistence after shutdown, document creation, read-only exports, PTY input/resize, private loopback tunnels, account revocation, descendant cleanup, emergency cgroup termination and Chromium/CDP. Passing unit tests alone does not establish those hardware results. Record the installed asset hashes and acceptance output for each deployment and after runtime/kernel upgrades.
