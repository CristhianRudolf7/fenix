---
name: security-artifact-triage
description: Inspect suspicious attachments and archives without activating payloads; preserve static evidence and parsing limits. Analizar archivos sospechosos y macros sin ejecutarlos.
metadata:
  version: 1.0.0
  category: security
  tags: artifact, suspicious, attachment, triage
  aliases: malicious PDF, VBA, XLM, suspicious archive, malware triage, archivo sospechoso, adjunto sospechoso, macros, indicadores
  required_tools: file_read,file_write,command_run
  requires: Supplied artifacts and available static parsers in a restricted analysis environment
  effects: local-write
---
# Security Artifact Triage

## Scope

Use when deciding what a suspicious supplied artifact contains and whether further analysis is warranted. Use phishing-triage for the surrounding message and reverse-engineering for deeper binary behavior. Ordinary document editing does not trigger this workflow.

## Inputs

Original artifact, provenance, acquisition time, analysis question, confidentiality requirements and available parser/isolation capabilities.

## Workflow

1. Preserve the original and record size, hash, acquisition details and actual file format. Analyze a working copy. File extensions, icons and a valid signature do not by themselves establish safety.
2. Verify the analysis environment: patched parsers, unprivileged account, limited writable directories, resource/time limits and no ambient credentials. Parsing attacker-controlled bytes is itself an exposure; if suitable isolation is unavailable, report the blocker before parsing a known hostile sample.
3. Apply only the relevant [artifact checks](references/artifact-checks.md). Disable preview, macros, embedded JavaScript, external relationship resolution and automatic URL/reputation submissions. Decode data without evaluating recovered code.
4. Preserve parent/child relationships and hashes for extracted artifacts. Bound archive depth, member count, expanded bytes and parser runtime. Reject traversal, absolute paths, symlinks and special files during extraction; do not extract into the evidence directory or existing workspaces.
5. Record object/stream IDs, member paths or offsets supporting suspicious behavior and extracted indicators. Keep candidate indicators separate from confirmed malicious infrastructure; ordinary URLs, encryption, obfuscation and high entropy have benign uses.
6. Deliver evidence, parser coverage/errors, verdict confidence and the next bounded investigation. Keep encrypted, unsupported, truncated or timed-out contents explicitly unexamined. Dynamic execution requires a separate appropriate analysis environment and scope; a clean static result is not permission to open the file normally.

## Verification

Confirm originals are unchanged and derived outputs link to them. Distinguish present structure, inferred behavior and observed execution. A zero-detection result with incomplete parsing is inconclusive; preserve the errors.

## Limits

Do not upload files or private indicators to public analysis services without authorization; even a hash query discloses interest in an artifact. No malware engine, forensic platform or dynamic sandbox is installed by this skill.
