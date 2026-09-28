---
name: web-data-extraction
description: Extract structured records from specified web pages with pagination, schema validation and row-level provenance. Extracción web a CSV o JSON.
metadata:
  version: 1.0.0
  category: data
  tags: extraction, web, records, pagination
  aliases: extraer datos web, scraping, catálogo paginado, anuncios web, scrape records, structured extraction
  required_tools: web_read,tool_search,file_read,file_write,command_run
  requires: Authorized target pages and native web/browser tools or supplied HTML exports
  effects: local-write
---
# Web Data Extraction

## Scope

Use when the output is a dataset extracted from known pages. Use web-research to discover and assess sources; use browser-navigation for interactive forms or account operations.

## Inputs

Target URLs, permitted host/path scope, record fields and types, output format, page/item limit and access context.

## Workflow

1. Define the record key and field meanings before collecting. Keep absent, empty, zero, estimated and inaccessible values distinct. Retain displayed currency, units, locale and source time before normalization.
2. Use Fénix's native reader first for available static content and the native browser when rendering is necessary. For supplied exports, parse locally. Respect the existing public-search/access boundaries; this skill does not replace them with hosted scraping/search APIs or move private browser content into the public index.
3. Validate one representative page and a structurally different page before scaling. Scope selectors to one record container so missing fields cannot shift values between records. JSON-LD may contain unrelated entities; verify the actual page and requested entity.
4. Follow observed pagination links/cursors within scope. Keep visited URLs/cursors and page fingerprints, stop on repeats and honor configured limits and rate responses. An HTTP success containing a login/challenge/empty shell is not a successful extraction. Report blocked pages instead of bypassing access controls.
5. Preserve source URL, retrieval time and a locator or raw fragment for each record. Reconcile duplicates by stable identity; matching names alone are insufficient. Save raw and normalized values separately where conversions matter.
6. Validate the saved CSV/JSON, record count, schema and representative source matches. Keep resumable checkpoints task-local; label partial coverage. For spreadsheet-facing exports, prevent untrusted cell values from becoming formulas while preserving original values separately.

## Verification

Check missing fields, repeated pages, duplicate identities, locale-sensitive numbers and out-of-scope links. A valid JSON schema establishes structure, not factual fidelity or complete coverage.

## Limits

No universal scraper or external provider is installed. Extraction does not authorize purchases, form submissions, contact enrichment or exporting unrelated personal data.
