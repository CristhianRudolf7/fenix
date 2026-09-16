---
name: environment-and-dependencies
description: Prepare development environments and diagnose dependency, build and installation failures. Entornos y dependencias.
version: 1.1.0
required_tools: command_run,command_status,file_read
---

# Environment And Dependencies

## Procedure

Inspect the existing lockfile and interpreter/toolchain before installing. Prefer the project-supported isolated environment and pinned dependencies. Read the first actionable error; do not repeat the same installation unchanged. Separate network failure, missing executable, version incompatibility and build failure. A running installation with no recent output may still be working: wait for its process event or bounded status. Preserve successful setup and do not cancel progressing work just to inspect files. Verify imports or the actual consumer command inside the intended environment.

## Local diagnostic recovery

Separate a failure in the diagnostic tool or container from a failure in the target program. Read the exact error, including the end of the output. If it identifies an environment constraint, inspect that constraint before rerunning. Apply only an authorized, reversible, process-local correction; do not change host-wide settings or invent application behavior. Then rerun the original diagnostic and report whether it actually observed the program. A tool that could not start is not evidence that the code is correct. Ask the owner about genuinely unresolved requirements or missing permission, not facts that the available files or diagnostics already establish.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Comprueba versiones y lockfiles. Clasifica errores de red, versión y compilación. Conserva instalaciones exitosas y verifica en el entorno correcto.

Distingue un fallo de la herramienta de diagnóstico de un fallo del programa. Comprueba el error exacto, corrige solo condiciones locales reversibles autorizadas y vuelve a ejecutar la comprobación. Pregunta cualquier duda que siga sin resolverse.
