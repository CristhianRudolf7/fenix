---
name: diagram-authoring
description: Create accurate flow, sequence, architecture or entity diagrams from concrete system evidence.
metadata:
  version: 1.2.0
  category: creative
  tags: diagram, authoring
  aliases: diagrama, flujo, Mermaid, secuencia, arquitectura visual
  required_tools: file_read,file_write,tool_search
  requires: Available text/vector diagram tools and source facts
  effects: local-write
---
# Diagram Authoring

## Scope

Create accurate flow, sequence, architecture or entity diagrams from concrete system evidence.

## Inputs

Question, audience, entities, relationships and output format. Prerequisite: Available text/vector diagram tools and source facts.

## Workflow

1. Choose a diagram type suited to the relationship: sequence for time, flow for decisions, entity model for data, architecture for boundaries.
2. Extract actual nodes and edges from the source; label inferred or proposed components instead of presenting them as implemented.
3. Keep direction, naming, units and boundary semantics consistent. Split an unreadable diagram by concern rather than shrinking labels.
4. Render with the available tool and inspect layout, clipped labels and ambiguity; deliver editable source plus requested export.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Fénix diagram workflow

Users navigate diagrams but edit them only through chat. Use `design_publish` to create a diagram, `design_read` before editing, and `diagram_edit` for focused semantic changes. Read [the authoring contract](references/diagrams.md) for the graph schema and examples. Prefer a structured graph for architecture, processes and concept maps; use Mermaid source for sequences, state machines, entity relationships and other supported Mermaid diagrams.

The engine archives each published revision as JSON and Mermaid in the current user's `data_dir/diagrams/<design_id>/` directory. Tool receipts include the exact paths. Edit using the tools, not those snapshot files; direct filesystem edits do not update the published design. Keep original IDs and unrelated details. Preserve factual labels and distinguish proposals from observed behavior. Restore by reading the requested historical revision, then publishing its content against the current revision number.

## Verification

Trace at least one real scenario through the diagram and verify every material relationship against evidence.

## Limits

Do not generate a repository knowledge graph or invent service dependencies merely to fill a diagram.
