---
name: repository-navigation
description: Orient in unfamiliar repositories; discover entrypoints, instructions, build and test commands. Repositorio,
  estructura, pruebas.
metadata:
  version: 1.2.0
  category: engineering
  tags: repository, navigation
  aliases: repositorio, estructura, navegar código, símbolos, context retrieval, recuperación de contexto, callers, contexto acotado
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

Maintain a compact question-to-evidence map: unresolved question, symbol/path, supporting lines and remaining gap. Search first for local implementations and actual callers; refine queries using the repository's own names. Change the query or hypothesis when a search adds no evidence. Expand to generated schemas, configuration or dependencies when they control the behavior; absence from one source file is not proof that a symbol is absent. Stop gathering when the next implementation or diagnostic decision is supported, retaining references rather than whole-file copies.

## Verification

Match discovered entrypoints and commands to actual manifests and callers.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Lee instrucciones y manifiestos, ubica puntos de entrada y pruebas. Busca por símbolos y conserva rutas y comandos comprobados.
