---
name: video-editing
description: Edit or transcode video with explicit time ranges, stream mapping and verified audio/video output.
metadata:
  version: 1.2.0
  category: creative
  tags: video, editing
  aliases: vídeo, video, ffmpeg, recortar, subtítulos, transcodificar, storyboard, timeline, narration cues, escenas, montaje, fotogramas
  required_tools: file_read,file_write,command_run,tool_search
  requires: Installed media probe/editor such as FFmpeg
  effects: local-write
---
# Video Editing

## Scope

Edit or transcode video with explicit time ranges, stream mapping and verified audio/video output.

## Inputs

Inputs, timeline, output format, dimensions, captions and quality constraints. Prerequisite: Installed media probe/editor such as FFmpeg.

## Workflow

1. Probe actual codecs, streams, frame rates, rotation and duration before constructing commands.
2. Define cuts and joins in a consistent timebase. Use stream copy only when codec/container and cut accuracy permit it; otherwise re-encode deliberately.
3. Preserve or select audio tracks, subtitles and metadata explicitly. Avoid repeated lossy conversions and unexpected overwrites.
4. Render a short representative segment before a long job when settings are uncertain; inspect the final file and synchronization.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

For a multi-scene video or narrated demo, load [timeline and delivery](references/timeline-delivery.md). Build a shot list with source assets, provenance, timing and measured narration. Use the local timeline checker before rendering to catch out-of-range scenes and mismatched cue IDs. A synthetic UI or terminal animation must be labeled as a reconstruction; it cannot stand in for a requested recording or evidence that commands ran.

## Verification

Check duration, dimensions, stream presence, sync and representative beginning/middle/end playback.

## Limits

Do not claim subtitle accuracy or visual quality from exit status alone, and do not upload private footage as an implicit fallback.

## Español

Inspecciona streams y tiempos, selecciona pistas y revisa reproducción y sincronización del archivo final.
