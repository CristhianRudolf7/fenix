---
name: code-review
description: Review a concrete code diff against requirements and repository conventions, reporting actionable defects
  with evidence.
metadata:
  version: 1.0.0
  category: engineering
  tags: code, review
  aliases: revisión de código, revisar PR, revisar cambios, code review
  required_tools: file_read,files_search,command_run
  requires: Repository and explicit change scope
  effects: read-only
---
# Code Review

## Scope

Use for review, not implementation. Use security-code-review when a focused security assessment is requested.

## Inputs

Base/head or working-tree scope, requirements, changed files and relevant validation results.

## Workflow

1. Resolve the intended base and head and include uncommitted/untracked changes when in scope. Record the comparison used; an empty committed diff may omit current work.
2. Read changed code together with callers and tests. Evaluate user-visible correctness separately from conventions and maintainability.
3. Focus on introduced defects with a concrete trigger and effect. Trace data/control flow to establish impact; avoid speculative findings without a reachable path.
4. Run a targeted reproduction when useful. Report severity, file location, triggering conditions and a practical correction; separate suggestions from defects.
5. If another reviewer is used by the coordinator, provide only the requirements, exact diff and relevant evidence. Resolve disagreement with code/tests, not a vote.

## Verification

Recheck each finding against current files and distinguish existing defects from new regressions. State review coverage and unexecuted checks.

## Limits

Do not edit files during a review-only request or claim absence of all bugs. Simple changes do not require mandatory additional agents.

## Español

Revisa un diff concreto y sus llamadas. Comunica defectos demostrables con ubicación e impacto; separa mejoras de estilo y resuelve desacuerdos mediante evidencia.
