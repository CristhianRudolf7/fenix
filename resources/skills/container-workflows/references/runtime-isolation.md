# Runtime isolation and workload compatibility

For hardening, inventory the image digest, configured runtime and effective process user/capabilities, privilege escalation, namespaces, devices, seccomp/LSM controls, mounts and network destinations. Inspect the created workload and actual runtime process; a label or configuration field alone does not prove the expected sandbox ran. Distinguish image vulnerabilities, risky configuration, attempted escape telemetry and a demonstrated escape.

Avoid privileged mode, host namespace sharing and runtime sockets in tenant workloads. Check read-only parent mounts, writable submounts, symlink paths and ownership; narrow mounts can still expose another tenant's data. Keep control-plane credentials and cloud metadata access outside the workload. Constrain egress and verify the enforcement layer covers DNS, IPv4/IPv6, redirects and private/control-plane destinations as required by the environment.

Use CPU, memory, process, storage and output limits appropriate to the workload. Test cleanup of descendants, timed-out commands and disk exhaustion in a disposable environment. Sandbox failure must not silently select a less isolated runtime. Preserve needed compiler, shell, browser and document-processing capabilities: indiscriminately deleting shells/package managers is incompatible with an assistant workspace and is not a substitute for isolation.

For a shared service, test two synthetic tenants with private file canaries, concurrent processes and identity revocation. Check intended persistence after restart, no cross-tenant reads and expected denial of host resources using harmless probes. Verify both denial and permitted operations. A scanner finding zero package vulnerabilities says nothing about these boundaries. State unsupported workloads and tests not run; do not promise escape-proof execution.

## Español

Verifica el runtime efectivo, permisos, montajes, red y límites. Distingue configuración riesgosa de escape demostrado. Prueba dos usuarios con canarios, persistencia, revocación y limpieza; conserva las herramientas que necesita el asistente. Nunca presentes una configuración declarada o un escáner de paquetes como prueba de aislamiento.
