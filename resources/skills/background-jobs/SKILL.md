---
name: background-jobs
description: Design durable job execution with leases, idempotency, retries and explicit partial effects.
metadata:
  version: 1.0.0
  category: engineering
  tags: background, jobs
  aliases: cola, worker, trabajos, reintentos, idempotencia
  required_tools: file_read,file_write,command_run
  requires: Existing database/queue and worker runtime
  effects: mixed
---
# Background Jobs

## Scope

Design durable job execution with leases, idempotency, retries and explicit partial effects.

## Inputs

Job outcome, side effects, delivery semantics and cancellation policy. Prerequisite: Existing database/queue and worker runtime.

## Workflow

1. Persist intent and identify the job before execution. Distinguish queued, leased, running, retryable, completed and terminal failure states.
2. Acquire work atomically and renew or expire leases. Use fencing/version checks when stale workers could commit effects after losing ownership.
3. Treat delivery as potentially repeated; make effects idempotent or use business keys/outbox records. Retry transient failures with recorded attempts, not recursive spawning.
4. On cancellation or restart, inspect committed effects and resume from durable state. Route exhausted work to an observable failure path.

## Verification

Test two workers claiming the same job, worker crash around commit and duplicated delivery.

## Limits

Exactly-once execution cannot be assumed across arbitrary external services. A queue acknowledgement is not business completion.

## Español

Persistencia, leases e idempotencia deben controlar duplicados y recuperación; comprueba caídas junto al commit.
