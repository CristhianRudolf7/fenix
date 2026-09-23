---
name: conversion-optimization
description: Diagnose conversion drop-off in landing pages, forms, pricing or onboarding and propose measurable improvements. CRO, abandono de formularios y activación.
metadata:
  version: 1.0.0
  category: business
  tags: conversion, funnel, activation, cro
  aliases: CRO, conversion rate, landing-page conversion, optimizar conversiones, abandono, activación, embudo
  required_tools: web_read,file_read,file_write,tool_search
  requires: Target page or flow and available audience, traffic and outcome evidence
  effects: local-write
---
# Conversion Optimization

## Scope

Use when the desired outcome is more successful user journeys or qualified conversions. Use marketing-content for copy alone, frontend-design-and-accessibility for implementation, and ab-test-analysis to analyze a randomized experiment. A URL alone does not establish a conversion problem.

## Inputs

User goal, funnel entry and completion events, eligible audience, observation window, traffic source, page/flow and existing research. Use supplied evidence before asking for unavailable analytics.

## Workflow

1. Define the denominator: eligible people, sessions or attempts, with deduplication and the conversion window. Separate technical failure, instrumentation loss and user abandonment. Segment by relevant acquisition source, device or new/returning status before interpreting an aggregate change.
2. Walk the actual journey from the entry promise to the completed outcome. Inspect message match, plan/price clarity, required fields, validation, loading, recovery and mobile/keyboard interaction. Record the exact step and evidence for each friction point.
3. Distinguish an observed defect from a behavioral hypothesis. Match the intervention to the cause: repair a broken submit action, clarify an ambiguous requirement, or test a proposition. Preserve necessary disclosures, usable cancellation and accessible controls.
4. For each proposed experiment, specify audience, change, mechanism, primary outcome and guardrails such as qualified leads, refunds, support load or successful activation. Decide duration/stopping rules before inspecting results. Use existing experimentation infrastructure; do not silently enroll live users.
5. Rank proposals using evidence, expected mechanism and implementation effort. Label uncertain effect sizes; do not invent uplift. With low traffic, use observed task completion or interviews to reduce uncertainty and explain that they do not estimate causal conversion lift.

## Verification

Check event definitions and totals against source records. Test the corrected journey and its failure/recovery paths. A click increase is not necessarily an improvement in completed or qualified conversions; report the measured window and population.

## Limits

Do not manufacture scarcity, testimonials or consent. Deliver an audit, experiment specification or authorized implementation as requested; publishing campaigns and enabling experiments are separate effects.

## Español

Define eventos, población y periodo antes de atribuir abandono. Recorre el flujo real, separa errores de hipótesis y propone cambios medibles con métricas de calidad. Sin volumen suficiente, comunica incertidumbre; no prometas aumentos de conversión ni dificultes cancelar.
