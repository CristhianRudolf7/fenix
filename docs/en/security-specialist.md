# Security specialist

Fenix delegates security audits through its existing asynchronous worker pool. The master keeps a short delegation instruction; specialized security skill descriptions and bodies stay out of its normal catalog. Workers cannot create other workers. Existing project scope, approvals, cancellation, three-attempt transient-provider retry behavior and report delivery remain in effect.

## Assignment and lifecycle

Use `subagent_spawn` with `profile: "security"`, a precise `objective`, `expected_output`, and nonempty `constraints` describing the target and permitted operations. `general` remains the default for old and new ordinary assignments. The profile is persisted inside the assignment JSON and participates in deduplication. It cannot be changed through a follow-up message. The master can send guidance or cancel using existing tools while continuing the conversation.

A `plan_manage` milestone can also select `profile: "security"`; the plan must provide scope/operation constraints. Revising the profile changes the milestone specification and invalidates obsolete work through the existing version mechanism. Security plan reviewers selected through security skill names retain the stricter read-only review policy and the plan-review report format.

```json
{
  "profile": "security",
  "objective": "Review the login changes for cross-user data exposure",
  "expected_output": "Evidence-backed findings and explicit coverage gaps",
  "constraints": "Inspect the assigned project and synthetic local fixtures; no production requests or code changes"
}
```

The same configured task model runs the specialist. This does not replace the conversation model or change the separate Sentinel publication reviewer configuration.

## Catalog separation

The runtime filters suggestions, lexical search and pagination before returning results. Direct `skill_load` calls, linked resources and execution resource mounts are checked against the task profile. Built-in security identities remain specialized even if an older installed copy lacks category metadata. Custom skills with `metadata.category: security` are included in the security catalog. Disabled skills remain unavailable.

The security worker also gets a small supporting catalog for repository navigation, tests, files and research. Its tool catalog excludes scheduling, memory mutation, connector calls and subagent management; forged direct calls encounter the same profile check. General workers keep ordinary secure implementation guidance. The owner's skills administration interface still displays the entire library and can edit it.

This is a catalog and tool-routing boundary, not a new operating-system sandbox. Existing execution isolation and project authorization still enforce filesystem/process boundaries. Free-text audit constraints are instructions, not a newly enforced per-endpoint network allowlist. A worker with authorized shell access is not made read-only merely by receiving a review objective. Do not expose real production credentials to a test environment. Independent per-audit egress policies would require execution-service support beyond this change.

## Evidence and deterministic processing

A completed security audit must include `subagent_finish.security_report`:

- `scope`: assessed target/revision/environment.
- `coverage`: areas marked `checked`, `inconclusive`, or `not_tested`, with evidence or a reason.
- `findings`: unique IDs, title, `confirmed`/`suspected`/`dismissed`, severity, location, preconditions, evidence and remediation.

The Rust validator rejects missing coverage/evidence and malformed classifications. Empty findings are allowed; they are not a security certification. Validation checks structure, not whether an LLM's evidence is true. Failed/blocked jobs can explain their blocker without manufacturing an assessment. Detailed redacted artifacts stay in the task workspace. Standalone audit handoffs contain a bounded summary, finding/coverage counts and up to eight finding previews; the full result remains recoverable with subagent_status and the worker ID. This bounds conversational context without deleting evidence.

`security_scan_read` reads local SARIF 2.1.0 or Semgrep JSON through the existing file boundary. It returns up to ten candidates per page, deterministic source IDs, producer-reported invocation status and coverage gaps. It never executes code, follows report URLs, uploads source, installs tools or declares a candidate confirmed. Missing/malformed output is an error. Failed or unknown invocations remain visible even when other runs succeed. Input currently uses the existing 1 MiB UTF-8 file limit; larger reports need producer-side partitioning. Process exit evidence and real target coverage must still be checked separately.

## Original skill additions

| Skill | Purpose |
| --- | --- |
| `security-change-review` | Patch scope, reachable callers and altered trust boundaries |
| `security-finding-validation` | Refutation, exploitability, variants and deduplication |
| `security-static-analysis` | Local analyzer selection and honest coverage interpretation |
| `security-rust-boundaries` | Unsafe/FFI invariants, logical races, cancellation and failure paths |
| `security-property-testing` | Invariants, synthetic identities and state-transition tests |
| `security-fix-validation` | Original failure, neighboring variants and legitimate behavior |

Each has one focused reference loaded only when needed. Existing code review, dependency/secret auditing, agent evaluation and skill-source review were strengthened. Known untouched bundled versions upgrade at startup; customized copies are preserved. No upstream framework, commercial scanner service or nested orchestration system is installed.

## Reference review and design choices

Reviewed the [Trail of Bits catalog](https://github.com/trailofbits/skills), including differential review, finding verification, Rust review, static analysis, property testing, variant analysis and post-patch validation, plus the scan runner, SARIF merger and Rust planning scripts (main branch, 2026-09-20). These are research inputs, not vendored implementations. [Read-source hashes](../skills/security-specialist-review.json) identify the downloaded script/procedure versions.

- [Differential review](https://github.com/trailofbits/skills/tree/main/plugins/differential-review) motivated examining callers beyond changed lines. Fenix uses its existing project scope and task journal.
- [Finding verification](https://github.com/trailofbits/skills/tree/main/plugins/fp-check) separates an initial suspicion from a supported claim. Fenix retains inconclusive outcomes and requests independent review through the master when justified, without a mandatory multi-agent gate for every candidate.
- [Rust review](https://github.com/trailofbits/skills/tree/main/plugins/rust-review) covers memory and logical/concurrent failure modes. Fenix uses one focused procedure and optional reference instead of adopting the repository's worker/judge tree or model-specific orchestration.
- [Static analysis](https://github.com/trailofbits/skills/tree/main/plugins/static-analysis) and its scan/merge helpers demonstrate why invocation failures and incomplete rule execution must survive aggregation. Fenix implements a bounded Rust artifact reader; it does not clone rulesets or require Pro services.
- [Property testing](https://github.com/trailofbits/skills/tree/main/plugins/property-based-testing), [variant analysis](https://github.com/trailofbits/skills/tree/main/plugins/variant-analysis) and [post-patch validation](https://github.com/trailofbits/skills/tree/main/plugins/post-patch-validation) informed invariant-based investigation and repair validation. Our procedures emphasize the assigned behavior and existing test framework.
- [PentAGI](https://github.com/vxcontrol/pentagi) provides an example of isolated testing with persistent actions. Fenix reuses its own execution and worker infrastructure.
- [Shannon](https://github.com/KeygraphHQ/shannon) emphasizes source analysis plus reproducible validation. Fenix keeps suspected findings and incomplete coverage visible instead of treating absent reproduction as proof of safety.

No general detection-quality or token-savings claim follows from this implementation. Behavioral tests verify routing, persistence, direct-call rejection, structured completion and scanner error handling. Real detection quality requires labeled code/app fixtures and repeated model runs.

## Validation

Validated the full agent library, existing planning/subagent integration suites, storage suites, ten affected skill packages and resource links. Dedicated regressions cover catalog filtering, direct resource loads, local scanner boundaries, partial scans, compact handoffs, legacy profile defaults and security plan reviewers. Integration model responses are deterministic mocks; these tests exercise runtime behavior, not model vulnerability-detection accuracy. Server test targets compile with the new schemas. No production service was restarted.
