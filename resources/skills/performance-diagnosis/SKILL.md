---
name: performance-diagnosis
description: Measure and diagnose latency, memory, CPU or throughput regressions before optimizing the relevant
  bottleneck.
metadata:
  version: 1.3.0
  category: engineering
  tags: performance, diagnosis
  aliases: rendimiento, lento, RAM, CPU, latencia, optimizar, optimization loop, freshness age, bucle de optimización, frescura de datos
  required_tools: command_run,command_status,file_read,files_search
  requires: A reproducible workload and available profiling tools
  effects: local-write
---
# Performance Diagnosis

## Scope

Use for measurable slowness or resource growth. Use test-driven-repair for functional bugs and process-and-service-management for process lifecycle.

## Inputs

Workload, expected behavior, observed metric, environment and recent changes.

## Workflow

1. Record a baseline under representative input, with warm/cold conditions and concurrency stated. Separate startup, queueing, network and execution time.
2. Use existing traces, timing and a profiler when available to identify where time or memory accumulates. Prefer targeted instrumentation over unrestricted logs.
3. Form a falsifiable bottleneck hypothesis. Change one relevant factor and compare under the same workload; avoid tuning unrelated code.
4. Check throughput and tail latency together, and distinguish resident memory from heap allocations, cache and leaks. Include steady-state and cleanup behavior.
5. Remove temporary instrumentation that exposes sensitive data and deliver the measured before/after result with tradeoffs.

For browser regressions, record URL/build, identity, viewport, data, browser version and cache/network conditions. Measure the actual user journey and relevant navigation, rendering, interaction or resource costs; page load timing alone does not establish interaction responsiveness. Missing measurements stay unknown, and zero baseline values cannot yield a meaningful percentage change. Keep failed navigations separate from slow successful runs.

Retain the comparison baseline instead of overwriting it with the current result. Report repeated samples and variability, and use the project's agreed budget or observed baseline for a regression decision. A single cold run against a warm baseline is not an improvement or regression claim.

For repeated optimization, define a finite time/variant/resource budget and an unchanged correctness gate. Keep each hypothesis, input identity, measurement, failure and accepted baseline; compare against the best verified candidate, not just the previous attempt. Confirm the winner with repeated comparable runs and describe it as the best measured option, not a global optimum. For streaming systems measure data age and queue depth separately from request latency: a fast stale response is not a fresh result. Preserve backpressure and validation.

For numerical optimization, first check whether an established library primitive satisfies the actual input domain. Verify algorithm assumptions (for example symmetry, complex eigenpairs or convergence conditions) before replacing it with a custom iteration. Use the supplied correctness/performance evaluator early. Only pursue a more complex method if the observed baseline or task constraints justify it; a fast wrong result is not an optimization.

## Verification

Run correctness checks as well as the performance scenario. Repeat enough to distinguish noise; report sample count and variability.

## Limits

Do not claim improvement from different hardware, input sizes or cache states. Unavailable profiling is a limitation, not evidence that a component is efficient.
