---
name: data-analysis
description: Clean and analyze tabular datasets with reproducible transformations, checked units and honest statistical
  conclusions.
metadata:
  version: 1.2.0
  category: research
  tags: data, analysis
  aliases: análisis de datos, csv, estadística, comparar grupos, limpiar datos
  required_tools: file_read,file_write,command_run,document_read
  requires: Readable data and a suitable local computation environment
  effects: local-write
---
# Data Analysis

## Scope

Use for data questions and statistics. Use spreadsheet skills for file operations; use sql-analysis-and-optimization for SQL semantics and query performance.

## Inputs

Dataset, observation unit, variable definitions, question, grouping and missing-value conventions.

## Workflow

1. Profile schema, row counts, types, duplicates and missingness before changing data. Preserve originals and record the input hash and transformations.
2. Resolve units, locale, time range and join keys. Check join cardinality so duplicated matches do not inflate totals.
3. Choose the analysis from the question and study design. Distinguish paired from independent observations, repeated subjects and clustered sampling.
4. Compute with code and retain executable steps. Report sample sizes, effect magnitudes and uncertainty; do not equate correlation or a small p-value with causation.
5. Check sensitivity to missing values and outliers where it changes the conclusion. Create clear tables or figures with units and source notes.

Keep this workflow for answering a dataset question. Use data-quality-audit for a standalone integrity report, ab-test-analysis for randomized comparisons, time-series-forecasting for future horizons, and data-pipelines for recurring ingestion. Load a specialist only when that decision is present.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Reconcile totals and row counts before/after transformations. Spot-check against raw records and explain excluded rows.

## Limits

Do not select a test only because it yields significance. Avoid sending raw sensitive records to external services; publish aggregates appropriate to the task.
