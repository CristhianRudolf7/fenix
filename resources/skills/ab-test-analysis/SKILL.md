---
name: ab-test-analysis
description: Design or analyze randomized product experiments with assignment integrity and decision uncertainty.
metadata:
  version: 1.0.0
  category: data
  tags: ab, test, analysis
  aliases: A/B, experimento producto, significancia, tamaño muestra
  required_tools: file_read,file_write,command_run
  requires: Experiment definition and assignment/outcome data
  effects: local-write
---
# Ab Test Analysis

## Scope

Design or analyze randomized product experiments with assignment integrity and decision uncertainty.

## Inputs

Hypothesis, randomization unit, primary metric, guardrails and stopping rule. Prerequisite: Experiment definition and assignment/outcome data.

## Workflow

1. Specify analysis and practical effect of interest before inspecting outcomes. Account for clustering or repeated observations at the randomization unit.
2. Check assignment ratios, exposure, contamination and missing outcomes. Separate intention-to-treat from analyses restricted to exposed users.
3. Estimate effect and uncertainty with an appropriate method; handle multiple comparisons or sequential monitoring when actually used.
4. Compare the result to practical impact and guardrails, including cost and implementation risk; label inconclusive results clearly.

## Verification

Reproduce counts and estimates from raw aggregates and examine sensitivity to exclusions.

## Limits

Do not stop on the first favorable p-value or interpret a nonsignificant result as proof of equivalence.

## Español

Verifica asignación y unidad experimental; informa efecto e incertidumbre sin detenerte por un resultado favorable aislado.
