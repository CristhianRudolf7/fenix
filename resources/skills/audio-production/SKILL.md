---
name: audio-production
description: Produce spoken narration, podcasts or mixed audio from a script using available TTS and audio tools, verifying the actual rendered file. Producción de voz y audio.
metadata:
  version: 1.0.0
  category: creative
  tags: audio, narration, tts, mixing
  aliases: texto a voz, narración, podcast, síntesis de voz, TTS, spoken audio
  required_tools: tool_search,file_read,file_write,command_run
  requires: Available speech synthesis or supplied audio; suitable local audio processing tools
  effects: mixed
---
# Audio Production

## Scope

Use for creating or mixing audio. Use audio-transcription for recognizing existing speech and video-editing for a video timeline. Keep a supplied script intact unless editing is part of the request.

## Inputs

Text, language/pronunciation, available voice, target duration, delivery format, channel needs and optional music or effects.

## Workflow

1. Discover installed/connected synthesis and processing tools; verify available voices and input syntax. Do not assume API keys, an upstream `sag` path or OpenClaw `MEDIA:` delivery semantics exist in Fénix.
2. Prepare text for the actual engine. Bracketed emotion labels are not portable SSML and may be spoken literally. Test a short representative sample including names, numbers and technical terms before rendering long content.
3. Estimate timing from that sample and measure the produced file. Speech rate depends on voice, language and pauses; a fixed words-per-minute constant cannot guarantee the requested duration. Adjust within the intended content and speed constraints.
4. For chunks, retain order, text boundaries, settings and source IDs. Listen across joins, preserve requested pauses and reconcile retries so a segment is not duplicated or omitted. Do not concatenate incompatible encoded files as raw bytes.
5. Mix with explicit stream/channel/sample-rate choices; keep speech intelligible. Use an appropriate loudness target and true-peak ceiling for the actual destination, measuring the final encode rather than treating a volume increase as normalization. Read [render checks](references/render-checks.md).
6. Deliver the actual artifact and script, with measured duration and any unreviewed passages. Generating a file and sending it to a person are distinct operations.

## Verification

Probe the saved output, decode it, inspect duration/channels and listen to representative speech and joins when an audio review tool is available. Technical decoding alone does not verify pronunciation or quality.

## Limits

No TTS provider or voice model is installed by this skill. Use authorized voices/assets; do not imply a real person recorded synthetic narration. Network synthesis must stay within the requested data and spending scope.
