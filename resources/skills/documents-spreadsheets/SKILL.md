---
name: documents-spreadsheets
description: Create and inspect Excel XLSX workbooks, tables of data and formulas; update individual cells while
  preserving existing workbook parts with native Fenix Documents.
metadata:
  version: 1.2.0
  category: office
  tags: documents, spreadsheets
  aliases: excel, xlsx, hoja de cálculo, celdas, fórmulas
  required_tools: document_read,document_create,document_edit
  requires: Native Fénix document tools
  effects: mixed
---
# Fénix Documents — Spreadsheets

## Scope

Create and inspect Excel XLSX workbooks, tables of data and formulas; update individual cells while preserving existing workbook parts with native Fenix Documents.

## Inputs

Workbook, sheet and cells, value types, formulas and requested local change.

## Workflow

Use native `document_*` tools. No cloud account or office runtime is needed.

Create with `document_create`:
```json
{"path":"budget.xlsx","document":{"format":"xlsx","sheets":[{"name":"Budget","rows":[["Item","Amount"],["Hosting",20],["Support",35],["Total",{"formula":"SUM(B2:B3)"}]]}]}}
```
Cells accept strings, finite numbers, booleans, null, or `{ "formula": "..." }`. Strings remain literal even if they start with `=`; use an explicit formula object only when intended. Rows start at A1. Sheet names must be unique and valid in Excel.

Inspect existing workbooks with `document_read`. It returns worksheet part names, A1 addresses, cell types, formulas, cached values and a whole-file hash. For a cell edit, call `document_edit` with `expected_sha256` and:
```json
{"action":"set_cell","part":"xl/worksheets/sheet1.xml","cell":"B2","value":25}
```
Use the actual returned part, not a guessed mapping between sheet names and numbers. Existing cell styles and untouched ZIP parts are retained. Reread after each mutation before another edit. Missing cells/rows can be created. Editing workbooks with shared/array/data-table formulas is refused rather than risking corruption.

Fénix stores formulas but has **no formula evaluation engine**. Recalculation is requested when a compatible spreadsheet app opens the result. Never present cached values as freshly computed. For numerical analysis, calculate explicitly with an available local computation tool, preserve input evidence, and distinguish computed results from exported formulas.

Read back cells that changed. Pivot creation, charts, formatting APIs, XLS/CSV import and full Excel compatibility are not implemented by these document tools. Never rebuild an imported workbook from extracted cells just to edit one value.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Read back changed cells and distinguish stored formulas from newly calculated results.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Native document tools do not render pages or calculate formulas. Do not install OfficeCLI or a cloud office dependency as an implicit fallback.
