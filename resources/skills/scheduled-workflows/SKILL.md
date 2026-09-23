---
name: scheduled-workflows
description: Configure recurring tasks with timezone, overlap, missed-run and duplicate-delivery behavior.
metadata:
  version: 1.3.0
  category: operations
  tags: scheduled, workflows
  aliases: cron, tareas programadas, horario, recurrencia, automatización, tarea recurrente, recurring task, solapamiento, missed runs, bounded automation, detener bucle, presupuesto de reintentos
  required_tools: schedule_manage,workflow_manage,connector_call,task_inspect
  requires: Fénix schedule tools or the existing scheduler
  effects: mixed
---
# Scheduled Workflows

## Scope

Configure recurring tasks with timezone, overlap, missed-run and duplicate-delivery behavior.

## Inputs

Task, timezone, recurrence, destination and expected behavior after missed runs. Prerequisite: Fénix schedule tools or the existing scheduler.

## Workflow

1. Resolve the requested time in the configured zone and distinguish calendar schedules from elapsed intervals.
2. Check existing schedules for the same intent. Define whether overlapping runs queue, skip or replace work according to the user’s need.
3. Keep each run identifiable and effects idempotent; preserve undelivered results rather than advancing a delivery cursor prematurely.
4. Inspect the created schedule and next run. For changes, update the existing schedule rather than silently creating another.
5. Choose a simple scheduled prompt for open-ended work. For a repeatable procedure, use `workflow_manage` to save a versioned flow and `schedule_manage` with `mode=workflow` and its `workflow_id`. Discover exact schemas first. Never invent a connection ID, destination, file-selection criterion, browser selector or schedule.
6. Use `connector_call` with `action=list`, then `action=describe` for the selected operation. Credentials and account authorization belong to **Integrations**, never the chat, skill or flow definition. Google Drive uses `drive_search`, then `drive_download` for files or `drive_export` for native Google documents. Use a new workspace-relative destination and preserve returned path/hash evidence. In workflow connector steps, omit destination for an automatic per-run path, or use `automations/{{run_id}}/report.pdf`; only run_id and workflow_id placeholders are interpolated.
7. A flow is a DAG with `nodes`: each has `id`, `label`, `kind`, `depends_on`, and `arguments`. Kinds: `tool`, `connector`, `browser`, `script`, `prompt`, `condition`, `wait`, `notify`. Tool steps also specify `tool`; browser steps have ordered `steps` with `id`, `tool`, and `arguments`. Reuse existing DOM tools and verify actual page state. A write or upload alone is not a confirmed submission; require a receipt or equivalent final check.
8. Bind prior outputs using `{"$ref":"search/files/0/id"}`. References preserve JSON types and must target dependencies. A condition uses `reference`, `operator` (`equals`, `not_equals`, `contains`, `exists`, `count_equals`) and `value`. Its output is `{"matches":true/false}`. Branches use `when` with the same predicate shape. Ask via a `wait` step (`question`) if there is no unique match; do not silently choose another file or course.
9. `workflow_manage action=test` executes known read-only steps and pauses before effects. It does not validate later writes or browser behavior. `run` starts the saved version asynchronously; inspect with `runs`/`inspect`. Recognized read-only steps can use up to three attempts in total; writes are not automatically repeated. Reuse saved execution IDs when continuing. A failed/interrupted effect requires a textual owner decision after inspecting the destination. Never describe retries as exactly-once delivery.
10. Updates require `expected_version`. Existing runs keep their pinned definition; newly scheduled runs use the latest saved version. A flow created or inspected through chat opens in the workspace panel. Use the original project scope. Subagents may inspect flows, but the master creates and starts them.

If a run contains repeated attempts, monitoring with an exit condition or automated repairs, load [bounded execution](references/bounded-execution.md). Define progress, terminal outcomes and enforced budgets in the existing runtime; reconcile uncertain effects before retrying and preserve the acceptance criteria.

## Verification

Check timezone, next occurrence, enabled state, target channel and duplicate behavior.

## Limits

Do not enable autonomous heartbeat or reminders merely because a routine was described; use the actual requested schedule.

## Español

Comprueba zona, próxima ejecución y solapamientos; actualiza sin duplicar ni perder resultados pendientes.

Para procedimientos repetibles, guarda un flujo con `workflow_manage` y prográmalo mediante `schedule_manage`, `mode=workflow` y `workflow_id`. El usuario ve el diagrama y lo modifica conversando. Descubre las conexiones y los argumentos reales antes de usarlos; las credenciales se introducen en Integraciones. No inventes carpetas, cuentas ni criterios para elegir documentos. Conserva la versión y los resultados de cada paso, comprueba el comprobante de entrega y no repitas una acción externa de resultado incierto. Las preguntas y aprobaciones se responden por texto.

Si la tarea programada contiene un bucle, define progreso, límites y terminación en el runtime. Conserva criterios de aceptación y reconcilia efectos inciertos antes de reintentar.
