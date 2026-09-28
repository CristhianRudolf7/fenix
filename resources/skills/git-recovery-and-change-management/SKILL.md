---
name: git-recovery-and-change-management
description: Inspect Git history and changes, recover content and preserve existing work. Git historial recuperación
  cambios.
metadata:
  version: 1.4.0
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

Before an authorized commit or merge, use the existing repository/account Git identity. If Git reports an unknown identity, inspect `git config --show-origin --get user.name`, `git config --show-origin --get user.email` and the effective HOME before asking: a changed execution home can hide an already configured identity. Reuse only the authorized account configuration; never borrow a commit author as the current user, invent identity, read another tenant home or change global configuration just to bypass the error. Ask only if the authorized identity remains unavailable.

Review resulting diffs and verify the requested files and behavior. Do not publish recovered secrets, raw credentials, private traces or backup references that retain data the user requested removed. Commit only intended changes with a clear English message and report the actual branch and commit. Destructive reset or cleanup is not a routine recovery method.

## Deployment verification

For an authorized push-to-deploy task, verify the complete transition: create unique new test content in an isolated clone/fixture, commit with an authorized identity, require a successful push exit status, and compare the served content with that new commit. Preexisting matching HTTP content does not prove that a rejected push deployed anything. Keep branch deployment directories independent so publishing one cannot erase another. Do not force push or overwrite user content to make a probe succeed. Remove only your temporary fixtures when appropriate, preserving the requested running service and deployment.

## Verification

Inspect the final diff, refs and retained content; preserve unrelated work.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.
