---
name: automation-audit
description: Inventory configured automations, jobs, hooks and connectors; verify their actual state and identify duplicate effects or missing coverage. Auditar automatizaciones existentes.
metadata:
  version: 1.0.0
  category: operations
  tags: automation, audit, jobs, hooks, connectors
  aliases: auditoría automatizaciones, qué automatizaciones funcionan, tareas duplicadas, automation inventory, audit scheduled jobs, duplicate automations, integración configurada
  required_tools: file_read,files_search,tool_search,task_inspect,schedule_manage,workflow_manage
  requires: Authorized project configuration and available scheduler or execution evidence
  effects: read-only
---
# Automation Audit

## Scope

Establish which existing automations are configured, usable, recently observed, failing or overlapping. Use scheduled-workflows to implement an authorized schedule change and agent-workflow-diagnosis for a particular agent run. An inventory request does not start the jobs it discovers.

## Inputs

User/project scope, relevant systems, observation window and symptoms. Use supplied exports if a live connection is unavailable; identify their date and coverage.

## Workflow

1. Discover existing configuration, scheduled flows, job definitions and enabled integration surfaces within the authorized scope. Describe available tool schemas and use only read/inspect operations. Read hook scripts as source; listing an executable is different from starting it.
2. Build one row per stable automation identity: trigger, owner, version, source, destination, expected effects, credential status if reported, enabled state, last observed run, outcome and evidence reference. Do not expose secrets from configuration.
3. Keep configured, enabled, authenticated, reachable and successfully executed as separate fields. A past success proves that run; absent logs or a disabled connection leave current execution unverified. Authentication status is unknown when no authorized source reports it.
4. Trace duplicate paths by trigger, account, source range and destination effect. Similar names do not prove duplication; deliberate failover or separate tenant scopes may explain overlap. Check for shared cursors, repeated deliveries, missing owners and old workflow versions still running.
5. Recommend a specific repair or consolidation with its evidence, affected consumers and expected result. Preserve disabled jobs, pending deliveries and user customizations. If fixes were requested too, hand the supported change to the matching skill and verify the resulting state; do not execute an audit probe that sends mail or moves money.

## Verification

Every live-state conclusion names its observation time and receipt/config source. Count unavailable systems separately from reviewed systems and distinguish an empty complete listing from a partial or failed one.

## Limits

This skill adds an audit procedure, not connectors, background monitoring or scheduler enforcement. Do not scan other users' home directories, import credentials or enable jobs merely to improve the inventory.

## Español

Inventaría las automatizaciones del ámbito autorizado y separa configuración, activación, autenticación y ejecución comprobada. Detecta duplicados por sus efectos y destinos, no solo por nombres. Conserva referencias, fechas y sistemas no revisados; una auditoría no ejecuta ni borra tareas por sí sola.
