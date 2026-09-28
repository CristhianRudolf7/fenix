---
name: scientific-figures
description: Produce reproducible scientific plots with truthful scales, uncertainty and accessible labels.
metadata:
  version: 1.1.0
  category: research
  tags: scientific, figures
  aliases: figura científica, gráfica, matplotlib, plot, incertidumbre
  required_tools: file_read,file_write,tool_search
  requires: Source data and available plotting/rendering tools
  effects: local-write
---
# Scientific Figures

## Scope

Produce reproducible scientific plots with truthful scales, uncertainty and accessible labels.

## Inputs

Question, variables, units, uncertainty and publication output format. Prerequisite: Source data and available plotting/rendering tools.

## Workflow

1. Choose the encoding from the comparison: distribution, relationship, trend or effect. Preserve sample sizes and data transformations.
2. Use honest axes and comparable scales; distinguish log transforms, normalized values and missing observations.
3. Display uncertainty with its actual definition and avoid visual significance claims unsupported by analysis.
4. Generate from reproducible code and export the requested vector/raster format with readable labels at final size.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Compare plotted values against source data and inspect the rendered artifact, legend and color accessibility.

## Limits

Do not generate illustrative imagery as if it were experimental data or redraw a scientific result without provenance.
