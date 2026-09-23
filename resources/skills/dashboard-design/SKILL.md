---
name: dashboard-design
description: Create decision-focused dashboards with traceable metrics, comparable periods and readable visual encodings.
metadata:
  version: 1.1.0
  category: data
  tags: dashboard, design
  aliases: dashboard, tablero, KPI, indicadores, panel datos
  required_tools: file_read,file_write,command_run
  requires: Dataset/metric definitions and requested output tool
  effects: local-write
---
# Dashboard Design

## Scope

Create decision-focused dashboards with traceable metrics, comparable periods and readable visual encodings.

## Inputs

Audience, decision, metric sources, refresh cadence and filtering needs. Prerequisite: Dataset/metric definitions and requested output tool.

## Workflow

1. Define each metric’s numerator, denominator, units, timezone and owner. Distinguish stock from flow and totals from rates.
2. Select visuals suited to comparison, trend or distribution. Preserve scales and show missing data instead of smoothing it away.
3. Use filters consistently across cards and plots; report freshness and coverage alongside the numbers.
4. Build with available tools and drill down to source records where authorized; avoid decorative precision unsupported by data.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Recompute representative numbers independently and check responsive layout, labels and comparison periods.

## Limits

A dashboard does not create data access or prove causation. Never mix currencies or incompatible cohorts into one unlabeled total.

## Español

Diseña métricas trazables y comparables, con unidades, fecha de actualización y filtros coherentes.
