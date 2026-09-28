---
name: financial-modeling
description: Build transparent budgets, cash-flow forecasts and financial scenarios with auditable assumptions.
metadata:
  version: 1.0.0
  category: business
  tags: financial, modeling
  aliases: presupuesto, flujo caja, proyección financiera, runway
  required_tools: file_read,file_write,tool_search
  requires: Supplied records and local computation/spreadsheet tools
  effects: local-write
---
# Financial Modeling

## Scope

Build transparent budgets, cash-flow forecasts and financial scenarios with auditable assumptions.

## Inputs

Period, currency, cash/accrual basis, assumptions and decision. Prerequisite: Supplied records and local computation/spreadsheet tools.

## Workflow

1. Separate actuals from forecasts and cash from recognized revenue/expense. Keep assumptions in explicit inputs with source/date.
2. Model drivers such as units, price, churn, payment delay and staffing rather than arbitrary growth curves.
3. Reconcile opening balances, inflows, outflows and closing balances; avoid mixing currencies without an explicit conversion basis.
4. Provide base/upside/downside or sensitivity scenarios derived from meaningful uncertainty, with formulas and unresolved data gaps.

## Verification

Check arithmetic, timing and balance consistency; compare a few rows independently from the model.

## Limits

Do not present projections as guarantees or infer investment suitability. Native XLSX export stores formulas but does not calculate them.
