---
name: contract-review
description: Summarize and compare contract terms, obligations and ambiguities for an informed review.
metadata:
  version: 1.0.0
  category: business
  tags: contract, review
  aliases: contrato, NDA, acuerdo, cláusula, SOW
  required_tools: file_read,file_write,tool_search
  requires: Supplied contract versions and current authoritative legal sources when needed
  effects: local-write
---
# Contract Review

## Scope

Summarize and compare contract terms, obligations and ambiguities for an informed review.

## Inputs

Parties, document purpose, jurisdiction if known and review questions. Prerequisite: Supplied contract versions and current authoritative legal sources when needed.

## Workflow

1. Identify the operative version and preserve clause numbering. Extract parties, scope, payment, term, termination, liability, IP and governing-law terms actually present.
2. Map obligations to owners, deadlines and triggering events; distinguish unconditional duties from contingent ones.
3. Compare proposed revisions against the baseline, including definitions and cross-references that change meaning elsewhere.
4. Explain ambiguous or commercially important terms in plain language and mark legal conclusions requiring current jurisdiction-specific verification.

## Verification

Tie each finding to clause/page evidence and check dates, totals and defined terms.

## Limits

Do not invent missing clauses, certify enforceability or sign/accept terms. Preserve legal meaning rather than simplifying away exceptions.

## Español

Resume obligaciones y cambios con cláusulas concretas; conserva excepciones y verifica conclusiones jurídicas según jurisdicción.
