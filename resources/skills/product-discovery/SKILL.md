---
name: product-discovery
description: Evaluate product opportunities and assumptions before committing to implementation.
metadata:
  version: 1.1.0
  category: business
  tags: product, discovery
  aliases: descubrimiento producto, oportunidad, validar idea, hipótesis, product demand, current workaround, demanda, descartar hipótesis
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

Match the decision to the project: commercial demand, internal adoption, research or learning. Describe the user's current workaround and its evidenced cost; distinguish stated interest, actual use, retention and payment without reducing every project's value to revenue. Compare a small intervention, the proposed solution and continuing the current approach. Define which result would change or stop the proposal. Reuse supplied evidence and ask only about gaps that materially change the decision; a discovery procedure need not become a scripted interrogation.

## Verification

Check that evidence reflects the target users and that the proposed experiment can falsify the assumption.

## Limits

Do not invent demand from enthusiasm, stakeholder opinion or a small convenience sample.
