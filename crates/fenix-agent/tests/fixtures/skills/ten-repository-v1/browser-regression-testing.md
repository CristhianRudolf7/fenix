---
name: browser-regression-testing
description: Create reproducible browser regression checks for user flows and diagnose flaky interactions.
metadata:
  version: 1.0.0
  category: engineering
  tags: browser, regression, testing
  aliases: pruebas navegador, playwright, e2e, flaky, regresión web
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

## Verification

Rerun the specific failure and relevant neighboring flows; report exact browser/device coverage.

## Limits

Chromium emulation is not an actual mobile-device test. Retrying until success hides intermittent failures.

## Español

Usa estados aislados y condiciones reales, captura evidencia y distingue emulación de pruebas en dispositivos.
