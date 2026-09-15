---
name: local-troubleshooting
description: Diagnose local failures using files, managed commands and task traces; verify authorized fixes.
version: 1.1.0
---
# Local troubleshooting

Use for a reported local failure. Inputs: symptom, expected behavior, relevant task/path and error if known.

1. Recover evidence with `task_inspect`, `history_search`, `file_read` or `files_search`.
2. Form a small explicit hypothesis; use bounded `command_run` diagnostics with an explicit working directory. Poll via `command_status`; cancel unnecessary managed processes with `command_cancel`.
3. Ask through `ask_user` if critical context is missing. Obtain any required concrete-action approval; do not repeat the same failed action indefinitely.
4. Apply a focused authorized fix, then rerun the relevant check. Report evidence, outcome and unresolved blockers.

Success: the failure is understood and any claimed fix is verified. Use `web_search` or `web_read` when public documentation is needed, respecting their coverage and fetch limits. Do not expose credentials, bypass the web tools through shell fetching, install unrelated software or claim to reverse side effects.

## Español

Diagnostica con evidencia y una hipótesis explícita. Usa comandos acotados, evita bucles y verifica las correcciones autorizadas. Consulta documentación pública con `web_search` o `web_read`, respetando su cobertura y límites; no los evadas con descargas por shell. Explica los bloqueos pendientes, conserva referencias y no expongas credenciales ni prometas revertir acciones.
