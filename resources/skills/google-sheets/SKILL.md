---
name: google-sheets
description: Analyze and update Google Sheets with explicit ranges, typed values and formula preservation.
metadata:
  version: 1.1.0
  category: office
  tags: google, sheets
  aliases: google sheets, hojas google, rango, tabla compartida
  required_tools: tool_search
  requires: Authorized Google Sheets connection
  effects: external-write
---
# Google Sheets

## Scope

Use for remote spreadsheets. Local XLSX files belong to documents-spreadsheets; statistical interpretation belongs to data-analysis.

## Inputs

Spreadsheet ID, tab, ranges, locale, meaning of columns and requested change.

## Workflow

1. Discover schemas and read spreadsheet metadata, tab IDs, headers and the relevant ranges. Resolve duplicate tab names or ambiguous columns before writing.
2. Distinguish formatted text, raw values and formulas. Record the locale, timezone, decimal separators and date interpretation. Preserve identifiers with leading zeros.
3. Calculate the exact rectangular update. Use raw input for user text; use formula interpretation only for intended formulas. Do not overwrite adjacent formulas with cached values.
4. Before appending, search a stable business key to avoid duplicate rows. For batch updates, preserve request-to-response mapping and inspect partial failures.
5. Reread the updated range; if other users changed it meanwhile, reconcile the current values rather than retrying an old whole-sheet write.

## Native Fénix connection

Use `connector_call` to list/select the account, then `workspace_schema` for service `sheets` and the exact read/update method. Execute through `request` with API-relative paths and separate query/body objects. Preview changes with `dry_run=true` when useful; this is local validation, not evidence of server acceptance. Confirm access by reading the actual spreadsheet and its metadata. Scope selection or a successful Drive probe alone does not prove Sheets API availability.

## Verification

Check cell types, row counts, totals, formulas and neighboring ranges. Report formula errors and whether values were recalculated by the service.

## Limits

Do not infer that an empty formatted cell is an empty raw value. Without a connection, offer an exported workbook or CSV workflow.
