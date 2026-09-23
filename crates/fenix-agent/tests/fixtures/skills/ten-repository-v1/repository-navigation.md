---
name: repository-navigation
description: Orient in unfamiliar repositories; discover entrypoints, instructions, build and test commands. Repositorio,
  estructura, pruebas.
metadata:
  version: 1.1.0
  category: engineering
  tags: repository, navigation
  aliases: repositorio, estructura, navegar código, símbolos
  required_tools: files_glob,files_search,file_read,command_run
  requires: Available task-authorized Fénix tools
  effects: read-only
---
# Repository Navigation

## Scope

Orient in unfamiliar repositories; discover entrypoints, instructions, build and test commands. Repositorio, estructura, pruebas.

## Inputs

Repository root, question and known relevant paths.

## Workflow

Read applicable repository instructions. Identify manifests, entrypoints and existing test commands before changing code. Search symbols narrowly; read bounded relevant sections instead of repeatedly listing the entire tree. Record relevant paths and the exact commands needed to reproduce and verify. Use configured code_query when available; absence of a language server is not a blocker. Do not assume a framework from a directory name. Verify the discovered command against the manifest and working directory.

## Verification

Match discovered entrypoints and commands to actual manifests and callers.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Lee instrucciones y manifiestos, ubica puntos de entrada y pruebas. Busca por símbolos y conserva rutas y comandos comprobados.
