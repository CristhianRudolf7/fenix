---
name: package-build-and-install
description: Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.
version: 1.0.0
required_tools: command_run,command_status,file_read,file_write
---

# Package Build And Install

## Procedure

Inspect the packaging layout and build metadata. Build the distributable using the declared backend. Inspect the produced artifact. If serving an index, verify the actual package-specific endpoint and links using the relevant packaging standard, not just the root listing. Keep the service supervised with persistent=true. Test installation from a separate clean environment using the requested source; a message such as Requirement already satisfied does not exercise that source. Import and use the installed package from outside the source checkout. Record the artifact, source URL and fresh-install result. Do not hard-code benchmark package names or hidden expected output.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Construye el artefacto y prueba una instalación nueva desde la fuente solicitada, fuera del repositorio. Una dependencia ya instalada no valida el índice.
