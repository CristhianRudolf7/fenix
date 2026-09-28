---
name: documents-presentations
description: Create PowerPoint PPTX decks with titled slides and bullet text; read and revise existing slide text
  with native Fenix Documents.
metadata:
  version: 1.2.0
  category: office
  tags: documents, presentations
  aliases: powerpoint, pptx, diapositivas, presentación
  required_tools: document_read,document_create,document_edit
  requires: Native Fénix document tools
  effects: mixed
---
# Fénix Documents — Presentations

## Scope

Create PowerPoint PPTX decks with titled slides and bullet text; read and revise existing slide text with native Fenix Documents.

## Inputs

Deck path, audience, slide content or exact local text change.

## Workflow

Use native `document_*` tools. Authoring is local and requires no office service or external CLI.

Create with `document_create`:
```json
{"path":"overview.pptx","document":{"format":"pptx","slides":[{"title":"Project overview","bullets":["Objective and scope","Current progress","Next steps"]},{"title":"Decision","bullets":["Approve the pilot"]}]}}
```
The native layout is 16:9, dark text on a light background, a title and a content box. Keep text concise. More than ten bullets per slide is rejected; split long material into additional slides. The engine does not measure visual overflow, so do not claim visual QA from text extraction.

For an existing deck, `document_read` returns text by slide part and paragraph plus a file hash. `document_edit` takes that `expected_sha256` and:
```json
{"action":"replace_text","part":"ppt/slides/slide1.xml","expected":"Current progress","replacement":"Pilot completed"}
```
Choose the actual part from the read result. Exactly one paragraph-local match is required, including text split across runs. Other objects and parts are preserved. Read again when the hash is stale; never recreate an imported deck from its extracted text.

Verify the affected text after editing. This initial engine does not create charts, pictures, transitions, speaker notes or custom layouts, and does not render slides to images/PDF. Report those limitations when relevant instead of substituting invented capabilities.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Read back slide order/text; do not claim a rendered visual review from XML inspection.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Native document tools do not render pages or calculate formulas. Do not install OfficeCLI or a cloud office dependency as an implicit fallback.
