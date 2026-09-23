---
name: meeting-notes
description: Turn meeting notes or transcripts into sourced decisions, action items and a useful agenda when requested.
metadata:
  version: 1.1.0
  category: office
  tags: meeting, notes
  aliases: acta, minuta, acuerdos, reunión, responsables
  required_tools: file_read,file_write,tool_search
  requires: Supplied notes or an authorized transcript source
  effects: local-write
---
# Meeting Notes

## Scope

Use for meeting artifacts, not reconstruction of missing conversations. Use conversation-recap for Fénix chat history.

## Inputs

Transcript or notes, date, participants when known, audience and requested format.

## Workflow

1. Read the provided material and retain speaker/time or section references. Mark unintelligible segments and uncertain speaker attribution.
2. Separate decisions from proposals, objections and unresolved questions. Extract action, owner and due date only when supported; leave unassigned work explicitly unassigned.
3. Group by topic and remove repeated discussion without losing changed decisions. Link later corrections to the earlier proposal rather than recording both as final.
4. Draft a concise record with decisions, actions, blockers and next meeting topics. For an agenda, express the desired outcome and time allocation without inventing attendee commitments.
5. Save to the requested location or return the text. Creating tasks, reminders or sending minutes requires the corresponding user request.

## Existing Google Meet transcripts

When the user supplies a Meet link or conference reference, discover `connector_call` and inspect service `meet` using `workspace_schema`. Resolve the conference record and list existing transcripts and entries through the selected account. Preserve resource IDs, speaker references and times, and check pagination completeness before summarizing. A transcript may not exist or may be inaccessible because of account policy or meeting settings; say so instead of inventing dialogue. The native integration reads existing artifacts; it does not join, record or transcribe a meeting. Use google-workspace-workflows for preparation spanning calendar and linked documents.

## Verification

Every decision and owner should map to source material. Check dates and names, and keep unresolved issues visible.

## Limits

Do not infer personality, intent or consensus from speaking frequency. No recording/transcription capability is installed by this skill.

## Español

Convierte notas en acuerdos y acciones con fuente. No inventes responsables ni fechas y no confundas propuestas con decisiones aprobadas.
