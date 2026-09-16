---
name: environment-and-dependencies
description: Prepare development environments and diagnose dependency, build and installation failures. Entornos y dependencias.
version: 1.0.0
required_tools: command_run,command_status,file_read
---

# Environment And Dependencies

## Procedure

Inspect the existing lockfile and interpreter/toolchain before installing. Prefer the project-supported isolated environment and pinned dependencies. Read the first actionable error; do not repeat the same installation unchanged. Separate network failure, missing executable, version incompatibility and build failure. A running installation with no recent output may still be working: wait for its process event or bounded status. Preserve successful setup and do not cancel progressing work just to inspect files. Verify imports or the actual consumer command inside the intended environment.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Comprueba versiones y lockfiles. Clasifica errores de red, versión y compilación. Conserva instalaciones exitosas y verifica en el entorno correcto.
