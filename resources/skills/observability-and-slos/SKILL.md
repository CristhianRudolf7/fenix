---
name: observability-and-slos
description: Instrument useful logs, metrics and traces and define service objectives tied to user-visible reliability.
metadata:
  version: 1.0.0
  category: operations
  tags: observability, and, slos
  aliases: observabilidad, métricas, logs, trazas, SLO, alertas
  required_tools: file_read,file_write,command_run
  requires: Existing telemetry stack or local instrumentation surface
  effects: mixed
---
# Observability And Slos

## Scope

Instrument useful logs, metrics and traces and define service objectives tied to user-visible reliability.

## Inputs

Critical flows, available data, operational owner and reliability question. Prerequisite: Existing telemetry stack or local instrumentation surface.

## Workflow

1. Define an observable success/latency indicator and its denominator before choosing dashboards. Separate application, infrastructure and business signals.
2. Correlate requests/jobs using scoped IDs while excluding secrets and personal payloads. Bound metric label cardinality.
3. Instrument failure and cancellation paths as well as success; propagate trace context across queues when supported.
4. Set alerts from actionable user impact and sustained error-budget consumption, with an owner and diagnostic link. Avoid paging on every isolated exception.

## Verification

Trigger a controlled local failure and confirm the expected signal reaches the sink; check missing data behavior.

## Limits

Do not invent an uptime target or install a monitoring service merely because a log was requested.
