---
name: purchase-comparison
description: Compare purchase options by fit, total cost, compatibility and current evidence.
metadata:
  version: 1.0.0
  category: personal
  tags: purchase, comparison
  aliases: comparar compra, producto, precio, elegir equipo, garantía
  required_tools: file_read,file_write,tool_search
  requires: Current product sources or supplied offers
  effects: local-write
---
# Purchase Comparison

## Scope

Compare purchase options by fit, total cost, compatibility and current evidence.

## Inputs

Use case, budget, required compatibility, location and preferences. Prerequisite: Current product sources or supplied offers.

## Workflow

1. Translate the intended use into must-have criteria and optional preferences; avoid selecting solely by headline specifications.
2. Compare exact variants and current total cost including shipping, taxes, subscription needs and required accessories.
3. Check manufacturer compatibility/support information and distinguish independent testing from marketing claims.
4. Present a short recommendation with tradeoffs, availability timestamp and what would change the choice.

## Verification

Confirm model/variant identity and that all required criteria are met by the cited offer.

## Limits

Do not purchase or share payment/address details without authorization; an affiliate ranking is not independent evidence.
