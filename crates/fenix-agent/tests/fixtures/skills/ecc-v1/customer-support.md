---
name: customer-support
description: Triage support cases and draft accurate responses using account context and known evidence.
metadata:
  version: 1.0.0
  category: business
  tags: customer, support
  aliases: soporte cliente, ticket, reclamación, atención, escalamiento
  required_tools: file_read,file_write,tool_search
  requires: Supplied case or authorized support-system connection
  effects: local-write
---
# Customer Support

## Scope

Triage support cases and draft accurate responses using account context and known evidence.

## Inputs

Customer issue, impact, account scope, history and support policy. Prerequisite: Supplied case or authorized support-system connection.

## Workflow

1. Read the conversation and previous actions before asking the customer to repeat information. Verify account/resource scope.
2. Classify impact and reproduce or inspect evidence where possible. Distinguish product defect, configuration, outage and misunderstanding.
3. Draft a clear next step and realistic expectation. Escalate with reproduction, logs stripped of secrets and attempted fixes.
4. Update or send only within authorization; retain case IDs and avoid closing a ticket solely because a reply was sent.

## Verification

Check the proposed answer against product behavior and confirm the actual resolution when evidence permits.

## Limits

Do not promise refunds, timelines or features outside policy, and do not expose another customer’s data.

## Español

Revisa historial y evidencia, propone pasos concretos y escala con contexto sin prometer resultados no autorizados.
