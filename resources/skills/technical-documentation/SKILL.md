---
name: technical-documentation
description: Write developer guides, reference documentation and architecture decision records grounded in actual
  code.
metadata:
  version: 1.1.0
  category: writing
  tags: technical, documentation
  aliases: documentación, readme, guía, ADR, manual técnico
  required_tools: file_read,files_search,file_write,command_run
  requires: Repository or verified technical source
  effects: local-write
---
# Technical Documentation

## Scope

Use for technical docs. A tutorial teaches a first success; a how-to solves a task; reference describes interfaces; an ADR records a decision.

## Inputs

Reader, task, supported version, repository conventions and desired artifact.

## Workflow

1. Read existing documentation and the relevant implementation. Identify setup assumptions, interfaces and current behavior before drafting.
2. Choose the document type and keep its scope focused. Record decision status, alternatives and consequences for an ADR without rewriting historical decisions.
3. Use commands and examples appropriate to the installed version. Label placeholders clearly and avoid secret-bearing sample output.
4. Run small examples where practical, or state that they were not executed. Verify links, paths, flags and error recovery guidance.
5. Keep bilingual project documentation aligned when required. Update navigation links and tell the reader the first concrete action.

For documentation accompanying a change, map modified public commands, configuration, interfaces and removal/deprecation behavior to existing reference, task guides, onboarding examples and architectural explanations. Update the affected surfaces, including diagrams and navigation, without generating four document types for every small change. Check examples against the final version and preserve historical release notes. Missing documentation is a concrete gap only when a reader needs it to use or maintain that behavior.

For a first-use or integration audit, use developer-experience-review; documentation quality alone does not prove the documented workflow works.

## Verification

A reader should be able to perform the stated task without unseen session context. Distinguish implemented, planned and deprecated behavior.

## Limits

Do not infer an API from a function name or present aspirational features as available. Preserve unrelated sections when updating docs.
