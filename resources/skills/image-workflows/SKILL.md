---
name: image-workflows
description: Create, edit or prepare images using available generation and local processing tools with clear asset
  provenance.
metadata:
  version: 1.2.0
  category: creative
  tags: image, workflows
  aliases: imagen, ilustración, editar foto, recortar, comprimir
  required_tools: file_read,file_write,tool_search
  requires: Available image tool or local image-processing software
  effects: local-write
---
# Image Workflows

## Scope

Create, edit or prepare images using available generation and local processing tools with clear asset provenance.

## Inputs

Source image if editing, purpose, dimensions, style and export requirements. Prerequisite: Available image tool or local image-processing software.

## Workflow

1. Inspect supplied images before editing. Distinguish illustration generation from deterministic crop/resize/compression and use the appropriate capability.
2. Preserve the requested subject and edits; do not add unrelated branding or replace a user-provided asset by default.
3. For production assets, check orientation, transparency, color profile and output dimensions. Keep an original when lossy processing matters.
4. Inspect the exported result and communicate any unverified text or visual detail. Record generated versus sourced provenance.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

For geometry-to-text tasks, first validate coordinates and relevant layers, then create a readable rendering or use available authorized OCR/vision tools. Check orientation, scale and cropping against the full geometry. If a text-only projection is unreadable, improve the representation; do not substitute an object filename or label for the requested visual content.

## Verification

Check final dimensions, format, file size and visible artifacts in the actual output.

## Limits

This skill does not install an image-generation provider. Do not fabricate a generated image or claim pixel-exact edits without inspection.
