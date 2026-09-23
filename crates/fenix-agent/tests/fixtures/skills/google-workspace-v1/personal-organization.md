---
name: personal-organization
description: Organize personal commitments into a realistic plan using existing tasks, dates and preferences.
metadata:
  version: 1.0.0
  category: personal
  tags: personal, organization
  aliases: organizar día, agenda personal, prioridades, pendientes, rutina
  required_tools: memory_search,history_search,schedule_manage
  requires: User commitments and available task/calendar evidence
  effects: local-write
---
# Personal Organization

## Scope

Organize personal commitments into a realistic plan using existing tasks, dates and preferences.

## Inputs

Time horizon, obligations, constraints, energy/preferences and desired outcome. Prerequisite: User commitments and available task/calendar evidence.

## Workflow

1. Collect existing commitments without creating duplicates. Separate hard deadlines from preferences and ideas.
2. Estimate available time and dependencies; preserve breaks, travel and fixed appointments where supplied.
3. Prioritize a feasible next-action set and expose conflicts or overload rather than fitting everything by assumption.
4. Update existing tasks/reminders only when requested and show what remains deferred or unresolved.

## Verification

Check schedule collisions, timezone and whether each action has a concrete next step.

## Limits

Do not cancel meetings, create recurring reminders or infer health/productivity diagnoses from incomplete activity data.

## Español

Ordena compromisos reales sin duplicarlos, detecta sobrecarga y distingue fechas obligatorias de preferencias.
