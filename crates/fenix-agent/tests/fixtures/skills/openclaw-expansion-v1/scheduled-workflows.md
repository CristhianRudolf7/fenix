---
name: scheduled-workflows
description: Configure recurring tasks with timezone, overlap, missed-run and duplicate-delivery behavior.
metadata:
  version: 1.0.0
  category: operations
  tags: scheduled, workflows
  aliases: cron, tareas programadas, horario, recurrencia, automatización
  required_tools: schedule_manage,task_inspect
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

## Verification

Check timezone, next occurrence, enabled state, target channel and duplicate behavior.

## Limits

Do not enable autonomous heartbeat or reminders merely because a routine was described; use the actual requested schedule.

## Español

Comprueba zona, próxima ejecución y solapamientos; actualiza sin duplicar ni perder resultados pendientes.
