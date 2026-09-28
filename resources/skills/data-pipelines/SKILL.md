---
name: data-pipelines
description: Merge and normalize JSON CSV Parquet datasets, deduplicate users by source priority, report conflicts; build ETL/ELT pipelines with stable keys and recoverable data flow.
metadata:
  version: 1.4.0
  category: data
  tags: data, pipelines
  aliases: merge datasets, JSON CSV Parquet, deduplicate users, source priority, conflict report, fusionar datos, combinar fuentes, pipeline datos, ETL, ELT, airflow, dbt, backfill throughput, ingestión masiva, acelerar ETL, live tail, conciliación de filas
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

For throughput or catch-up work, load [throughput and accounting](references/throughput-and-accounting.md). Separate the frozen historical boundary from new arrivals, measure pipeline stages and reconcile row identity and transformation semantics before accepting a faster variant.

## Requirement-linked checks

Before writing a parser or aggregation, inspect representative source records and identify the actual field boundaries, types and units. Count the requested field, not matching words in unrelated free text. Preserve a small independent example with potentially misleading content and verify its expected aggregate before processing everything. Validate selected source-to-output values as well as shape and totals. Batch sampling with initial discovery; avoid full redundant rereads.

Inspect real numeric syntax, category values and coordinate/layer ranges before filtering. Validate a few representative parsed records and report selection coverage; an empty or tiny unexpected subset is a diagnostic, not an answer. Preserve relative/absolute modes, units and state transitions. A metadata label is not the underlying content. Reuse a parsed representation instead of repeatedly rescanning while changing unrelated assumptions.

For a one-off file merge (JSON, CSV, Parquet), use a small local transformation rather than introducing an orchestration platform. Normalize field aliases and requested date formats; deduplicate by the user's explicit source priority; preserve and report conflicts. Check output schema, unique keys, selected values and conflict counts against source records. Do not load structured-llm-output just because the input/output is JSON: that skill is for model-generated structured responses.

## Verification

Replay a batch, interrupt near commit and test late/corrected data; reconcile source and sink totals.

## Limits

Do not assume that file modification time is a complete change feed or silently discard malformed records.
