---
name: audio-transcription
description: Transcribe or prepare audio with timestamps, uncertainty and source-linked corrections.
metadata:
  version: 1.0.0
  category: creative
  tags: audio, transcription
  aliases: audio, transcripción, voz, subtítulos, diarización
  required_tools: file_read,file_write,tool_search
  requires: Available transcription model/service and authorized audio input
  effects: local-write
---
# Audio Transcription

## Scope

Transcribe or prepare audio with timestamps, uncertainty and source-linked corrections.

## Inputs

Language, desired verbatim/edited output, timestamps and speaker needs. Prerequisite: Available transcription model/service and authorized audio input.

## Workflow

1. Inspect duration, channels and audio quality; preserve the original and use a suitable supported format.
2. Transcribe with timestamps when needed and mark uncertain spans instead of inventing words. Distinguish speaker labels from verified identities.
3. Check names, numbers and technical terms against the audio or supplied glossary. Keep edits separate from verbatim text.
4. Export text or subtitle segments with valid ordering and timing, and link unclear portions for review.

## Verification

Spot-check difficult passages and timing; report inaudible sections and any unreviewed coverage.

## Limits

Do not infer personal identity or sensitive attributes from a voice. No STT provider is installed by this skill.

## Español

Marca incertidumbre y revisa cifras y nombres; etiquetas de voz no demuestran identidad de personas.
