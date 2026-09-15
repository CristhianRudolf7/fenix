---
name: test-driven-repair
description: Reproduce software bugs, implement focused fixes and verify behavior and regressions. Reparar errores y pruebas.
version: 1.1.0
required_tools: file_read,file_edit,file_patch,command_run
---

# Test Driven Repair

## Procedure

Translate the requested behavior into observable examples without inventing requirements. Reproduce the failure using a minimal test. Inspect the relevant interface before implementing: inheritance, argument conventions, return types and side effects matter. Once the failing path is located, make a focused change and run the failing test plus nearby regressions; repeated unchanged reads are not a repair.

An import or successful build does not prove behavior. For crashes or memory corruption, inspect ownership, lifetime, bounds and allocation size against the actual data layout. Compare the affected build modes when relevant; debug success does not establish optimized-build correctness. Use an available sanitizer or memory checker to localize a suspected memory fault, and rerun the failing scenario after editing. Do not hide the failure by removing required behavior, disabling checks or changing protected tests.

For terminal adapters, test actual input delivery, newline, interrupt and output, not merely object creation. If tests cannot run, report the specific blocker and unverified claims. Seek an independent read-only review for material uncertainty that executable checks cannot resolve; do not create a reviewer for routine successful edits.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Reproduce el fallo, revisa contratos e interfaces, corrige y ejecuta pruebas funcionales y regresiones. Tras localizar el problema, cambia y comprueba en vez de releer sin avance. En fallos de memoria, revisa propiedad, vida útil, límites y tamaños; verifica los modos de compilación pertinentes. Distingue compilación de comportamiento y conserva las pruebas y restricciones existentes.
