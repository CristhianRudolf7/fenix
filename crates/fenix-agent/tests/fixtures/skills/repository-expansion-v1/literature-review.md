---
name: literature-review
description: Find and compare scholarly papers, verify citations and synthesize methods, evidence and research gaps.
metadata:
  version: 1.0.0
  category: research
  tags: literature, review
  aliases: papers, arxiv, revisión bibliográfica, estado del arte, citas
  required_tools: web_search,web_read,file_write
  requires: Academic sources accessible through configured web tools or supplied papers
  effects: local-write
---
# Literature Review

## Scope

Use for research literature. A small exploratory search is not a systematic review; name the coverage honestly.

## Inputs

Research question, allowed years, domains, inclusion criteria and intended output.

## Workflow

1. Define inclusion/exclusion rules and a search log before screening. Search complementary academic sources where relevant and retain exact queries and dates.
2. Deduplicate by DOI, arXiv ID/version and title. Distinguish preprints from peer-reviewed versions; prefer the version that supports the claim being cited.
3. Read methods and results, not only abstracts. Extract task, sample, baseline, metric, conditions, limitations and what evidence is unavailable.
4. Compare comparable experiments; differences in models, datasets or budgets can invalidate a ranking. Preserve negative results and contradictory findings.
5. Write a thematic synthesis and a claim-to-paper map. Verify bibliographic fields against primary records and mark abstract-only access.

## References

Load [the evidence worksheet](references/evidence-table.md) for multi-paper comparisons or conflicting results.

## Verification

Every citation must resolve to the actual work and support its attached claim. Report screening counts and exclusions only if recorded.

## Limits

Do not invent DOI values or turn reported benchmark scores into independent reproductions. A review does not require image generation or a paid research service.

## Español

Define criterios y registra búsquedas. Compara métodos y condiciones, verifica referencias y aclara qué textos completos pudiste leer.
