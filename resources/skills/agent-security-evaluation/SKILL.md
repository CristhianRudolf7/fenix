---
name: agent-security-evaluation
description: Evaluate agent, RAG and MCP security through controlled prompt-injection, memory, identity and tool-effect tests. Evaluar seguridad del agente con canarios y efectos simulados.
metadata:
  version: 1.1.0
  category: security
  tags: agent, security, evaluation, injection
  aliases: indirect prompt injection, tool poisoning, approval replay, cross tenant memory leakage, inyección de instrucciones, memoria contaminada, seguridad del agente
  required_tools: file_read,file_write,command_run
  requires: Authorized test harness or inspectable traces; isolated identities and recording tools for effect tests
  effects: local-write
---
# Agent Security Evaluation

## Scope

Use for an adversarial evaluation of an agent's actual application paths. Source defects belong to security-code-review; ordinary retrieval relevance belongs to rag-evaluation. A test harness can evaluate controls without installing a red-team framework.

## Inputs

Target build/model/prompt versions, trust boundaries, allowed test surfaces, synthetic accounts/data, permitted effects and request/cost limits. Existing task authorization applies; resolve only missing scope before interacting with a live target.

## Workflow

1. Define the protected outcome and a working legitimate baseline. Map external documents, tool descriptions/results, files, persistent memory and queued actions to the identities and operations they can influence.
2. Build cases from the applicable rows in [security scenarios](references/security-scenarios.md). Use unique synthetic canaries and recording substitutes for email, payments, network exfiltration and destructive tools. Keep fixtures separate from real user memory and credentials.
3. Place an indirect injection in the actual lower-trust entrypoint. Record ingestion, retrieval rank, rendered context and whether the model saw it. Sending the same text as a user message tests a different surface. A probe that was never retrieved is not evidence of resistance after exposure.
4. Observe tool proposals, authorization decisions, attempted calls, completed effects and canary disclosures separately. Preserve request IDs and sanitized traces. Refusal wording, keyword matches, JSON-schema validity or vector similarity alone do not establish a security outcome.
5. Test server-owned identity, resource ownership, nested operations and stale authorization at execution time. Where approvals exist, exercise changed arguments, expired/replayed approvals and revocation while work is queued. The harness must not bypass the control under test with an auto-approval option.
6. Include legitimate tasks using the same documents and tools. Freeze cases before comparing revisions, reset persistent state between independent trials, record repeated trials and keep errors, unexposed cases and missing traces distinct from passes. Report per-surface counts and benign completion alongside confirmed violations.
7. Reproduce confirmed failures with the smallest case, propose or apply an in-scope fix and rerun the failure plus neighboring legitimate cases. Preserve traces of partial effects before cleanup.

## Focused follow-up

Use security-property-testing for sequences involving retrieval, queued actions, revocation and session reconnects. Record the legitimate control case and actual tool effects independently of the model response. After an authorized repair, use security-fix-validation to ensure prompt-injection defenses preserve ordinary task completion.

## Verification

Tie each verdict to the actual identity, exposure and observed effect. Use pass, violation, inconclusive or not-run with a reason; a timeout without effect telemetry is inconclusive. Report harness-only checks separately from real model or deployed-runtime runs.

## Limits

This skill does not enforce isolation, install a proxy or establish immunity to prompt injection. Never place real secrets in evaluation fixtures or send test effects to real recipients. Do not change production memory or connected accounts merely to construct a test.
