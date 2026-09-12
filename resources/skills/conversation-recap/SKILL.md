---
name: conversation-recap
description: Recover decisions, progress and pending work from the continuous conversation with references.
version: 1.0.0
---
# Conversation recap

Use when the owner asks what was agreed, completed or remains pending. Inputs: topic and optional time/task references.

1. Search `history_search` and `memory_search` with precise terms. Read nearby history using `history_read`.
2. Inspect relevant tasks with `task_inspect`. Distinguish completed, running, blocked, failed, cancelled and interrupted states.
3. Summarize Objective; Constraints and preferences; Progress (Completed, In progress, Blocked); Key decisions; Relevant files; Next steps; Critical context. Label assumptions and conflicting evidence.
4. Cite message, memory and task IDs. Say when evidence is incomplete. Do not invent details hidden by compaction.

Success: a source-backed recap without changing history, memories or task state. No automatic resumption.

## Español

Recupera historial, memoria y estado de tareas; resume objetivo, restricciones, progreso, decisiones, archivos, próximos pasos y contexto crítico. Incluye referencias e identifica supuestos o conflictos. No transformes tareas interrumpidas en tareas completadas ni las reanudes automáticamente.
