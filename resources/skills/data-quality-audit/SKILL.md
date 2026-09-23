---
name: data-quality-audit
description: Audit dataset completeness, validity, joins and lineage before analysis or ingestion.
metadata:
  version: 1.0.0
  category: data
  tags: data, quality, audit
  aliases: calidad datos, faltantes, duplicados, consistencia, validación
  required_tools: file_read,file_write,command_run
  requires: Readable dataset and its intended schema
  effects: local-write
---
# Data Quality Audit

## Scope

Audit dataset completeness, validity, joins and lineage before analysis or ingestion.

## Inputs

Observation unit, field meanings, expected ranges and source provenance. Prerequisite: Readable dataset and its intended schema.

## Workflow

1. Profile actual types and missing-value encodings without modifying the source. Distinguish absent, zero, unknown and not applicable.
2. Check keys, duplicate records, referential relationships and join cardinality. A valid type does not establish a plausible business value.
3. Validate units, time coverage and distribution shifts against the documented contract. Investigate anomalies before deleting them.
4. Return row/field evidence, affected counts and proposed corrections with reversible transformations.

## Verification

Reconcile all records into accepted, rejected or unresolved groups and preserve the original hash.

## Limits

Do not create an arbitrary composite quality score or impute missing values without documenting the model/assumption.

## Español

Audita claves, unidades, faltantes y cruces; conserva evidencia y no borres anomalías sin analizarlas.
