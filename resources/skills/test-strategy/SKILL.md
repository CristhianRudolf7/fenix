---
name: test-strategy
description: Select meaningful unit, integration and end-to-end checks from behavior and failure risk.
metadata:
  version: 1.6.0
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

First reuse existing checks and tool receipts. Before adding a test, identify the requested outcome and realistic defect it would catch; keep this reasoning brief in the existing work, not a separate model review or generated test contract. Add a check only when it covers an unresolved requirement or a plausible failure introduced by the change. Batch independent inexpensive checks and stop after relevant checks pass unless new evidence justifies more. An import-only check does not establish cancellation, data integrity or behavior under load. Derive expected values from the requirement or an independently checked fixture, not the same helper being tested. Keep the real boundary behavior under test; mock slow or external effects at the boundary that preserves the relevant contract. Assertions about calls are useful when call arguments, order or count are themselves that contract.

For regression repairs, observe the test fail on the defect and pass after correction when feasible. If implementation already exists, use a disposable copy or controlled mutation to check sensitivity instead of deleting working code. Report when only the passing half was observed. Rerun required project checks and the affected integration surface; do not imply a full-suite pass from a narrow run.

## Focused execution checks

For differential tests, snapshot the initial fixture and give each implementation an independent restored copy. Exercise the requested successful mutation and assert that the expected state actually changed; equality after two rejected/no-op executions is not evidence for the success path. Add a relevant rejection only when needed, and preserve the original data. After a repair, rerun the affected acceptance paths on the final artifact.

## Requirement-linked checks

Separate hard requirements from preferences and evaluate every applicable hard condition on the final candidate. File existence, schema validity and nonnegative counts are not semantic checks. Choose a small independent counterexample that would expose the likely mistake; derive expected values from the request or inspected inputs. Keep units and boundary conventions consistent. Reject always-true assertions and checks that simply repeat the chosen output. No extra model reviewer is needed.

For costly numerical or data checks, begin with a tiny deterministic counterexample or invariant relevant to the requirement. Check units, support/range, boundary conventions and degenerate equal-value cases where applicable before increasing sample size. For probabilistic algorithms, use tolerances and an independent reference instead of expecting identical random draws. A small passing case does not replace the requested full run or output artifacts. Measure a slow check by phase/progress before scaling or changing the algorithm; silence alone is not failure.

Acceptance probes must fail when a required outcome is missing or wrong. Use assertions and propagate exit status; printing "absent", an error message, or appending `|| true` does not establish a passing check. A diagnostic may deliberately tolerate failure to collect evidence, but keep that result distinct from acceptance. For a deliberately long-running program, verify the requested artifact/protocol within a bounded wait and distinguish expected timeout from application failure.

## Verification

Demonstrate that a regression case fails for the faulty behavior and passes after correction when practical.

## Limits

Coverage percentages alone do not prove correctness. A reversible low-impact prose/style edit need not create a new test suite.
