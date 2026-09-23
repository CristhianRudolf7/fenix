---
name: git-worktree-coordination
description: Isolate concurrent repository changes with Git worktrees and evidence-based integration.
metadata:
  version: 1.0.0
  category: engineering
  tags: git, worktree, coordination
  aliases: worktree, ramas paralelas, aislamiento git, conflictos
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

1. Inspect current status, refs and existing worktrees. Create a separate worktree from the intended base without moving or resetting unrelated work.
2. Assign files/interfaces and isolate ports, build outputs and task data. Do not copy .env or credentials into every worktree by default.
3. Integrate changes using their actual base/head and review conflicts semantically, preserving both tasks’ required behavior.
4. Remove only worktrees whose work is preserved and whose cleanup is authorized; inspect untracked files and unmerged commits first.

## Verification

Run integration checks on the combined result; independent branch success does not prove compatibility.

## Limits

A worktree isolates files, not shared databases, running services or external effects.

## Español

Aísla archivos y recursos por tarea; integra con pruebas y no borres trabajo sin conservarlo.
