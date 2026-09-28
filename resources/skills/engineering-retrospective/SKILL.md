---
name: engineering-retrospective
description: Review engineering delivery, rework and bottlenecks over a defined period using repository, PR and incident evidence. Retrospectiva de ingeniería.
metadata:
  version: 1.0.0
  category: coordination
  tags: engineering, retrospective, delivery, bottlenecks
  aliases: engineering retro, weekly engineering retrospective, delivery trends, rework analysis, retrospectiva ingeniería, retrospectiva semanal, cuellos de botella equipo
  required_tools: file_read,command_run,tool_search
  requires: Scoped repository history and available pull request, deployment or incident records
  effects: read-only
---
# Engineering Retrospective

## Scope

Explain delivery outcomes and repeated engineering friction during a period. Use project-status-review for current commitments; process-improvement for redesigning a business process; agent-workflow-diagnosis for one agent execution. A retrospective does not create issues, assign people or export team data automatically.

## Inputs

Repository/project, start/end and timezone, question to answer and available records. State the chosen interval explicitly; use non-overlapping equal intervals for comparisons.

## Workflow

1. Resolve the repository and analyzed ref. Report shallow history, inaccessible remotes, missing PR records and freshness limits. An inactive repository can legitimately have zero changes; an old latest commit is not by itself proof of stale history.
2. Identify delivered user outcomes from changes and verified release records. Keep committed, merged and deployed separate. Exclude generated/vendor churn from source-change analysis when appropriate and record that exclusion.
3. Examine evidence of review delays, build failures, reverted changes, escaped defects and repeated touches to the same area. Use actual timestamps to measure intervals. State definitions and denominators: a changed test file is not a new test, a commit is not a PR, and an unlinked fix is not automatically an incident.
4. Account for merge/squash history, bots, coauthors and identity aliases before aggregating counts. Do not infer hours worked, effort, employee performance or causal productivity from commit gaps, line counts or a contributor leaderboard. Note invisible review, support and design work where evidence is missing.
5. Compare equivalent windows and data coverage, reporting sample sizes and unavailable metrics. An empty denominator is undefined, not zero percent. Correlation between churn and failures is a lead for investigation, not proof of root cause.
6. Present the supported outcomes, repeated friction, counterevidence and a small improvement experiment with an observable success criterion. Preserve effective practices and accepted tradeoffs. Store a project-local report only if requested; never publish or send it as part of an informational retrospective.

## Verification

Every substantial claim links to changes, PRs, deployments or incident records within the declared period. Distinguish measured metrics from interpretation and unknowns. A follow-up experiment should address the evidence without judging people from incomplete telemetry.

## Limits

Stay within authorized repositories and team data. Do not scan neighboring workspaces, invent delivery counts from commit subjects, or treat an automatically generated narrative as a durable fact about an individual.
