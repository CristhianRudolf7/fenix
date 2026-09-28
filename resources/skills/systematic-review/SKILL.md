---
name: systematic-review
description: Conduct a protocol-based systematic literature review with reproducible search, screening records, exclusion reasons and optional meta-analysis. Revisión sistemática y PRISMA.
metadata:
  version: 1.0.0
  category: research
  tags: systematic, review, screening, prisma, meta-analysis
  aliases: revisión sistemática, metaanálisis, PRISMA, criterios de inclusión, cribado, screening ledger
  required_tools: web_search,web_read,file_read,file_write,command_run
  requires: Research question, accessible sources and a defined review protocol
  effects: local-write
---
# Systematic Review

## Scope

Use for a reproducible review with an explicit search and selection protocol. Use literature-review for exploratory synthesis. A short search cannot establish exhaustive coverage or PRISMA compliance.

## Inputs

Question, population or domain, eligible designs, dates, languages, databases, existing protocol and supplied corpus. Retain the owner's protocol; record amendments instead of silently changing eligibility after seeing results.

## Workflow

1. Define search sources, exact queries, date bounds and inclusion/exclusion rules before screening. Identify unavailable databases and restricted full texts. Use supplied literature first when its provenance and selection rules are known; search to address remaining coverage.
2. Save each retrieved record and search origin. Deduplicate by work identity while preserving reports, revisions and repeated database hits. Related publications can report the same study; record that relationship rather than count them as independent experiments.
3. Record title/abstract and full-text decisions, reasons and unresolved records. Distinguish not retrieved from excluded. Do not invent a second reviewer, adjudication or registration that did not occur.
4. Extract outcome definition, sample, intervention or setting, comparator, uncertainty, design limitations and conflicts. Assess bias with the method appropriate to the study design, consulting its current authoritative guidance.
5. Pool only compatible outcomes and designs with an explicit effect measure, dependence handling and missing-data policy. Otherwise use structured narrative synthesis. Reconcile screening totals and discuss coverage and uncertainty before delivery.

## References

Use [screening records](references/screening-records.md) and `scripts/screening_counts.py` to calculate counts from recorded decisions. For claim-to-source checking, load research-evidence-audit. For statistical computation, use the available analysis tool; this skill does not install a meta-analysis package.

## Verification

Recompute counts from record IDs, inspect exclusions and duplicate links, and trace conclusions to included studies. State which review stages and reviewer roles actually occurred. Pending records mean screening is incomplete.

## Limits

Search coverage, source quality and review conduct remain substantive judgments. A valid ledger proves neither exhaustive retrieval nor unbiased screening. Do not manufacture PRISMA counts, registration IDs or confidence ratings.
