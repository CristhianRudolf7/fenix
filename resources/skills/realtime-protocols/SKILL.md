---
name: realtime-protocols
description: Implement WebSocket, SSE or streamed events with ordering, reconnect and cancellation semantics.
metadata:
  version: 1.0.0
  category: engineering
  tags: realtime, protocols
  aliases: websocket, SSE, streaming, tiempo real, pub sub
  required_tools: file_read,file_write,command_run
  requires: Chosen transport and server/client runtime
  effects: mixed
---
# Realtime Protocols

## Scope

Implement WebSocket, SSE or streamed events with ordering, reconnect and cancellation semantics.

## Inputs

Event schema, user/project scope, delivery needs and disconnect behavior. Prerequisite: Chosen transport and server/client runtime.

## Workflow

1. Select SSE for server-to-client streams or WebSocket for bidirectional messages when suitable. Define event IDs, schema/version and ownership.
2. Handle partial chunks, framing, backpressure and cancellation. Do not treat a received fragment as a complete JSON message.
3. Authenticate subscriptions and scope channels server-side. Separate connection lifetime from background task lifetime when work must continue after disconnect.
4. Reconnect with a cursor or explicit snapshot protocol where supported; deduplicate replayed events and disclose gaps instead of inventing missing progress.

## Verification

Test fragmentation, reconnect, duplicate/out-of-order events, slow clients and two-user isolation.

## Limits

A live connection is not proof of task progress. Do not narrate hidden reasoning as observable execution state.
