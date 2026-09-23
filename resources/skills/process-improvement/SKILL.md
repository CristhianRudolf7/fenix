---
name: process-improvement
description: Analyze repeated work and design a simpler process with measurable handoffs and exceptions.
metadata:
  version: 1.0.0
  category: business
  tags: process, improvement
  aliases: mejorar proceso, automatización negocio, flujo trabajo, capacidad
  required_tools: file_read,file_write,tool_search
  requires: Observed process, case samples and available operational data
  effects: local-write
---
# Process Improvement

## Scope

Analyze repeated work and design a simpler process with measurable handoffs and exceptions.

## Inputs

Outcome, actors, inputs, bottlenecks and change constraints. Prerequisite: Observed process, case samples and available operational data.

## Workflow

1. Map the actual sequence, queues, decisions and rework rather than an idealized policy diagram.
2. Measure arrival volume, service time and variability; distinguish active work from waiting and do not size capacity from averages alone.
3. Identify unnecessary handoffs and failure-prone repetition. Automate stable rules while leaving ambiguous exceptions visible to the responsible person.
4. Pilot a bounded change and compare quality, elapsed time and workload; document rollback and ownership.

## Verification

Check that the process still satisfies required controls and that improvement is not shifted work or hidden errors.

## Limits

Do not replace human approval required by the real business process or change staffing commitments from a speculative model.

## Español

Mide trabajo y espera reales, reduce reprocesos y valida mejoras sin ocultar errores ni trasladar carga.
