---
name: repository-navigation
description: Orient in unfamiliar repositories; discover entrypoints, instructions, build and test commands. Repositorio, estructura, pruebas.
version: 1.0.0
required_tools: files_glob,files_search,file_read,command_run
---

# Repository Navigation

## Procedure

Read applicable repository instructions. Identify manifests, entrypoints and existing test commands before changing code. Search symbols narrowly; read bounded relevant sections instead of repeatedly listing the entire tree. Record relevant paths and the exact commands needed to reproduce and verify. Use configured code_query when available; absence of a language server is not a blocker. Do not assume a framework from a directory name. Verify the discovered command against the manifest and working directory.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Lee instrucciones y manifiestos, ubica puntos de entrada y pruebas. Busca por símbolos y conserva rutas y comandos comprobados.
