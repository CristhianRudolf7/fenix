---
name: cloud-cost-review
description: Explain cloud or API spending using actual usage, billing dimensions and defensible alternatives.
metadata:
  version: 1.0.0
  category: operations
  tags: cloud, cost, review
  aliases: costes nube, factura API, ahorro, billing, gasto
  required_tools: file_read,file_write,command_run
  requires: Billing export or authorized read-only usage access
  effects: mixed
---
# Cloud Cost Review

## Scope

Explain cloud or API spending using actual usage, billing dimensions and defensible alternatives.

## Inputs

Period, currency, accounts, pricing assumptions and desired comparison. Prerequisite: Billing export or authorized read-only usage access.

## Workflow

1. Reconcile totals by service/account/time, distinguishing taxes, credits, commitments and unbilled estimates.
2. Attribute compute, storage, egress and idle capacity using usage evidence. Separate list prices from effective rates and cached/token pricing dimensions.
3. Estimate alternatives under the same workload and include migration, minimum commitment and operational cost.
4. Prioritize changes with expected range and uncertainty; prepare recommendations without shutting down resources or purchasing commitments automatically.

## Verification

Recompute totals from source rows and cite pricing dates; mark missing usage rather than treating it as zero.

## Limits

Do not promise savings from nominal discounts alone or equate a cheaper unit with lower task cost.
