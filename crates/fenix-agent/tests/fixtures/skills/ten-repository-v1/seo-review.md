---
name: seo-review
description: Audit website discoverability and content structure with evidence from pages, metadata and available
  search data.
metadata:
  version: 1.0.0
  category: writing
  tags: seo, review
  aliases: SEO, buscadores, sitemap, indexación, robots, metadatos
  required_tools: web_read,tool_search,file_read
  requires: Authorized site or local source; browser/search tools as available
  effects: read-only
---
# Seo Review

## Scope

Use for a focused SEO audit or correction. Do not promise rankings; search engines control indexing and placement.

## Inputs

Site, target audience/locales, affected pages, symptom and available analytics.

## Workflow

1. Inspect representative pages, response status, canonical URLs, robots rules and sitemap entries. Distinguish blocked crawling from noindex and canonicalization.
2. Check titles, headings, internal links and content usefulness. Identify duplicate or orphaned pages with evidence rather than a generic checklist score.
3. For client-rendered content, compare source HTML with the rendered DOM before declaring metadata or structured data absent.
4. Validate structured data against the actual visible content. Review alternate-language links, redirects and mobile rendering when relevant.
5. Prioritize fixes by affected user paths and observable defects; separate technical issues from hypotheses about ranking.

## Verification

Recheck changed pages and links. Label lab performance data separately from field metrics, and report whether search-console access was available.

## Limits

Do not scrape private analytics, mass-generate thin pages or claim that implementing suggestions guarantees traffic.

## Español

Revisa rastreo, metadatos y contenido con evidencia. Distingue HTML estático de DOM renderizado y evita promesas de posiciones o tráfico.
