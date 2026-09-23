---
name: test-strategy
description: Select meaningful unit, integration and end-to-end checks from behavior and failure risk.
metadata:
  version: 1.0.0
  category: engineering
  tags: test, strategy
  aliases: estrategia pruebas, unitarias, integración, cobertura, regresión
  required_tools: file_read,file_write,command_run
  requires: Project test tools and observable requirements
  effects: mixed
---
# Test Strategy

## Scope

Select meaningful unit, integration and end-to-end checks from behavior and failure risk.

## Inputs

Changed behavior, consumers, known failure modes and practical execution constraints. Prerequisite: Project test tools and observable requirements.

## Workflow

1. Map requirements to externally observable outcomes. Use the lowest layer that reproduces the actual failure, including database/framework behavior when it matters.
2. Keep deterministic local tests for logic and contracts; use integration checks for boundaries that mocks would erase. Avoid tests that merely repeat implementation details.
3. Include representative invalid inputs, concurrency or cancellation only where relevant. Record why a slow/flaky end-to-end check is needed.
4. Separate environment failures from product failures and preserve failure evidence. Do not weaken assertions simply to obtain a green run.

## Verification

Demonstrate that a regression case fails for the faulty behavior and passes after correction when practical.

## Limits

Coverage percentages alone do not prove correctness. A reversible low-impact prose/style edit need not create a new test suite.

## Español

Prueba resultados y fallos reales en la capa adecuada; no sustituyas calidad por porcentaje de cobertura.
