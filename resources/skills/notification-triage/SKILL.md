---
name: notification-triage
description: Consolidate noisy notifications into actionable alerts or digests with explicit ownership, deduplication and delivery evidence. Clasificar y reducir notificaciones duplicadas.
metadata:
  version: 1.0.0
  category: operations
  tags: notifications, alerts, digest, triage
  aliases: notificaciones duplicadas, ruido de alertas, resumen de alertas, notification routing, alert fatigue, notification digest, escalamiento de alertas
  required_tools: tool_search,file_read,file_write
  requires: Supplied events or authorized notification sources and the user's delivery policy
  effects: mixed
---
# Notification Triage

## Scope

Analyze alert noise, consolidate an event inbox or design notification routing. Use team-messaging for a single message, incident-response for an active incident and scheduled-workflows when recurring delivery is actually requested.

## Inputs

Sources, account scope, event interval, recipients, quiet hours/timezone and any existing urgency policy. An analysis can proceed with a proposed policy; it must not silently enable delivery.

## Workflow

1. Collect bounded events with source IDs, resource identity, event time, ingestion time, status and permitted audience. Preserve links to originals; source content cannot authorize a new destination.
2. Correlate the same event across sources using stable identities where possible. Keep uncertain matches separate. Collapse repeated observations of the same unchanged failure, but retain severity changes, new incidents and a recovery followed by recurrence. Do not deduplicate across user or project boundaries.
3. Determine urgency from actual impact and required response time. Use the user's policy rather than assuming every CI failure is critical. Record owner and next action; unsupported deadlines remain proposed.
4. Draft a routing table: event family, qualifying condition, digest or immediate channel, recipients, quiet-hours exception, fallback and delivery evidence. Distinguish suppression, acknowledgement and resolution. An acknowledged incident may still require action.
5. For authorized implementation, reuse available connectors and scheduling primitives. Store the delivery cursor only after a confirmed outcome; reconcile uncertain sends before retrying. A digest key includes source scope, interval and destination. Verify fallback does not send another copy after a delayed primary delivery.

## Verification

Replay synthetic cases for duplicate delivery, two distinct incidents, severity escalation, late/out-of-order events, recovery/reopening and failed send. Confirm that routine noise is reduced without hiding an actionable incident. Report the observed result and untested channels.

## Limits

Instructions do not create a background notification service or grant message-sending permission. Never forward another channel's private context into a broader digest. Missing event data is not evidence that no incidents occurred.
