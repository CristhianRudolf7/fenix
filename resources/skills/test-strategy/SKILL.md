---
name: test-strategy
description: Select meaningful unit, integration and end-to-end checks from behavior and failure risk.
metadata:
  version: 1.1.0
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

Before adding a test, name the realistic defect it would catch. Derive expected values from the requirement or an independently checked fixture, not the same helper being tested. Keep the real boundary behavior under test; mock slow or external effects at the boundary that preserves the relevant contract. Assertions about calls are useful when call arguments, order or count are themselves that contract.

For regression repairs, observe the test fail on the defect and pass after correction when feasible. If implementation already exists, use a disposable copy or controlled mutation to check sensitivity instead of deleting working code. Report when only the passing half was observed. Rerun required project checks and the affected integration surface; do not imply a full-suite pass from a narrow run.

## Verification

Demonstrate that a regression case fails for the faulty behavior and passes after correction when practical.

## Limits

Coverage percentages alone do not prove correctness. A reversible low-impact prose/style edit need not create a new test suite.

## Español

Prueba resultados y fallos reales en la capa adecuada; no sustituyas calidad por porcentaje de cobertura.

Define qué defecto detecta cada prueba y deriva resultados esperados de forma independiente. Conserva el comportamiento real relevante al simular dependencias. Verifica fallo y corrección, o declara que solo observaste el resultado correcto; no borres implementación válida para recrear una secuencia de pruebas.
