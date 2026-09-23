---
name: experiment-and-benchmark-design
description: Design reproducible experiments and fair model or harness benchmarks with quality, latency, token and
  cost accounting.
metadata:
  version: 1.0.0
  category: research
  tags: experiment, and, benchmark, design
  aliases: benchmark, experimento, evaluación, tokens, latencia, costo
  required_tools: file_read,file_write,command_run
  requires: Defined task set, evaluator and authorized execution resources
  effects: local-write
---
# Experiment And Benchmark Design

## Scope

Use when evaluating changes, skills or models. Improving the training cases is not evidence of improvement on unseen tasks.

## Inputs

Hypothesis, baseline/candidate, task population, success criteria, versions and available execution budget.

## Workflow

1. Freeze tasks and scoring before running. Separate development cases from held-out cases and include realistic failures and ambiguous inputs.
2. Control model settings, harness version, environment and input order. Use repetitions or paired tasks when variability matters; record seeds where supported.
3. Capture task outcome, elapsed time, retries, errors and provider usage. Distinguish measured billing from estimates and cached/reasoning tokens from ordinary output.
4. Keep traces with secrets removed. Attribute failure to task/environment/tool/protocol/model only when evidence supports that diagnosis.
5. Compare quality and resource use together. Report incomplete runs in the denominator, uncertainty and the boundary of what the task set represents.

## Verification

Recompute aggregate metrics from per-task records. Preserve failed and timed-out trials; never replace them silently with successful retries.

## Limits

This skill does not grant permission to spend on new providers or run generated code outside isolation. Small samples are exploratory, not a universal ranking.

## Español

Congela tareas y criterios, separa casos de desarrollo y evaluación y registra fallos, tiempo, tokens y costes. No ocultes reintentos ni extrapoles muestras pequeñas.
