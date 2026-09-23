---
name: web-presentations
description: Build browser-based slide presentations with readable layouts, keyboard navigation and export boundaries.
metadata:
  version: 1.1.0
  category: creative
  tags: web, presentations
  aliases: presentación HTML, slides web, diapositivas navegador
  required_tools: file_read,file_write,tool_search
  requires: Existing web stack or static HTML/CSS/JS environment
  effects: local-write
---
# Web Presentations

## Scope

Build browser-based slide presentations with readable layouts, keyboard navigation and export boundaries.

## Inputs

Audience, story, assets, slide count constraints and delivery format. Prerequisite: Existing web stack or static HTML/CSS/JS environment.

## Workflow

1. Outline one message per slide and map evidence/visuals to the story. Preserve supplied brand and content requirements.
2. Use the existing stack or a simple static artifact; implement navigation, focus and responsive scaling with readable text.
3. Keep content accessible outside animation and respect reduced motion. Avoid putting essential meaning only in timed transitions.
4. Run the presentation in a browser and inspect overflow, navigation, media loading and any print/export path requested.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Check first/last slides, keyboard controls, offline assumptions and the actual exported artifact if generated.

## Limits

An HTML deck is not automatically a PowerPoint file; do not claim native PPTX compatibility.

## Español

Crea slides legibles con teclado y movimiento reducido; verifica el formato entregado y sus límites.
