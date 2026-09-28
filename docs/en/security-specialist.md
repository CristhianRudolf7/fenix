# Security tasks in the shared skill catalog

All ordinary tasks and subagents can discover the complete enabled skill catalog,
including security. There is no dedicated security worker type. Delegate an audit
with `subagent_spawn`, a concrete objective, expected output and scope constraints;
load only relevant procedures through skill search and skill loading.

Categories organize browsing and retrieval, never grant permissions. Disabled
skills stay unavailable. Project ownership, tenant isolation, approval requirements,
read-only plan reviewers and the single delegation level remain enforced.

Local SARIF/Semgrep reports can be inspected with `security_scan_read` by any task
within its authorized filesystem. Results remain candidates with explicit coverage
gaps, not proof of vulnerabilities or a clean assessment. Subagents can optionally
return a structured `security_report`; supplied reports are validated for scope,
coverage and evidence. Ordinary completion and requirement checks still apply.

Legacy persisted security profile labels remain readable for history compatibility;
they no longer restrict skill discovery or tool access. New runtime assignments use
the ordinary profile, including reviewers selected for security expertise. The
independent marketplace publication scanner is unchanged.
