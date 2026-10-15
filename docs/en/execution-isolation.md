> Historical document: the multi-account gateway and Google account registration/sign-in have been removed. See [personal access](remote-access.md) for the current development version.

# Execution isolation and account security

Implementation: 2026-09-19. [Español](../es/execution-isolation.md) · [Original assessment](sandbox-security-review.md) · [Acceptance plan](user-isolation-plan.md).

The default backend is now [self-hosted gVisor with systrap](self-hosted-gvisor.md), with persistent tool homes and monthly execution budgets, without KVM or nested virtualization. The OCI baseline and optional Kata adapter require explicit selection. Runtime identity is verified before user code runs; changing a label alone is insufficient.

This is implemented code for the single-host Linux deployment. It is not an assertion that existing accounts have already been migrated. The root execution service, account volumes, updated controller image and startup units must be installed together. Old containers do not acquire these protections from a repository update.

## Boundary and supported behavior

The account controller retains conversations, provider credentials, schedules and configuration. Shell commands, interactive terminals, language servers and browser processes run in separate gVisor sandboxes by default (or the explicitly configured alternative backend). Skill helper scripts use the same command path; only skills explicitly loaded by the task are mounted, read-only. The controller never gives a workload the Docker socket, the execution socket, its environment, account database or other project mounts.

`fenix-execution` is the Rust client. `scripts/fenix_execution/` is the host policy, lifecycle and broker implementation. The controller launches a fixed Rust stdio relay; that relay executes no workload code locally. In container/multi-user mode, a missing service, identity, authorization revision, policy or supported runtime rejects execution. Standalone local mode remains available and is not a multi-user sandbox.

Every request derives the account from a host-validated credential and authorization revision. Execution IDs are owner-bound. Attachment capabilities are random, single-use and expire if not attached within 30 seconds. Status, cancellation, terminal resizing and private service connections check current ownership. Registry corruption fails closed. Request bodies, raw command arguments, cookies and credentials are excluded from the host audit log.

Selected paths are opened relative to directory descriptors without following links. The service bind-mounts those pinned inodes into a private, root-owned staging directory before Docker receives a path. This avoids both a path-swap race and runc's rejection of direct `/proc/PID/fd/N` bind sources. Staging must remain in the host mount namespace visible to dockerd; do not add mount-namespace systemd hardening directives to this Docker-control service. Its root/Docker authority is part of the trusted boundary.

Native file reads, writes, patches, document imports, visual imports and browser transfers also use descriptor-based regular-file access. Ancestor symlinks and hard-linked files are rejected. File replacement uses a same-directory temporary file, content precondition and atomic rename; a multi-file patch is not a filesystem transaction.

Different tasks cannot simultaneously hold overlapping writable runner mounts. Create an explicit Git worktree when parallel tasks need independent changes. This is an exclusive workspace lease, not automatic merging. Language servers receive a read-only project and offline network; their caches belong in `/tmp`. Browser runners receive only the selected profile and its downloads. Uploads are copied from an authorized project file into that profile.

## Resource and network policy

Each account has a distinct non-root UID/GID and a dedicated fixed-capacity filesystem. `tenant-storage.py` reserves a non-sparse ext4 image before use; an ordinary bind-mounted directory is rejected as a disk quota. Account controllers and their runners share a systemd account slice. Individual containers additionally limit memory/swap, CPUs, PIDs, descriptors, temporary storage and logs. Admission limits concurrent runners and their aggregate reserved memory. Configured controller reservations, runner reservation and host reserve must fit physical RAM. Include the gateway, execution service and other applications in the host reserve.

| Profile | Files | Network |
|---|---|---|
| Offline inspection | Explicit read-only inputs; immutable image dependencies | None |
| Language server | One read-only project | None |
| Development | One project plus selected read-only skill resources | Public TCP 80/443 and fixed DNS resolvers |
| Browser | One profile and its downloads | Public TCP 80/443 and fixed DNS resolvers |
| Account controller | Its account state; protected execution socket | Public egress; private/metadata/host destinations denied |

The development profile permits the public web on those ports; it is not a package-domain allowlist. It does not support arbitrary SSH, UDP or QUIC. Other requirements need an explicit platform policy change.

IPv6 is disabled in container network namespaces. Bridge rules reject private, loopback, link-local, metadata, special-use and detected host public IPv4 addresses. Configure `host_public_ips` for additional public/NAT addresses not assigned directly to a host interface. DNS resolver exceptions are limited to port 53. Rules are installed transactionally, verified in full and placed ahead of unrelated accept rules. No global firewall flush is used. Empty/stale owned chains are repaired. Cleanup refuses to remove a network guard while containers remain attached.

Startup verifies policy before execution. The service monitors rules and current registry state, stopping affected workloads on failure. This polling monitor is not a guarantee of zero exposure between an operator's firewall change and detection. Keep host firewall administration controlled and coordinate Docker/firewall maintenance with stopped account workloads.

There are no published task or CDP ports. Authenticated connections are relayed to loopback inside the exact owned container. The existing Design HTML preview remains script-isolated and blocks external network. Public application previews, shared conversations and bearer preview links are not introduced by this change.

## Browser sandbox

The browser keeps Chromium's inner sandbox, non-root identity, dropped host capabilities and `no-new-privileges`. The service selects the executable from the approved image, so an old account-configured wrapper cannot silently add `--no-sandbox`.

For the OCI/runc baseline, two browser-only policies enable the user/PID/network namespaces and inner chroot needed by Chromium while retaining syscall and AppArmor filtering:

- `packaging/linux/seccomp-browser.json`: based on the [Moby 26.1.5 default seccomp policy](https://github.com/moby/moby/blob/v26.1.5/profiles/seccomp/default.json), with constrained namespace-clone flags, three explicit unshare combinations and chroot. No `SYS_ADMIN`, privileged mode or unconfined seccomp.
- `packaging/linux/fenix-browser.apparmor`: based on the [Moby AppArmor policy](https://github.com/moby/moby/blob/v26.1.5/profiles/apparmor/template.go), adding user namespaces while retaining mount/proc/sys restrictions. Apache-2.0 attribution and license accompany both files.

The gVisor adapter applies OCI seccomp inside the sandbox and retains its own host filters; it does not apply the runc-specific AppArmor browser profile to Sentry. On the OCI baseline, these policies increase the kernel functionality available specifically to the browser. Keep the kernel, Chromium and base policy patched and repeat the browser test after upgrading Docker. See Chromium's [namespace/AppArmor guidance](https://chromium.googlesource.com/chromium/src/+/main/docs/security/apparmor-userns-restrictions.md). A host that cannot enforce the policies fails browser startup; it does not disable the sandbox automatically.

## Integration credentials

The optional `integration_call` tool lists or invokes host-configured operations. `integrations.example.json` documents the schema. A connection belongs to one account and has an expiry, fixed credential host, exact HTTPS URLs/methods, explicit request fields, fixed resource/account fields and usage budgets. Lifetime call counts survive service restart. Failed or ambiguous calls consume a use and are never automatically retried. Existing command approval policy applies to invocations.

DNS answers must all be public and cannot identify a host interface or a configured `host_public_ips`/NAT address. The broker connects to the validated IP while verifying TLS against the configured hostname. It rejects redirects, caller-supplied URLs/headers, oversized bodies and non-JSON responses. Credentials are injected only at the configured destination. Response headers are not forwarded; literal/common encoded credential reflection is rejected. Reflection filtering is secondary protection: only grant operations on trusted APIs, and use narrowly scoped credentials. An already accepted external write cannot be undone by disconnecting or restoring files.

This is a native bounded JSON/HTTPS broker. It is not a preconfigured Composio installation, a universal MCP adapter or an automatic OAuth refresh service. Operators provision and rotate each user's scoped connection; the agent never supplies an external user/account identifier to select credentials. Removing/disabling a connection is checked before dispatch and before returning a response. An in-flight external operation may already have taken effect.

## Encrypted file checkpoints

`workspace_checkpoint` supports create/list/restore/clone/delete inside the task's fixed workspace. The optional host storage uses the maintained `cryptography` AES-GCM implementation, a 32-byte key outside account storage, and authenticated owner/generation/image/policy metadata. Snapshots contain project files only, not RAM, conversations, provider secrets or browser sessions. Links, special files, excessive depth, excessive entries and oversized archives are rejected. Large dependency trees may need a smaller project or an increased explicit quota.

Checkpoint create/restore/clone requires overlapping writable project runners to be stopped. Read-only language servers and unrelated browser/project sessions can remain active. Listing and deleting encrypted copies do not require stopping runners. The service serializes them against launch admission; native task effects also acquire the existing controller effect lease. This is a file archive, not an atomic block-device snapshot: stop external writers as well when a consistent application-level checkpoint is required. Defaults: 32 MiB per archive, 128 MiB per account, 1 GiB globally, at most 32 snapshots per account and retention from one minute to seven days. Ciphertext is authenticated and archive entries are validated before restoration. Restore/clone creates a new destination and fresh generation, preserves existing files, rechecks current authorization and does not replay external calls. Image/policy changes require an explicit migration; saved authority is never restored.

## Installation and migration

Requirements: Linux, systemd, Docker with cgroup v2/systemd driver, seccomp, ext4 tools, iptables, Python 3.11+ and the distribution's `python3-cryptography`. The host operator, gateway registry and execution service are trusted. AppArmor with user-namespace policy support is additionally required for the explicit runc browser backend. The default execution backend is gVisor systrap. Install a verified complete gVisor release, then follow [runtime preparation](self-hosted-gvisor.md#installation) before activation. The explicit `oci`/`runc` backend remains available for baseline testing; unavailable runtimes fail closed.

1. Build the new Fénix binary and tenant image using the existing build process and `Dockerfile.tenant`. Pin the resulting image ID/digest in the execution configuration. Keep the image free of account files, credentials and mutable warm state.
2. Run `sudo scripts/install-isolation.sh`. It installs files only and refuses to replace a running execution service. It does not migrate accounts, activate profiles, enable units or restart existing services.
3. Prepare and register gVisor using the linked runtime guide. Create root-owned `/etc/fenix/execution.json` (0600) from the example. Set real account UUIDs, unique reserved UID/GIDs, registry path/owner, quotas and host public/NAT IPs. The registry can remain owned by the existing trusted gateway/operator UID using `registry_uid`; that UID must never be a tenant UID. Keep its existing 0600 protection. Preserve the gateway's established access to its registry; do not run an internet-facing gateway as root merely to simplify permissions.
4. If using checkpoints, generate the key with `python3 -c 'import json,secrets; print(json.dumps({"key":secrets.token_hex(32)}))'` into a private root-owned key file. Back it up separately. Create a private integrations file with `{"accounts":{}}`, or omit `integrations_file` to disable the broker. Omit `checkpoint_key` to disable checkpoints. Do not use example credentials.
5. During a maintenance window, stop the old account service, runners and Telegram polling instance. Back up its original state. Make the `users_root` parent root-owned and not writable by tenants. For each existing account run `sudo python3 /opt/fenix/isolation/tenant-storage.py /etc/fenix/execution.json ACCOUNT_UUID migrate --write-unit /etc/systemd/system`. For a genuinely empty account use `create`. Migration preserves the old directory as a host-only `.pre-isolation` backup, reserves disk capacity, copies files without following symlinks and adjusts ownership. It never reformats an existing image or overwrites a backup. Account creation into the old user-owned directory tree should happen before changing that parent; new accounts can be prepared in an operator staging tree and moved by the host operator before volume provisioning.
6. For the explicit runc backend, load its browser policy with `sudo apparmor_parser -r /etc/apparmor.d/fenix-browser-v1`. Enable the generated mount units with systemd. Run `sudo systemctl daemon-reload` and start `fenix-execution.service`. The service's state and mount staging must be on protected host storage outside every account directory.
7. Recreate each stopped controller with `tenant-container.py REGISTRY USERS_ROOT EMAIL --execution-config /etc/fenix/execution.json --image APPROVED_IMAGE`. Preserve the account UUID, volume and database; remove only the old stopped container after reviewing its image/configuration. Start the new **system** unit `fenix-controller@ACCOUNT_UUID.service`. Its preflight validates the registry, volume, identity, current credential/revision and firewall, and it binds controller lifetime to the execution service. Retire the old user unit to avoid two instances or duplicate jobs.
8. Test a synthetic account first: login, project read/write, shell and terminal input, LSP, browser/CDP/download, cancellation, integration grant and checkpoint restore. Then canary real accounts under the usual deployment process. Do not enable public ingress for the execution socket or arbitrary runner ports.

Root administration changes permissions and storage. Keep the original backup until account histories, projects, integrations and the fixed seven-day login behavior are verified. On migration failure, stop the account and inspect the stage/image/backup before retrying; the tool deliberately keeps recoverable data. Rollback must not enable an unrestricted host-shell fallback.

## Revocation, restart and incidents

The local commands `fenix accounts disable|enable|passwd REGISTRY EMAIL` atomically update a locked registry and advance its authorization revision; password input uses stdin. Run account administration as the registry's established owner. Live gateway requests and quiet SSE streams recheck authorization (quiet streams poll each second). Logout ends that web session; account disablement and credential/revision changes also revoke runner leases and stop the old controller. Re-enabling does not resurrect old web sessions. Reprovision a stopped controller with the new revision before starting it again.

The execution service kills the complete container on cancellation, connection/heartbeat loss, deadline, account revocation or policy failure. It records the Docker container ID before attachment. If the Docker API fails, it can kill the recorded systemd cgroup directly and retain metadata until infrastructure cleanup is confirmed. Restart kills recorded abandoned cgroups and removes only containers/networks labelled for its configured instance. A global instance lock prevents competing services from cleaning one another's resources. Cancellation reports pending cleanup instead of falsely claiming deletion during an outage.

Audit files rotate at 4 MiB with four backups; completed execution metadata is retained for at most seven days and 10,000 completed records. Metadata databases have a 64 MiB page limit. Checkpoint data has separate quotas and retention. For an incident: disable the account, confirm its controller/cgroups stopped, retain the bounded host audit, rotate affected credentials, inspect the approved image and recover project files into a new directory. Do not restore old session state or replay external writes.

## Verification and limits

Run `python3 -m unittest discover -s tests/isolation -v` for policy/ownership/broker/checkpoint tests. The Docker tests are skipped unless explicitly opted in. On a disposable Linux host with an already present image:

```sh
sudo env FENIX_DOCKER_TESTS=1 FENIX_TEST_IMAGE=fenix-tenant:local \
  python3 -m unittest discover -s tests/isolation -p test_docker.py -v
sudo env FENIX_DOCKER_TESTS=1 FENIX_TEST_IMAGE=fenix-tenant:local \
  python3 -m unittest discover -s tests/isolation -p test_browser.py -v
sudo env FENIX_DOCKER_TESTS=1 \
  python3 -m unittest discover -s tests/isolation -p test_storage.py -v
cargo test --workspace --all-targets --locked -- --test-threads=2
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The tests use synthetic accounts, bounded temporary filesystems and separately labelled containers/networks. They do not deploy the implementation to existing accounts. See [recorded validation](../research/execution-isolation-validation-2026-09-19.json) for actual results and limitations.

The default [gVisor adapter](self-hosted-gvisor.md) uses an application-kernel boundary without KVM. The explicit OCI baseline shares the host kernel directly; the optional Kata backend still requires KVM and has not been hardware-validated here. RAM snapshots, warm pools, escape-resistance certification, reboot drills and comparative p50/p95 benchmarks are not claimed. Run the acceptance suite for the selected backend on the deployment host before customer traffic.
