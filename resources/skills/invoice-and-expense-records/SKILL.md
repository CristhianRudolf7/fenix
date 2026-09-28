---
name: invoice-and-expense-records
description: Extract invoice and receipt fields, organize expense evidence and reconcile amounts without making
  payments.
metadata:
  version: 1.1.0
  category: office
  tags: invoice, and, expense, records
  aliases: factura, recibo, gastos, comprobante, reembolso, conciliación bancaria, movimientos bancarios, bank statement, receipts reconciliation
  required_tools: file_read,file_write,document_read,command_run
  requires: Readable documents; OCR or PDF tools if source files require them
  effects: local-write
---
# Invoice And Expense Records

## Scope

Use for bookkeeping preparation and expense records. This is not tax advice or payment execution.

## Inputs

Source files, date range, reporting currency and requested output columns.

## Workflow

1. Inventory sources and preserve originals. Extract vendor, invoice number, issue/due dates, currency, subtotal, tax and total with page or cell references.
2. Mark unreadable or absent fields instead of guessing. Distinguish invoice date, payment date and due date; a receipt may be paid while an invoice may not.
3. Check arithmetic using decimal amounts. Keep currencies separate; convert only with an explicit rate and dated source, preserving original values.
4. Detect candidate duplicates using vendor, invoice number, amount and file hash. Similar totals alone do not establish duplication; do not delete originals.
5. Produce a reconciled table, exception list and links to evidence. If organizing files, use collision-safe names and retain an old-to-new path manifest.

## Verification

For statement reconciliation or payment matching, read [statement records](references/statement-records.md). Use `scripts/reconcile_statement.py` with explicitly normalized decimal strings to check account/currency balances. It does not categorize expenses, match invoices automatically or verify source authenticity.

Reconcile source count, accepted rows, duplicates and unresolved entries. Totals must include only the stated currency and period.

## Limits

Do not infer tax deductibility or pay a bill. Scans need an available OCR capability; extraction confidence is not proof of authenticity.
