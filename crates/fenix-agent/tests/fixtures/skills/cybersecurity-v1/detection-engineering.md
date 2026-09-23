---
name: detection-engineering
description: Author, tune or evaluate security detection rules against actual telemetry, benign negatives and backend field mappings. Reglas Sigma, SIEM y YARA-L.
metadata:
  version: 1.0.0
  category: security
  tags: detection, telemetry, sigma, siem
  aliases: Sigma, SIEM, YARA-L, detection rule, falsos positivos, regla de detección, cobertura de detección
  required_tools: file_read,file_write,command_run,tool_search
  requires: Authorized log samples and target query engine or available rule validator
  effects: local-write
---
# Detection Engineering

## Scope

Use when the deliverable is detection logic, a rule test or a coverage assessment. Use incident-response to investigate an event and security-code-review for source defects. Creating a rule does not enable monitoring or alerts.

## Inputs

Behavior to detect, relevant log sources, schema and timestamps, target engine/version, existing rules, evaluation interval and acceptable false-positive workload.

## Workflow

1. Describe the observable behavior and prerequisites: which events, fields, identities and correlations must exist. Inspect representative records and ingestion coverage before authoring logic. A technique tag or installed rule is not evidence that the necessary telemetry is collected.
2. Express selection, exclusion, grouping and time-window semantics in the actual engine. For Sigma, verify the current specification and conversion backend/pipeline; field names and operators may not carry unchanged between providers. Retain every emitted query and report unsupported constructs or failed files.
3. Build positive examples, realistic benign near-misses and boundary cases such as missing fields, escaped paths, casing, duplicates and late events. Avoid negatives that are trivially unrelated. Keep synthetic records labeled and separate from historical logs.
4. Validate syntax, then evaluate the converted logic against the target schema. Report matched record IDs, misses and false positives. If only a parser or local approximation ran, state that backend behavior remains untested. Do not treat a clean syntax check as detection evidence.
5. Estimate alert volume and analyst effort from a bounded historical interval when access exists. Account for sampling, missing sources and repeated-event suppression. Tune exclusions narrowly and rerun both positive and negative cases.
6. Deliver the rule, field mapping, fixtures, test outcome and remaining gaps. Apply live changes only when requested and within the configured permissions; rule creation, enablement, alert delivery and response actions are distinct effects.

## Verification

Trace each claimed coverage item to a passed behavioral case and available telemetry. Report zero matches with the denominator and source coverage. Retain false negatives and compare neighboring rules before declaring a coverage gap closed.

## Limits

Do not scan hosts or replay attacks to obtain test data by default. No SIEM, log collector or commercial connector is installed by this skill. Framework mappings organize evidence; they are not a security certification.

## Español

Crea reglas sobre campos y registros reales. Valida sintaxis, conversión y comportamiento por separado con positivos, casos benignos y límites. Una etiqueta ATT&CK no demuestra cobertura; publicar la regla tampoco activa alertas. Conserva errores, falsos positivos y datos faltantes.
