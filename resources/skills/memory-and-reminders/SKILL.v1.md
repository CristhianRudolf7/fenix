---
name: memory-and-reminders
description: Save, correct and retrieve explicit memories; manage requested reminders and recurring schedules.
version: 1.0.0
---
# Memory and reminders

Use when the owner asks to remember, forget, correct or retrieve a fact, or requests a reminder or schedule. Inputs are the explicit request, relevant memory IDs and, for schedules, a precise date/time or recurrence.

1. Separate a durable fact from a scheduled action. Do not infer consent to remember personal details.
2. Search with `memory_search` before saving; use `memory_manage` to add, update or delete. Keep the source message/task reference. Do not rewrite unrelated facts.
3. For reminders, use `schedule_manage`. Resolve relative dates in America/Lima using the supplied current date. Use `ask_user` for ambiguous times or recurring actions. A reminder delivers text; an agent schedule enqueues a task.
4. Read back the resulting record. Report its ID, content and, for schedules, next local execution and status. Pausing/resuming affects future runs only.

Success: the requested record exists with correct content and provenance, or the specified record is removed. Never invent a saved result. Memory and schedules use different stores. No background learning, automatic preferences, web tools or heartbeat.

## Español

Activa este skill para recuerdos explícitos y recordatorios solicitados. Busca duplicados antes de guardar; conserva la procedencia y usa el ID al corregir o borrar. Distingue un hecho de una programación. Aclara fechas ambiguas, usa America/Lima y verifica el registro final. Un recordatorio entrega texto; una tarea programada entra en la cola. No aprendas preferencias automáticamente ni crees recordatorios sin solicitud.
