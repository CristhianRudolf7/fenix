---
name: package-build-and-install
description: Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.
metadata:
  version: 1.1.0
  category: engineering
  tags: package, build, and, install
  aliases: paquete, distribución, wheel, instalación limpia
  required_tools: command_run,command_status,file_read,file_write
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Package Build And Install

## Scope

Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.

## Inputs

Package source, declared build backend, artifact and consumer environment.

## Workflow

Inspect the packaging layout and build metadata. Build the distributable using the declared backend. Inspect the produced artifact. If serving an index, verify the actual package-specific endpoint and links using the relevant packaging standard, not just the root listing. Keep the service supervised with persistent=true. Test installation from a separate clean environment using the requested source; a message such as Requirement already satisfied does not exercise that source. Import and use the installed package from outside the source checkout. Record the artifact, source URL and fresh-install result. Do not hard-code benchmark package names or hidden expected output.

## Verification

Install the produced artifact into a clean environment and use it outside the source checkout.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Construye el artefacto y prueba una instalación nueva desde la fuente solicitada, fuera del repositorio. Una dependencia ya instalada no valida el índice.
