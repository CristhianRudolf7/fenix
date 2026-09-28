---
name: analytics-instrumentation
description: Define and verify product events, funnels and retention metrics with consistent identities and denominators.
metadata:
  version: 1.0.0
  category: data
  tags: analytics, instrumentation
  aliases: analítica, tracking, eventos, conversión, retención, cohortes
  required_tools: file_read,file_write,command_run
  requires: Application event path and authorized analytics access
  effects: local-write
---
# Analytics Instrumentation

## Scope

Define and verify product events, funnels and retention metrics with consistent identities and denominators.

## Inputs

Business question, user journey, event definitions and privacy constraints. Prerequisite: Application event path and authorized analytics access.

## Workflow

1. Define events from the decision they support, including trigger, actor, timestamp, properties and deduplication key.
2. Separate anonymous/user/account identities and specify how merges affect cohorts. Preserve event time versus ingestion time.
3. Implement capture on the correct success boundary rather than every button click; inspect retries and client/server double reporting.
4. Validate funnels and retention with explicit denominator, timezone, eligibility and cohort windows.

## Verification

Trace known test journeys into raw events and derived metrics, including duplicate and failed actions.

## Limits

Avoid personal payloads in event properties and do not interpret instrumentation gaps as zero usage.
