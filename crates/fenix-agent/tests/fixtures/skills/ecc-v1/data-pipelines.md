---
name: data-pipelines
description: Build ETL or ELT pipelines with stable keys, incremental state and recoverable data flow.
metadata:
  version: 1.0.0
  category: data
  tags: data, pipelines
  aliases: pipeline datos, ETL, ELT, airflow, dbt
  required_tools: file_read,file_write,command_run
  requires: Existing data stack and authorized source/sink access
  effects: local-write
---
# Data Pipelines

## Scope

Build ETL or ELT pipelines with stable keys, incremental state and recoverable data flow.

## Inputs

Sources, schema, freshness target, transformations and write semantics. Prerequisite: Existing data stack and authorized source/sink access.

## Workflow

1. Define input/output contracts and lineage before transformations. Preserve raw source data or a reproducible source version.
2. Use stable business keys and explicit watermarks; consider late arrivals, updates and deletions rather than only new rows.
3. Make batch publication atomic where possible and handle retries without double counting. Separate transformation failure from delivery failure.
4. Record per-stage counts, rejected rows and cursor state; advance watermarks only after durable publication.

## Verification

Replay a batch, interrupt near commit and test late/corrected data; reconcile source and sink totals.

## Limits

Do not assume that file modification time is a complete change feed or silently discard malformed records.

## Español

Define contratos y cursores, conserva rechazados y prueba reintentos y datos tardíos sin duplicados.
