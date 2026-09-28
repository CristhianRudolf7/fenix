---
name: latex-authoring
description: Create and repair LaTeX documents with reproducible builds and inspected PDF output.
metadata:
  version: 1.1.0
  category: research
  tags: latex, authoring
  aliases: latex, tex, PDF académico, ecuaciones, compilación
  required_tools: file_read,file_write,tool_search
  requires: Installed TeX toolchain and the requested class/template
  effects: local-write
---
# Latex Authoring

## Scope

Create and repair LaTeX documents with reproducible builds and inspected PDF output.

## Inputs

Source, document type, bibliography, figures and output requirements. Prerequisite: Installed TeX toolchain and the requested class/template.

## Workflow

1. Inspect the existing class, engine and build command before changing packages. Preserve template structure and content.
2. Escape user text according to its context; keep math, code and URLs distinct. Use stable labels and cross-references.
3. Build through the project command and read the first actionable error plus warnings; do not repeatedly change packages without diagnosing the cause.
4. Inspect the PDF for overflow, missing glyphs, clipped figures and unresolved references, not only successful compilation.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Rebuild from declared inputs and verify pages, figures, bibliography and expected sections.

## Limits

Do not enable shell escape for untrusted documents or fabricate a PDF review when no renderer was available.
