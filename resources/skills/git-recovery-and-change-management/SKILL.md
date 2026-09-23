---
name: git-recovery-and-change-management
description: Inspect Git history and changes, recover content and preserve existing work. Git historial recuperación
  cambios.
metadata:
  version: 1.2.0
  category: engineering
  tags: git, recovery, and, change, management
  aliases: git, historial, recuperar archivo, commit, rama
  required_tools: command_run,file_read,files_search
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Git Recovery And Change Management

## Scope

Inspect Git history and changes, recover content and preserve existing work. Git historial recuperación cambios.

## Inputs

Repository, current changes, relevant references and authorized operation.

## Workflow

Inspect status and diff before making changes. Distinguish tracked edits, untracked files, staged work and branch history. Determine whether the request concerns current files, historical objects or both: editing the working tree does not change earlier commits. Do not expand a current-file change into a history rewrite without authorization.

Recover required content from explicit commits, reflogs or object references without overwriting unrelated work. When a history transformation is authorized, inspect the relevant refs and object structure, preserve a recoverable reference in the authorized environment, and use Git-aware operations for commits and trees. Reapplying working-tree patches cannot repair arbitrary historical objects. Check the resulting refs and retained content, as well as any removal requirements explicitly stated by the user. Preserve repository validity and requested history properties.

Review resulting diffs and verify the requested files and behavior. Do not publish recovered secrets, raw credentials, private traces or backup references that retain data the user requested removed. Commit only intended changes with a clear English message and report the actual branch and commit. Destructive reset or cleanup is not a routine recovery method.

## Verification

Inspect the final diff, refs and retained content; preserve unrelated work.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Revisa estado y diferencias, recupera contenido sin sobrescribir trabajo ajeno y verifica el resultado. Distingue archivos actuales de objetos históricos: editar un archivo no modifica commits anteriores. Transforma el historial solo dentro del alcance autorizado, verifica referencias y contenido y evita publicar respaldos con datos que debían eliminarse. Publica solo cambios previstos.
