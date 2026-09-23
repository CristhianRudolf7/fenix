---
name: roadmap-prioritization
description: Prioritize initiatives using explicit outcomes, dependencies, effort and uncertainty.
metadata:
  version: 1.0.0
  category: business
  tags: roadmap, prioritization
  aliases: roadmap, priorizar, RICE, backlog, estrategia producto
  required_tools: file_read,file_write,tool_search
  requires: Candidate initiatives and actual constraints
  effects: local-write
---
# Roadmap Prioritization

## Scope

Prioritize initiatives using explicit outcomes, dependencies, effort and uncertainty.

## Inputs

Desired outcomes, evidence, capacity, dependencies and time horizon. Prerequisite: Candidate initiatives and actual constraints.

## Workflow

1. Group duplicate requests by underlying outcome while preserving distinct stakeholders and constraints.
2. Compare impact, evidence strength, effort and risk using a consistent scale; do not turn unknown estimates into precise scores.
3. Identify dependencies and opportunity costs. Separate committed work from options that need discovery.
4. Publish a rationale and review triggers rather than fixed dates unsupported by capacity. Translate accepted initiatives into the existing planning system.

## Verification

Check that top priorities serve agreed outcomes and fit actual capacity; expose ranking sensitivity to uncertain assumptions.

## Limits

A prioritization framework is optional. Do not promise delivery dates or reject explicit user priorities because a score says otherwise.

## Español

Agrupa necesidades, explicita incertidumbre y dependencias y distingue compromisos de opciones por validar.
