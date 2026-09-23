---
name: local-troubleshooting
description: Diagnose local service, command or tool failures using files and managed commands, and verify authorized repairs. Diagnosticar un servidor que no arranca o un comando que falla.
metadata:
  version: 1.4.0
  category: operations
  tags: local, troubleshooting
  aliases: diagnóstico local, fallo local, error herramienta, problema, servidor no arranca, command failure, service failure
  required_tools: task_inspect,history_search,file_read,command_run,command_status
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Local troubleshooting

## Scope

Diagnose local failures using files, managed commands and task traces; verify authorized fixes.

## Inputs

Symptom, expected behavior, path/task and exact error if known.

## Workflow

Use for a reported local failure. Inputs: symptom, expected behavior, relevant task/path and error if known.

1. Recover evidence with `task_inspect`, `history_search`, `file_read` or `files_search`.
2. Form a small explicit hypothesis; use bounded `command_run` diagnostics with an explicit working directory. Poll via `command_status`; cancel unnecessary managed processes with `command_cancel`.
3. Ask through `ask_user` if critical context is missing. Obtain any required concrete-action approval; do not repeat the same failed action indefinitely.
4. Apply a focused authorized fix, then rerun the relevant check. Report evidence, outcome and unresolved blockers.

Success: the failure is understood and any claimed fix is verified. Use `web_search` or `web_read` when public documentation is needed, respecting their coverage and fetch limits. Do not expose credentials, bypass the web tools through shell fetching, install unrelated software or claim to reverse side effects.

When the complaint concerns the agent's decisions or repeated actions rather than a failing local operation, discover and load agent-workflow-diagnosis. That workflow investigates task evidence without executing repairs.

## Verification

Repeat the failing operation after a focused fix and report unresolved causes.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Diagnostica con evidencia y una hipótesis explícita. Usa comandos acotados, evita bucles y verifica las correcciones autorizadas. Consulta documentación pública con `web_search` o `web_read`, respetando su cobertura y límites; no los evadas con descargas por shell. Explica los bloqueos pendientes, conserva referencias y no expongas credenciales ni prometas revertir acciones.

Para analizar decisiones o acciones repetidas del agente, usa agent-workflow-diagnosis. Esa investigación es de solo lectura y debe sustentar cualquier corrección posterior.
