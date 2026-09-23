---
name: browser-regression-testing
description: Create reproducible browser regression checks for user flows and diagnose flaky interactions.
metadata:
  version: 1.3.0
  category: engineering
  tags: browser, regression, testing
  aliases: pruebas navegador, playwright, e2e, flaky, regresión web, checkout regression, browser QA, pruebas de formularios, evidencia visual, regresión de formulario, click path, sequential state reset, botón no hace nada, conflicto de estado
  required_tools: file_read,file_write,command_run
  requires: Available browser automation or project E2E runner
  effects: mixed
---
# Browser Regression Testing

## Scope

Create reproducible browser regression checks for user flows and diagnose flaky interactions.

## Inputs

App URL, test user/data, flow and expected outcome. Prerequisite: Available browser automation or project E2E runner.

## Workflow

1. Use isolated test accounts/profiles and known starting data. Observe the application’s readiness rather than sleeping a fixed duration.
2. Choose stable semantic locators and assert business-visible results, not only successful clicks. Keep tests independent of execution order.
3. Capture relevant trace, screenshot, console and network evidence on failure. Determine whether the issue is application state, selector ambiguity or timing before retrying.
4. Exercise requested viewport/browser variants and cleanup only test-created resources. Keep real user sessions separate from testing.

Before comparing screenshots, verify the final URL, signed-in identity, fixtures, viewport and application state: a login wall or loading fallback is not the requested page. For a broken interaction, retain the initial state, action, observable result and relevant console/network evidence under a stable issue ID. For a static defect, a located screenshot may suffice. Recheck the same flow after correction and retain new regressions. Wait for the required state or response; network silence is not a universal readiness condition. Report unvisited flows as untested, not passed, and avoid a synthetic overall health score with no validated rubric.

Choose report-only or test-and-repair behavior from the user's request. Finding a bug does not authorize source edits, commits or deployment in a report-only audit. Both modes may exercise mutations only on authorized test data; a production checkout or message send is not a harmless click. When repair is included, preserve the original reproduction, verify the same flow after the fix and run its relevant neighbors. Stop repeating a failed approach when evidence no longer improves; report the remaining issue and concrete blocker.

For a new or changed flow, trace the path from its entrypoint to persisted outcome and recovery, including relevant empty/loading/error states. Record which roles and routes were actually exercised; source inspection of a route does not count as browser execution.

When an action appears to run but its result disappears, trace the ordered handler calls, shared-state writes/resets, effects, subscriptions and asynchronous completions. Identify which later update can undo the intended transition. Reproduce rapid repetition, a changed selection during a pending request, cancellation and reload when relevant; verify the settled user-visible and persisted result. Source reasoning identifies a hypothesis, not a browser-confirmed defect.

## Verification

Rerun the specific failure and relevant neighboring flows; report exact browser/device coverage.

## Limits

Chromium emulation is not an actual mobile-device test. Retrying until success hides intermittent failures.

## Español

Usa estados aislados y condiciones reales, captura evidencia y distingue emulación de pruebas en dispositivos.

Distingue informe de QA y corrección autorizada. Conserva reproducción y evidencia antes/después, prueba el resultado persistido y no cuentes rutas leídas como navegadas ni realices transacciones reales por completar un test.

Si una acción se deshace sola, revisa el orden de escrituras, resets y respuestas asíncronas. Comprueba el estado final tras cambios de selección, cancelación o recarga cuando correspondan.
