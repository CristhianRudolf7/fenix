---
name: code-health-review
description: Report repository check health from its configured type checker, linter and tests, with explicit failures, missing coverage and comparable history. Informe de salud del código.
metadata:
  version: 1.0.0
  category: engineering
  tags: repository, health, checks, quality
  aliases: code health dashboard, repository health, lint and test status, run configured checks, salud del código, estado de checks, informe calidad código
  required_tools: file_read,files_search,command_run,command_status
  requires: Repository check configuration and an authorized environment to run project code
  effects: mixed
---
# Code Health Review

## Scope

Report the outcome of a repository's configured checks. Use code-review for defects in a diff, performance-diagnosis for slowness and completion-verification to accept a specific delivery. Do not modify product code, auto-fix lint, update dependencies or alter CI during a report-only request. Checks can still create caches and reports.

## Inputs

Repository/revision, included working changes, requested checks and execution constraints. Read repository instructions, manifests and CI commands before choosing the checks.

## Workflow

1. Identify the actual configured commands and their scope. Check local availability and side effects, including lifecycle hooks, snapshot updates, remote writes and required secrets. Use supported check-only modes. Do not install a new checker solely to fill a category.
2. Run authorized independent checks; keep checks sharing files, caches, ports or databases sequential or isolate them. Record command, directory, revision/dirty state, environment, start/end, actual process exit status and a private full-output artifact. Retrieve completion for commands that yield a running session.
3. Assign per-check states: passed, failed, unavailable before execution, execution/capture error, cancelled or still running. An executed command returning 127 is a failure to investigate, not a successful skip. A parser with no matches cannot turn nonzero exit into passed; a log tail's exit status is not the checker's status.
4. Prefer machine-readable runner results for counts. Label counts unknown if parsing is ambiguous; do not infer tests executed from test files or assertion lines. Redact credentials before presenting excerpts and keep sensitive logs in the current user's workspace.
5. Compare prior results only for equivalent command sets, versions, scopes and environments. Changed coverage is a separate event. Keep old failures visible; use a disposable baseline only if it is needed to establish whether the change introduced a failure.

## Verification

Return a table of check, scope, status, duration and evidence, plus blockers and checks not run. No checks executed means not assessed. Mixed results remain mixed; passing tests do not erase a failed type check. Use factual counts instead of inventing a weighted quality score or claiming the entire application is correct.

## Limits

Test and build commands execute repository code and remain subject to Fénix isolation and task permissions. A report does not authorize executing hostile code on the host, sending logs externally or repairing all detected issues. An unavailable environment limits coverage; it does not justify weakening the check.
