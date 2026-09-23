---
name: git-worktree-coordination
description: Isolate concurrent repository changes with Git worktrees and evidence-based integration.
metadata:
  version: 1.1.0
  category: engineering
  tags: git, worktree, coordination
  aliases: worktree, ramas paralelas, aislamiento git, conflictos, finalizar rama, branch handoff
  required_tools: file_read,file_write,command_run
  requires: Git repository and available disk within the workspace
  effects: mixed
---
# Git Worktree Coordination

## Scope

Isolate concurrent repository changes with Git worktrees and evidence-based integration.

## Inputs

Base ref, task ownership, branch names and integration target. Prerequisite: Git repository and available disk within the workspace.

## Workflow

1. Inspect current status, refs and existing worktrees. Reuse an already isolated workspace and distinguish a linked worktree from a submodule before creating another. When isolation is needed, create a separate worktree from the intended base without moving or resetting unrelated work. Verify the chosen project-local directory is ignored; a directory name alone proves neither isolation nor ownership.
2. Assign files/interfaces and isolate ports, build outputs and task data. Do not copy .env or credentials into every worktree by default.
   Run proportionate baseline checks and record existing failures before editing so later failures can be attributed correctly.
3. Integrate changes using their actual base/head and review conflicts semantically, preserving both tasks’ required behavior.
4. At branch handoff, record the actual base/head, dirty files, checks and integration state. Follow an already authorized merge or PR request; otherwise leave the work ready and preserved. Detached HEAD does not imply permission to publish or create remote branches.
5. Remove only worktrees whose ownership is established, work is preserved and cleanup is authorized; inspect untracked files and unmerged commits first. Never infer ownership from a .worktrees directory name or force removal after Git refuses it.

## Verification

Run integration checks on the combined result; independent branch success does not prove compatibility.

## Limits

A worktree isolates files, not shared databases, running services or external effects.

## Español

Aísla archivos y recursos por tarea; integra con pruebas y no borres trabajo sin conservarlo.

Reutiliza el aislamiento existente, registra fallos previos y verifica el directorio ignorado. Al finalizar la rama, informa base, cambios y comprobaciones; conserva el worktree si no se autorizó su limpieza y no deduzcas propiedad por su nombre.
