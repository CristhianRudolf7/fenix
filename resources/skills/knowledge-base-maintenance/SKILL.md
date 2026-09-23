---
name: knowledge-base-maintenance
description: Organize and retrieve local notes with provenance, stable links and explicit handling of duplicates
  or contradictions.
metadata:
  version: 1.0.0
  category: knowledge
  tags: knowledge, base, maintenance
  aliases: wiki, base conocimiento, notas, enlaces rotos, notas huérfanas
  required_tools: files_search,file_read,file_write,file_edit
  requires: Authorized notes folder and its existing conventions
  effects: local-write
---
# Knowledge Base Maintenance

## Scope

Use for document-based knowledge bases. Use memory-and-reminders for Fénix’s canonical memory database; do not edit its generated exports as the source of truth.

## Inputs

Knowledge root, topic or maintenance goal, scope and retention preferences.

## Workflow

1. Inspect directory indexes and naming/frontmatter conventions. Search titles and aliases first, then relevant passages rather than loading every document.
2. Return answers with file and section references. Preserve timestamps and distinguish current facts from historical notes.
3. When adding notes, link sources and related existing notes; use stable identifiers where available. A similar title does not prove the same entity.
4. For cleanup, identify exact duplicates by content and semantic overlaps by reading. Keep conflicting claims with their sources until evidence resolves them.
5. Repair links after authorized moves and report orphaned or expired candidates. Preserve a change manifest; do not automatically delete historical evidence.

## Verification

Check destination links, retained provenance and unresolved contradictions. Ensure user/project boundaries remain intact.

## Limits

Do not build a graph or embeddings merely because notes exist. No automatic background cleanup or memory promotion is enabled by this skill.

## Español

Consulta índices y fragmentos, conserva procedencia y distingue duplicados de contradicciones. Repara enlaces sin borrar evidencia ni confundir notas con la memoria canónica.
