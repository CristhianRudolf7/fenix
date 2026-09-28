---
name: reverse-engineering
description: Analyze binaries, undocumented formats and authorized black-box interfaces from evidence. Ingeniería inversa y caja negra.
metadata:
  version: 1.2.0
  category: engineering
  tags: binary, reverse, disassembly, protocol
  aliases: reverse engineering, ELF, PE, Mach-O, decompiler, binary diff, ingeniería inversa, binario, desensamblado, protocolo binario, black-box reconstruction, query-only, model extraction, caja negra
  required_tools: file_read,file_write,command_run,tool_search
  requires: Authorized artifacts and compatible inspection tools; isolated environment for dynamic execution
  effects: local-write
---
# Reverse Engineering

## Scope

Use to understand a compiled program, compare binary versions, reconstruct an undocumented data format or infer a black-box model from permitted queries. For query-only tasks, the information boundary below takes precedence over static source inspection. Source-code review belongs to code-review; incident evidence and attribution belong to incident-response. This workflow does not assume a live target is a lab.

## Inputs

Artifact and origin, specific behavior/question, known versions, symbols, captures and available analysis environment. An offline inspection can proceed while execution prerequisites remain unresolved.

## Workflow

1. Preserve original bytes and compute a hash for each version; analyze a working copy. Identify format, architecture, endianness, loader metadata and available symbols using actual tools. Record parse failures or packing rather than interpreting an unreadable table as absent behavior.
2. Form a bounded hypothesis and inspect relevant entrypoints, imports/exports, strings, cross-references and control/data flow. Distinguish file offsets, relative virtual addresses and loaded addresses. A decompiler's guessed name or type is an annotation, not source truth.
3. For version comparison, match several independent anchors such as call structure, constants and references. Record uncertain matches and conflicting candidates before applying symbol renames to an analysis copy. Address proximity or a plausible model answer alone is insufficient.
4. For protocols, separate transport segments from application messages and reassemble streams first. Compare multiple messages for lengths, endianness, optional fields and state transitions; preserve unknown bytes. Validate a proposed decoder on samples not used to derive it, including truncated input. A TLS capture without keys does not reveal plaintext message fields.
5. Use dynamic analysis only when it answers a remaining question in an authorized, genuinely isolated environment. Record arguments, environment, observations and resulting artifacts. Do not execute unknown samples on the host as an automatic next step or upload binaries to a public analysis service by default.
6. Deliver an evidence map linking claims to offsets, instruction ranges, message bytes or observations, plus a reproducible decoder or inspection command when appropriate. Keep inferred behavior separate from observed execution and state what would disprove the interpretation.

Before mapping binary values, inspect the actual format headers, architecture, endianness and address conventions. Do not infer relocation or signedness from illustrative output. Compare representative decoded values and their addresses using an independent reader; include another small valid input when generality is required. Missing public format metadata calls for authorized investigation before an owner question.

For query-only or black-box reconstruction, the permitted oracle is the information boundary. Do not read hidden weights, random seeds, answer files or implementation internals to choose constants or validate candidates merely because the filesystem exposes them. Infer unknown dimensions/counts from permitted observations, document that evidence, and use independent allowed queries to challenge the hypothesis. Do not label a guessed or source-derived constant as experimentally inferred. A diagnostic comparison against privileged internals is not a black-box validation. Outside such restrictions, authorized static analysis remains valid.

## Verification

Recheck important inferences against a second anchor or independent sample. Preserve originals and record tool/version limitations. Missing imports, strings or suspicious behavior in one run do not prove absence of a capability.

## Limits

No automatic toolchain bootstrap, remote exploitation or bypass is implied. If a needed disassembler or sandbox is unavailable, report the supported static findings and the exact remaining prerequisite.
