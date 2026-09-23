---
name: team-messaging
description: Search or send team messages with correct channels, threads, recipients and duplicate-send handling.
metadata:
  version: 1.0.0
  category: office
  tags: team, messaging
  aliases: Slack, Teams, Discord, mensaje equipo, canal, hilo
  required_tools: tool_search
  requires: Authorized messaging connection or supplied transcript
  effects: external-write
---
# Team Messaging

## Scope

Search or send team messages with correct channels, threads, recipients and duplicate-send handling.

## Inputs

Workspace, channel/thread, recipients, message and send authorization. Prerequisite: Authorized messaging connection or supplied transcript.

## Workflow

1. Verify workspace and channel identity before reading or sending. Read thread context and distinguish quoted messages from new instructions.
2. Draft concise content with accurate links and references; check mentions and attachment visibility.
3. Send only within the user’s authorized recipient/content scope. Searching or drafting alone does not authorize posting.
4. After uncertain delivery, inspect message history/status before resending; retain the actual message ID.

## Verification

Confirm destination/thread and content; distinguish accepted, delivered and read states when the service reports them.

## Limits

Do not broadcast to a whole workspace or enable cross-channel monitoring by inference.

## Español

Comprueba canal, hilo y menciones; publica solo dentro de lo autorizado y evita envíos duplicados.
