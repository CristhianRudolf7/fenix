---
name: knowledge-workspace-connectors
description: Maintain Notion, Confluence or similar hosted knowledge pages while preserving native structure and
  access.
metadata:
  version: 1.0.0
  category: office
  tags: knowledge, workspace, connectors
  aliases: Notion, Confluence, wiki nube, espacio conocimiento
  required_tools: tool_search
  requires: Authorized workspace connection or exported pages
  effects: external-write
---
# Knowledge Workspace Connectors

## Scope

Maintain Notion, Confluence or similar hosted knowledge pages while preserving native structure and access.

## Inputs

Workspace/space, page IDs, template and requested update. Prerequisite: Authorized workspace connection or exported pages.

## Workflow

1. Discover supported read/write operations and inspect the native block/page structure and revision.
2. Resolve parent pages and database schemas before creation. Preserve properties, relations, attachments and unsupported blocks.
3. Apply section/block-level edits; do not replace a rich page with plain text merely because that is easier to generate.
4. Check links, hierarchy and permissions after mutation. Handle concurrent edits by rereading rather than replaying stale indexes.

## Verification

Confirm updated blocks and unchanged neighboring content; return actual page links.

## Limits

A local Markdown export is not a live workspace update. No hosted connector is installed by this skill.

## Español

Respeta bloques, propiedades y revisiones; verifica páginas reales y no confundas exportar con actualizar la nube.
