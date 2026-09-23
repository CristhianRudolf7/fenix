# Cluster security review

Separate workload namespace, service account and caller identity. Trace effective Role/ClusterRole bindings, aggregated roles and inherited groups. Review `bind`, `escalate`, `impersonate`, service-account token creation, secrets read/list/watch, pod creation/exec and admission/CSR changes as possible escalation edges; prove the required combination and reachability before calling an edge a working escalation.

An impersonated access query requires the audit identity's impersonation rights and the subject's actual groups. Missing impersonation permission or incomplete authorizer information is not proof the subject is denied. Prefer a specific resource/verb/namespace decision to interpreting a broad list as exhaustive. An authorization decision also does not prove admission will accept a workload.

Inspect init, regular and ephemeral containers plus pod defaults: user, capabilities, privilege escalation, seccomp, host namespaces, hostPath and service-account token mounting. Verify admission policy in the intended namespace and the effective accepted pod. A namespace label alone is not an observed rejection test.

For network policies, verify CNI enforcement and coverage of ingress, egress, DNS, metadata and expected control-plane access. Absence of policy typically leaves traffic allowed; existence of YAML does not establish enforcement. Check secret encryption and access at the configured platform boundary; base64 in a Secret is not encryption. Audit metadata without printing secret values.

Apply narrowly scoped changes only within the requested operation. Preserve unrelated bindings and test both permitted workloads and denied fixtures, with rollback access that does not depend on the identity being changed. Keep routine deployment diagnosis separate from a full cluster audit.

## Español

Revisa permisos efectivos y cadenas de escalada junto con admisión y red. Distingue fallo del auditor de denegación al sujeto. Incluye contenedores init y efímeros, montaje de tokens y políticas realmente aplicadas. YAML, etiquetas y base64 no prueban aislamiento ni cifrado.
