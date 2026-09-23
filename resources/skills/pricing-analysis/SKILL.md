---
name: pricing-analysis
description: Evaluate pricing models using customer value, unit economics and explicit usage scenarios.
metadata:
  version: 1.0.0
  category: business
  tags: pricing, analysis
  aliases: precios, tarifas, monetización, margen, pricing
  required_tools: file_read,file_write,tool_search
  requires: Product cost/usage evidence and current competitor pricing if relevant
  effects: local-write
---
# Pricing Analysis

## Scope

Evaluate pricing models using customer value, unit economics and explicit usage scenarios.

## Inputs

Customer segments, value metric, costs, constraints and proposed plans. Prerequisite: Product cost/usage evidence and current competitor pricing if relevant.

## Workflow

1. Define the billable unit and what drives delivery cost. Separate fixed, variable and step costs, including heavy-user tails.
2. Compare pricing structures under low, medium and high usage with the same assumptions. Include taxes, discounts and currency conversion only with dated inputs.
3. Use willingness-to-pay evidence as a range, not a fact inferred from competitors. Check incentives and limits that could surprise users.
4. Present sensitivity and break-even conditions; propose tests or further evidence where uncertainty dominates.

## Verification

Recompute scenario arithmetic and ensure margin calculations use the correct denominator.

## Limits

Do not publish new prices or change customer billing as part of analysis.

## Español

Modela unidades y costes por escenarios, revisa márgenes y sensibilidad sin convertir supuestos en certezas.
