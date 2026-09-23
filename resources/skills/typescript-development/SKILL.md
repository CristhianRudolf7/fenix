---
name: typescript-development
description: Implement TypeScript changes with sound runtime boundaries, discriminated states and compatible module
  exports.
metadata:
  version: 1.0.0
  category: engineering
  tags: typescript, development
  aliases: typescript, javascript, tipos, tsconfig, promesas
  required_tools: file_read,file_write,command_run
  requires: Project Node/browser toolchain and package manager
  effects: mixed
---
# Typescript Development

## Scope

Implement TypeScript changes with sound runtime boundaries, discriminated states and compatible module exports.

## Inputs

Runtime target, tsconfig, package exports and requested behavior. Prerequisite: Project Node/browser toolchain and package manager.

## Workflow

1. Inspect module resolution, build mode and package manager lockfile. Separate type-only guarantees from runtime validation of API or storage data.
2. Model meaningful states with explicit variants when it prevents impossible combinations. Narrow unknown data; avoid casting away an actual mismatch.
3. Trace async errors and stale responses; cancel or ignore obsolete work. Preserve ESM/CJS and browser/server boundaries in imports.
4. Run the actual project typecheck and build plus focused behavior tests. Check generated declarations or exports when the package is consumed externally.

## Verification

Verify both type-level usage and runtime examples, including malformed external input.

## Limits

Do not replace the stack, enable incompatible compiler flags or add advanced generic machinery without a concrete benefit.

## Español

Distingue tipos de validación real, controla respuestas obsoletas y comprueba exports y compilación con la configuración existente.
