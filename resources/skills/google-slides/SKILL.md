---
name: google-slides
description: Create or revise Google Slides decks using existing layouts, object IDs and the requested visual template.
metadata:
  version: 1.1.0
  category: office
  tags: google, slides
  aliases: google slides, diapositivas google, presentación compartida
  required_tools: tool_search
  requires: Authorized Google Slides connection
  effects: external-write
---
# Google Slides

## Scope

Use for native cloud presentations. Use documents-presentations for local PPTX. A content outline is not a rendered deck.

## Inputs

Presentation ID or template, audience, message, slide scope and supplied assets.

## Workflow

1. Discover presentation read, copy, edit and preview capabilities. Inspect page dimensions, masters, layouts and target slide/object IDs.
2. Write one clear message per slide and select existing layouts suited to its content. Preserve supplied brand and template relationships instead of inventing a replacement theme.
3. Use stable object IDs and narrowly scoped requests. Preserve untouched slides and notes; avoid global text replacement when the same placeholder has different meanings.
4. After editing, inspect available thumbnails or renders for overflow, contrast and ordering. If preview is unavailable, check geometry and content but state that appearance remains unverified.
5. Return the actual deck link, affected slide numbers and any unsupported request.

## Native Fénix connection

Discover the account through `connector_call`. Inspect `workspace_schema` for service `slides` and the required `slides.presentations.*` method, then invoke `request` with a relative path. Keep object IDs, page IDs and revision controls in the batch body; read the current presentation before editing. Use thumbnails only when the account exposes the corresponding method. A partially checked connection requires a read of the actual presentation, not a blanket assertion that Slides works.

## Verification

Confirm slide count, object text, order and source attribution. A successful API response alone does not establish readable layout.

## Limits

Remote editing requires a connection. Do not claim that unsupported transitions, image generation or exports have been performed.
