---
name: conversation-recap
description: Recover decisions, progress and pending work from the continuous conversation with references.
metadata:
  version: 1.1.0
  category: knowledge
  tags: conversation, recap
  aliases: historial, conversación, resumen, qué acordamos
  required_tools: history_search,history_read,memory_search,task_inspect
  requires: Available task-authorized Fénix tools
  effects: read-only
---
# Conversation recap

## Scope

Recover decisions, progress and pending work from the continuous conversation with references.

## Inputs

Conversation topic and optional time/task references.

## Workflow

Use when the owner asks what was agreed, completed or remains pending. Inputs: topic and optional time/task references.

1. Search `history_search` and `memory_search` with precise terms. Read nearby history using `history_read`.
2. Inspect relevant tasks with `task_inspect`. Distinguish completed, running, blocked, failed, cancelled and interrupted states.
3. Summarize Objective; Constraints and preferences; Progress (Completed, In progress, Blocked); Key decisions; Relevant files; Next steps; Critical context. Label assumptions and conflicting evidence.
4. Cite message, memory and task IDs. Say when evidence is incomplete. Do not invent details hidden by compaction.

Success: a source-backed recap without changing history, memories or task state. No automatic resumption.

## Verification

Tie decisions and task states to message, memory or task evidence.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.
