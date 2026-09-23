---
name: wearable-data-analysis
description: Summarize supplied or connected wearable sleep, activity and training records with unit, session-identity and missing-day checks. Análisis de datos de actividad y sueño.
metadata:
  version: 1.0.0
  category: personal
  tags: wearable, sleep, activity, coverage
  aliases: Fitbit, Oura, Garmin, Strava, registros de sueño, pasos, wearable trends
  required_tools: file_read,file_write,command_run,tool_search,web_read
  requires: Owner-supplied exports or an existing authorized provider connection
  effects: local-write
---
# Wearable Data Analysis

## Scope

Use for descriptive analysis of personal wearable records. Keep medical diagnosis, treatment and exercise prescriptions outside this data-summary workflow.

## Inputs

Question, files/provider, period, timezone, metric definition, device changes and available synchronization timestamps.

## Workflow

1. Inspect only the requested data source and its actual schema. Use a supplied historical export directly; a failed live sync does not make a dated export unusable. Label last sync, covered interval and incomplete current days separately.
2. Preserve provider record IDs and revisions, measurement time, local date attribution and device. Verify whether a sleep record belongs to its starting or ending date before interpreting last night. Keep naps, main sleep and overlapping sessions distinguishable.
3. Normalize declared units explicitly. Duration milliseconds, seconds and minutes are different fields; missing wear time or sensor data is not zero activity. Retain original values for traceability.
4. Deduplicate synced copies by identity/provenance before aggregation. Do not add two devices' measurements of the same walk or night. Document the chosen primary source and retain unresolved conflicts; different providers' recovery scores are not a common calibrated scale.
5. Compare like periods with coverage denominators, complete-day counts and relevant device/firmware changes. Show missingness and outliers before interpreting a trend. A correlation between sleep and activity does not establish causation.
6. Produce a concise sourced summary with observed values, uncertainty and data limitations. Route health interpretation to current authoritative guidance when requested; do not infer a diagnosis from a wearable score.

## Verification

Reconcile sessions, days, duplicates and excluded records. Independently check units and sample aggregates against the original export. State what was measured, estimated by the provider or inferred in the analysis.

## Limits

No provider integration or health database is installed. Do not import credential archives, enable continuous syncing or copy raw health records to external services merely to prepare a summary.

## Español

Conserva sesiones, fechas, unidades y cobertura. Distingue ausencia de cero y evita sumar la misma actividad de varios dispositivos. Resume observaciones sin convertir puntuaciones de consumo en diagnósticos.
