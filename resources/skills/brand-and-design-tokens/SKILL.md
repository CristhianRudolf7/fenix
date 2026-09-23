---
name: brand-and-design-tokens
description: Define or reconcile reusable brand tokens and component states without replacing established identity.
metadata:
  version: 1.0.0
  category: creative
  tags: brand, and, design, tokens
  aliases: marca, design tokens, sistema diseño, colores, tipografía
  required_tools: file_read,file_write,tool_search
  requires: Existing brand assets or an explicit design brief
  effects: local-write
---
# Brand And Design Tokens

## Scope

Define or reconcile reusable brand tokens and component states without replacing established identity.

## Inputs

Audience, platform, visual references, accessibility and reuse needs. Prerequisite: Existing brand assets or an explicit design brief.

## Workflow

1. Inventory actual colors, type, spacing and component states before proposing changes. Reuse existing semantic roles where possible.
2. Separate primitive values from semantic roles and component overrides. Define light/dark behavior and contrast requirements.
3. Create a small representative set of components with loading, focus, disabled and error states; avoid style variants without a use case.
4. Check application consistency and document token intent so future changes do not require scattered value edits.

## Verification

Inspect real rendered components in relevant modes and check text contrast, focus and scaling.

## Limits

Do not impose an upstream author’s aesthetic or copy proprietary brand assets without permission.

## Español

Define roles y estados reutilizables, respeta identidad existente y verifica contraste y modos reales.
