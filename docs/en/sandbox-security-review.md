# Sandbox repositories and Fénix security review

Research date: 2026-09-19. Status: **assessment and proposal; security changes are not implemented by this review**. [Español](../es/sandbox-security-review.md) · [Implementation plan](user-isolation-plan.md) · [Evidence manifest](../research/sandbox-security-2026-09-19.json).

Fénix should retain its authenticated gateway and separate account runtimes, strengthen their deployment controls, and move untrusted task execution away from account state and credentials. The seven projects provide useful implementation patterns, but installing one does not establish correct authorization between Fénix users. Skills can describe these rules; enforcement belongs in the server, operating system, execution service, and tests.

## Scope and method

The input is the seven repositories in the `sandbox` area of the local `/home/Rudy/proyectos/repo/repo.sqlite` catalog. The catalog was consulted read-only. This review inspected their pinned README feature inventories, selected architecture/security/lifecycle documentation, selected authorization and SDK source, and relevant official product documentation. GitHub trees were fetched without truncation; downloaded blobs were checked against their Git object hashes and recorded with SHA-256 hashes.

This is a detailed architectural and configuration review, **not a line-by-line audit of every repository**. Downloaded files are not all claimed as fully read. The evidence manifest distinguishes source acquisition from focused inspection. No upstream programs, installers, containers, or cloud environments were executed. No commercial security claims or performance numbers were independently benchmarked. Current Fénix source was inspected and its account tests run; deployed firewall, containers, host kernel, and real user data were not inspected or attacked.

| Repository | Pinned commit | What is being compared |
|---|---|---|
| daytonaio/daytona | `ec4c21b2d597091ac09ecc278f3bcc172575a987` | Public repository status plus separately identified product documentation |
| firecracker-microvm/firecracker | `23b09b943fa3cd4a04004fe55724f0b5c2410f45` | MicroVM monitor, jailer, host requirements, snapshots |
| sandboxie-plus/Sandboxie | `e82c55ba4969af79abddcd31155bb1bfe316bee6` | Windows application isolation and privacy modes |
| opensandbox-group/OpenSandbox | `ff4d1bc269cf6a9be03592649bed19633bf45222` | Sandbox lifecycle/execution platform and security components |
| coder/coder | `a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c` | Workspace platform, agent architecture, access governance |
| e2b-dev/E2B | `ccaf9fc0ffe6ac39c7ec786af7608ab1de19467b` | SDK contract and separately documented managed infrastructure |
| TencentCloud/CubeSandbox | `e4e5dc244daaddc0fc706ff67fa4d4e51948cd86` | MicroVM platform, lifecycle, network enforcement, deployment defaults |

## 1. Daytona

**Functions.** The public README and product documentation describe sandbox lifecycle, executable environments, filesystem operations, Git workflows, terminal/process sessions, snapshots, resource management, SDKs, application previews, and network restrictions. Product documentation also describes secret placeholders substituted by an outbound proxy for approved HTTPS destinations. These provide a useful model for developer-friendly task environments. [Pinned README](https://github.com/daytonaio/daytona/blob/ec4c21b2d597091ac09ecc278f3bcc172575a987/README.md), [secret handling](https://www.daytona.io/docs/en/secrets/).

**Security distinctions.** Outbound policy and public preview access are separate controls. A standard preview token is sandbox-wide, including internal service ports, and is not revocable; signed preview URLs can instead be limited to a port, expire, and be revoked. Proxy environment variables alone are not a network boundary. [Network controls](https://www.daytona.io/docs/en/network-limits/), [preview access](https://www.daytona.io/docs/en/preview/).

**Critical availability limit.** The pinned public README states that core development moved to a private codebase in June 2026 and that this repository will receive no further fixes or releases. Its current tree contains documentation/assets, not the current engine source. The old license link points to a historical tag. Consequently, current product features must not be presented as an inspected, maintained open-source engine that Fénix can simply fork.

**Learn and decide.** Adopt separate permissions for execution, outbound connections, and previews; use short-lived, revocable, port-scoped preview grants. Treat a managed Daytona backend as an optional future adapter subject to operational review. Do not base Fénix's primary security boundary on an unsupported historical fork.

## 2. Firecracker

**Functions.** A small Linux/KVM virtual machine monitor, written in Rust, with configurable vCPU/memory, block and network devices, API-driven lifecycle, snapshots, metrics/logs, and device rate limiting. It provides a separate guest kernel; it is not a user-account system, credential manager, or complete agent service. [README](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/README.md).

**Security.** The jailer drops privileges, isolates filesystem/process context, clears inherited environment and file descriptors, and supports cgroups and resource limits. Production guidance recommends distinct nonprivileged identities for VM processes and protected host paths. Seccomp is per-thread; debug/experimental builds do not necessarily have the release defaults. Patching KVM, kernels and microcode remains the operator's responsibility. [Jailer](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/jailer.md), [host setup](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/prod-host-setup.md), [seccomp](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/seccomp.md).

**Snapshot limits.** Snapshot files are trusted input, not automatically authenticated/encrypted archives. Their protection and ownership are the integrating platform's responsibility. Cloning can duplicate application secrets and userspace random-generator state. VMGenID helps kernel randomness after notification but does not automatically repair every userspace cache. [Snapshot security](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/snapshotting/snapshot-support.md), [clone entropy](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/snapshotting/random-for-clones.md).

**Learn and decide.** Reuse a maintained isolation engine instead of writing a hypervisor. Evaluate a Firecracker-based backend for hostile workloads after verifying KVM availability, browser/build compatibility, operational burden, and end-to-end resource usage. Adopt its privilege separation and lifecycle discipline even before selecting microVMs. Hardware isolation does not fix a gateway that authorizes the wrong owner.

## 3. Sandboxie Plus / Classic

**Functions.** Windows application isolation through kernel/user-mode components, filesystem and registry virtualization, multiple boxes, snapshot/recovery, import/export, process restrictions, WFP firewall, DNS/proxy controls, memory/process limits, IPC/clipboard/device controls, encrypted storage, lifecycle triggers, and management interfaces. It shares the Windows host kernel. It is not a Linux execution backend. [Feature inventory](https://github.com/sandboxie-plus/Sandboxie/blob/e82c55ba4969af79abddcd31155bb1bfe316bee6/README.md).

**Security distinctions.** Preventing persistent writes to the host is different from preventing reads of private files. Privacy mode hides host data unless explicitly allowed; hardened mode limits syscall/device access. Some enhanced modes require a supporter certificate. Folder access must also be owner-scoped: `LockBoxToUser` exists because default sandbox-folder ACLs can permit other authenticated Windows users. [Privacy mode](https://sandboxie-plus.github.io/sandboxie-docs/PlusContent/privacy-mode/), [hardened mode](https://sandboxie-plus.github.io/sandboxie-docs/PlusContent/security-mode/), [folder ownership](https://sandboxie-plus.github.io/sandboxie-docs/Content/LockBoxToUser.html).

**Learn and decide.** Adopt explicit read exposure, not just write protection; separate privacy, execution, and sharing policies; include files, IPC, clipboard, and browser profiles in isolation tests. Do not install Sandboxie into the current Linux design. Its README distinguishes Classic and custom Plus licensing; GitHub's single license label is insufficient to characterize the entire repository.

## 4. OpenSandbox

**Functions.** Lifecycle and execution APIs; Docker and Kubernetes backends; multiple language SDKs and CLI; commands, files, code interpreters, browser/desktop examples; ingress and egress components; optional secure runtimes; credential brokerage; and isolation sessions within a sandbox. Release documentation includes artifact signature/provenance verification. [README](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/README.md).

**Authorization.** The inspected middleware supports a global API key or a `TenantProvider` resolving a key to tenant identity. Missing both can disable that middleware's authentication. A tenant-provider failure is distinguished from an invalid credential. Separate endpoint tokens also exist. This is evidence of an extensible authentication design, not proof that every backend operation checks Fénix's owner correctly. [Middleware](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/server/opensandbox_server/middleware/auth.py), [provider contract](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/server/opensandbox_server/tenants/provider.py).

**Network and credentials.** Platform-owned deny rules can override workload-requested access. Credential Vault keeps real secrets in a separate egress component, matching scheme/host/port/method/path before injection. DNS-only filtering is insufficient for this feature; it requires packet enforcement. Sidecar loss can interrupt traffic, and recreated sidecars require trusted credential re-injection. [Network design](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/docs/architecture/network-isolation.md), [Vault](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/docs/guides/credential-vault.md).

**Compatibility and incomplete features.** Isolation sessions use additional namespaces/overlays, but their documented bootstrap grants `CAP_SYS_ADMIN` and disables AppArmor confinement. That is not a safe automatic replacement for Fénix's current dropped-capability setup. The overlay size threshold is an admission check, not a continuous disk quota; diff/commit endpoints are documented stubs. The documented egress implementation has compatibility limits with gVisor and transparent service meshes. [Sessions](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/docs/guides/isolation-sessions.md).

**Cross-check.** Its networking explanation overstates general limitations of Kubernetes NetworkPolicy. Kubernetes supports default-deny ingress/egress across selected pods, including a namespace-wide empty selector, with a supporting network plugin. Fénix should evaluate actual deployment requirements rather than inherit that blanket claim. [Kubernetes documentation](https://kubernetes.io/docs/concepts/services-networking/network-policies/).

**Learn and decide.** Borrow the execution API separation, operator-enforced network baseline, precise credential bindings, and capability probing. Consider an adapter only with explicit authenticated tenancy and tested runtime compatibility. Do not enable additional host privileges merely to obtain a convenient feature.

## 5. Coder

**Functions.** Terraform-based workspace provisioning across containers/VMs/infrastructure, workspace agents, remote terminal/IDE connectivity, lifecycle/autostop, organizational access controls, models/MCP configuration, and agent orchestration. Its agent design separates the model/orchestration loop in the control plane from code execution inside workspaces; workspace access uses an authenticated tunnel. [README](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/README.md), [agent architecture](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/architecture.md).

**Access lessons.** Organization configuration, resource ACLs, per-user OAuth, tool allow/deny policies, and read-only chat sharing are distinct concepts. Sharing a parent conversation affects child visibility; granting view access must not imply execution rights. Identity headers are metadata, not independently sufficient authentication. [Organization scopes](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/platform-controls/organizations.md), [MCP governance](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/platform-controls/mcp-servers.md), [sharing](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/chat-sharing.md).

**Limits.** Workspace secrets may deliberately be exposed to workspace code. Database encryption is optional, covers specified fields, and does not automatically encrypt old records merely by enabling it. Multiple organizations, template permissions, and audit logs have documented Premium restrictions. The repository includes AGPL and enterprise licensing boundaries; copying implementation requires a separate license review. [Secrets](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/security/secrets.md), [database encryption](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/security/database-encryption.md), [template permissions](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/templates/template-permissions.md), [audit logs](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/security/audit-logs.md).

**Learn and decide.** This is the strongest reference for separating trusted orchestration from developer-controlled execution and for access governance. Preserve Fénix's separate per-account controllers while introducing task runners. Do not migrate to a shared all-user controller or buy an entire workspace platform solely to reproduce those principles.

## 6. E2B

**Functions.** SDK-driven sandbox lifecycle, commands/processes/PTYs, files, templates, timeouts, pause/resume, application URLs, and integrations with code-interpreter and desktop packages. The inspected repository is principally the SDK contract. The service describes Firecracker-based isolation, while infrastructure is maintained separately. [README](https://github.com/e2b-dev/E2B/blob/ccaf9fc0ffe6ac39c7ec786af7608ab1de19467b/README.md), [service security](https://e2b.dev/security).

**Security distinctions from the pinned SDK.** Environment-daemon access is always secured; the old `secure` option is deprecated and ignored. This does not mean application URLs are automatically private: `allowPublicTraffic` is separately documented with a true default. Network selectors, per-domain transforms, workload-identity placeholders, and an egress proxy are represented in the API. Some capabilities are deployment-dependent: the SDK explicitly says the named egress proxy is supported in Cloud/BYOC but rejected by the open-source runtime. [SDK contract](https://github.com/e2b-dev/E2B/blob/ccaf9fc0ffe6ac39c7ec786af7608ab1de19467b/packages/js-sdk/src/sandbox/sandboxApi.ts).

**Limits.** An SDK type is not proof that an independently hosted backend implements it. Product security/enterprise descriptions do not establish security of an arbitrary self-hosted installation. Older documentation describes different defaults, so a Fénix adapter must pin versions and test behavior. Infrastructure self-host instructions could not be retrieved in this review; no full infrastructure audit is claimed.

**Learn and decide.** Adopt explicit deadlines, cancellation, authenticated reconnect, separate application ingress policy, and backend feature negotiation. A managed adapter can be useful later, but must use Fénix-owned mappings from account/task to backend sandbox IDs, never trust a model-supplied ID, and keep service API keys outside runners.

## 7. CubeSandbox

**Functions.** Linux/KVM microVM platform with a Rust monitor, task execution, templates, copy-on-write cloning, snapshots, rollback, pause/resume, persistent volumes, and E2B-compatible SDK surfaces. Networking combines eBPF packet policy with an optional HTTP/HTTPS security proxy. Some distributed deployment paths are documented as preview. Published startup/memory measurements remain author claims here. [README](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/README.md), [lifecycle](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/snapshot-rollback-clone.md).

**Authorization and deployment.** API authentication is optional unless an authorization callback is configured. Hardening guidance identifies unauthenticated internal services, broad listener bindings, and unencrypted internal channels needing deployment protection. Public sandbox URLs are accessible by default; token-required ingress is explicit, and the documented ingress token has no rotation API during a sandbox's lifetime. [Authentication](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/authentication.md), [network hardening](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/network-hardening.md), [public access](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/restrict-public-access.md).

**Network and secrets.** The proxy can constrain host/SNI/method/path, inject credentials outside the guest, and log decisions. Traffic not intercepted remains governed by the packet layer; an L7 rule does not automatically remove broader L3 permission. Packet allow rules take precedence over deny rules, so a platform baseline needs protection from tenant overrides. Snapshot contents persist independently of the running sandbox and need separate deletion/ownership policy. [Network policy](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/network-policy.md), [security proxy](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/security-proxy.md).

**Learn and decide.** Borrow external enforcement, lifecycle generation tracking, and safe clone cleanup. Do not adopt the quickstart as a production security configuration. Do not assume complete E2B parity or that rolling back local state undoes external API actions. The license file declares Apache-2.0 with listed third-party exceptions; a single GitHub metadata label does not capture that structure.

## Comparison relevant to Fénix

| Project | Principal boundary | User authorization | Credential lesson | Recommended use |
|---|---|---|---|---|
| Daytona | Managed sandbox product; current engine not inspected | Product/API plus separate preview grants | Destination-bound substitution | Product patterns; optional managed adapter |
| Firecracker | Guest kernel + KVM + constrained host VMM | Integrator must supply it | Keep secrets out of guest/snapshot | Candidate strong execution backend |
| Sandboxie | Windows kernel-mediated application box | Box ownership and filesystem ACLs matter | Hide private reads explicitly | Privacy principles; possible future Windows research |
| OpenSandbox | Selected container/secure runtime | Configurable auth and tenant provider | Separate vault and packet enforcement | API/broker reference; conditional adapter |
| Coder | Provisioned workspace plus control plane | Rich account/resource governance | Separate controller; injection still exposes workspace secrets | Architecture and authorization reference |
| E2B | Service microVM; SDK is client contract | Service identity plus sandbox/application credentials | Backend-scoped identity and transformations | Optional adapter; lifecycle reference |
| CubeSandbox | KVM VM plus host network enforcement | Deployment callback and separate ingress token | Host-side injection and auditing | Network/lifecycle ideas; further hardening needed |

These are different layers, not seven interchangeable products. Combining all seven would increase configuration and patching obligations without proving better isolation.

## Fénix: verified controls and gaps

The current source already provides account UUIDs, Argon2 password verification, protected registry/session files, fixed one-week sessions, owner-based proxy routing, and an upstream credential per runtime. The proxy selects the runtime from authenticated state and strips arbitrary incoming headers. Each account has its own runtime and state directory. The shared skill marketplace is an intentional shared subsystem with owner-scoped operations and explicitly public approved content. [Gateway](../../apps/server/src/accounts.rs), [sessions](../../apps/server/src/accounts/sessions.rs), [marketplace](../../apps/server/src/skill_guard/mod.rs).

Container provisioning requests a read-only image, dropped capabilities, no-new-privileges, a private PID namespace through Docker defaults, a private tmpfs, loopback publication, a dedicated bridge, and only the account directory bind mount. It does not mount the Docker socket. Runtime container mode requires an upstream token. Restricted read-only execution additionally uses Landlock and seccomp and rejects unsupported enforcement. These are valuable controls to preserve. [Provisioning](../../scripts/tenant-container.py), [runtime entry](../../apps/server/src/main.rs), [read-only helper](../../crates/fenix-agent/src/read_only.rs).

| Finding | Evidence and confidence | Consequence / scope | Priority |
|---|---|---|---|
| Existing firewall chain is trusted without rebuilding its core rules | `tenant-network-guard.py`; inert mocked execution confirmed zero destination-deny repairs for an existing empty chain | Partial/altered host policy may survive startup checks. No deployed exposure demonstrated | P0 |
| Container UID derives from invoking UID | `tenant-container.py`; inert root simulation produced `--user 0:0` | Non-root guarantee depends on invocation; same provisioning UID also means shared host UID across accounts | P0 |
| No tenant CPU/RAM/PID or persistent disk quotas in provisioning | Docker arguments and deployment documentation | One tenant can exhaust shared capacity; 16 managed commands and a 512 MiB tmpfs are not aggregate quotas | P0 |
| Trusted controller and writable task execution share account mount/identity | Runtime, process execution, storage layout | Untrusted code within an account can target its controller state/credentials; this is distinct from demonstrated access to another account | P1 |
| Environment handling differs by subprocess path | Shell removes names containing API_KEY/TOKEN/SECRET; LSP spawn inherits environment | A denylist is incomplete; every execution path needs a common minimal environment | P1 |
| Tenant browser wrapper disables Chromium's inner sandbox | `Dockerfile.tenant` uses `--no-sandbox` | Container boundary carries more of the browser compromise risk | P1 |
| Public Internet is generally allowed | Host guard rejects specified private/link-local ranges | Confidential data readable by a compromised task may leave through allowed public destinations | P1 |
| Registry revocation is reload-dependent | Account/session implementation and docs | Password/account edits are not an immediate live revocation mechanism without reload | P1 |
| Sharing, task artifacts and background jobs need broader adversarial coverage | Current account tests cover selected gateway/marketplace paths | Coverage gap, not a proven cross-user vulnerability | P0/P1 |

The trusted-host administrator remains outside the ordinary user isolation boundary. Neither containers nor microVMs should be advertised as protecting users from an administrator who controls their host and keys. Path canonicalization and per-tool checks remain useful but do not replace process-level mounts and authorization; race-resistant file access is a separate hardening task.

## Validation performed

`cargo test -p fenix-server --lib accounts:: --locked` passed **7 tests**, including forged-header routing, authentication, persistent sessions, durable revocation behavior, and marketplace ownership. This is not a full repository test run or a kernel/container escape test.

Two local inert checks ran the provisioning/guard functions with every external command mocked, using synthetic account data and a temporary directory. They confirmed the root UID propagation and the existing-chain repair gap. No Docker, sudo, firewall command, or account secret was accessed by those checks. Exact observations are preserved in the evidence manifest.

The resulting [implementation plan](user-isolation-plan.md) defines the changes, dependencies, migration, and acceptance tests. No claim that Fénix is fully secure follows from this review or the seven passing tests.
