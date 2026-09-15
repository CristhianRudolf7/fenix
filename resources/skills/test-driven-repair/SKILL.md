---
name: test-driven-repair
description: Reproduce software bugs, implement focused fixes and verify behavior and regressions. Reparar errores y pruebas.
version: 1.0.0
required_tools: file_read,file_edit,file_patch,command_run
---

# Test Driven Repair

## Procedure

Translate the requested behavior into observable examples without inventing requirements. Reproduce the failure using a minimal test. Inspect the relevant interface before implementing: inheritance, argument conventions, return types and side effects matter. Make one focused change and run the failing test plus nearby regressions. An import or successful build does not prove behavior. For terminal adapters test actual input delivery, newline, interrupt and output, not merely object creation. If tests cannot run, report the specific blocker and unverified claims. Seek an independent read-only review for material uncertainty that executable checks cannot resolve; do not create a reviewer for routine successful edits.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Reproduce el fallo, revisa contratos e interfaces, corrige y ejecuta pruebas funcionales y regresiones. Distingue compilación de comportamiento.
