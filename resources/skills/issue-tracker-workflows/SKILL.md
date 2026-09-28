---
name: issue-tracker-workflows
description: Read and update issues, boards and project trackers without duplicate tickets or false status changes.
metadata:
  version: 1.0.0
  category: office
  tags: issue, tracker, workflows
  aliases: Jira, Linear, GitHub issues, Trello, Asana, ClickUp, tickets
  required_tools: tool_search
  requires: Authorized tracker connector/CLI or exported issue data
  effects: external-write
---
# Issue Tracker Workflows

## Scope

Read and update issues, boards and project trackers without duplicate tickets or false status changes.

## Inputs

Workspace/project, issue IDs, requested fields and workflow state. Prerequisite: Authorized tracker connector/CLI or exported issue data.

## Workflow

1. Discover actual tool schemas and permitted transitions. Resolve project and issue IDs, not only titles.
2. Search for existing work before creating a ticket. Preserve original requirements, links, dependencies and assignees.
3. Apply narrow field updates with current-state checks. A status named Done may require evidence or workflow conditions.
4. After partial failure, reread affected issues before retrying; return stable issue URLs and unresolved updates.

## Verification

Confirm fields, relations, comments and status actually changed as requested.

## Limits

This skill does not install Jira/Linear/GitHub tools or grant permission to notify others. Do not close issues from a summary alone.
