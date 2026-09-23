---
name: performance-diagnosis
description: Measure and diagnose latency, memory, CPU or throughput regressions before optimizing the relevant
  bottleneck.
metadata:
  version: 1.1.0
  category: engineering
  tags: performance, diagnosis
  aliases: rendimiento, lento, RAM, CPU, latencia, optimizar
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

## Verification

Run correctness checks as well as the performance scenario. Repeat enough to distinguish noise; report sample count and variability.

## Limits

Do not claim improvement from different hardware, input sizes or cache states. Unavailable profiling is a limitation, not evidence that a component is efficient.

## Español

Mide primero y localiza el cuello de botella. Compara con la misma carga y verifica corrección, latencia y memoria sin ocultar variabilidad.

En navegador compara la misma versión, sesión, datos y condiciones. Conserva la línea base y la variabilidad; carga de página no equivale a respuesta a interacciones y una medición ausente no es cero.
