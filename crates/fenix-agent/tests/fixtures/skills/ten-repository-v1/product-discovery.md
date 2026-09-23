---
name: product-discovery
description: Evaluate product opportunities and assumptions before committing to implementation.
metadata:
  version: 1.0.0
  category: business
  tags: product, discovery
  aliases: descubrimiento producto, oportunidad, validar idea, hipótesis
  required_tools: file_read,file_write,tool_search
  requires: User/customer evidence and business context
  effects: local-write
---
# Product Discovery

## Scope

Evaluate product opportunities and assumptions before committing to implementation.

## Inputs

Problem, affected users, current alternatives and decision to make. Prerequisite: User/customer evidence and business context.

## Workflow

1. Separate observed problems from proposed solutions. Map each claim to interviews, usage evidence or an explicit assumption.
2. Identify the riskiest assumptions about value, usability, feasibility and viability; do not treat all assumptions as equally important.
3. Choose the smallest experiment that can change the decision, with a success/failure criterion stated in advance.
4. Synthesize evidence, contradictions and next decisions; keep unvalidated ideas out of committed requirements.

## Verification

Check that evidence reflects the target users and that the proposed experiment can falsify the assumption.

## Limits

Do not invent demand from enthusiasm, stakeholder opinion or a small convenience sample.

## Español

Separa problemas y soluciones, identifica supuestos críticos y diseña pruebas capaces de cambiar la decisión.
