# Timeline and delivery

For a multi-scene edit, record the purpose and source of each shot, frame rate, frame interval, narration/caption cues and output requirements. Keep asset provenance and availability separate from the timeline. Measure rendered narration before locking scene timing; word counts only estimate duration. Distinguish captured footage, generated imagery and reconstructed UI demonstrations.

Use half-open frame intervals `[start_frame, end_frame)`: the first frame belongs to the scene and the end frame starts the next. Rational frame rates such as `30000/1001` avoid converting every scene through rounded seconds. Use the actual renderer's timebase and account for transition handles and overlay tracks separately.

## Local timeline check

`python3 scripts/check_timeline.py timeline.json` validates a contiguous primary scene track and explicitly paired narration/visual cue IDs:

```json
{
  "fps": "30000/1001",
  "duration_frames": 120,
  "tolerance_frames": 2,
  "scenes": [
    {"id": "intro", "start_frame": 0, "end_frame": 60},
    {"id": "demo", "start_frame": 60, "end_frame": 120}
  ],
  "cues": [
    {"id": "open-panel", "scene_id": "demo", "visual_frame": 70, "audio_frame": 71}
  ]
}
```

The checker rejects invalid frame types, duplicate IDs, gaps, overlaps, out-of-range cues and misalignment beyond the declared tolerance. A nearby unrelated event cannot satisfy a cue. Exit codes: `0` consistent, `2` timing issues, `1` malformed input. It checks only the declared primary track and paired frame positions: it does not inspect media, verify cue meanings, validate overlay/transition tracks or prove lip sync.

## Render evidence

Render a representative segment with real assets, then probe the final output for duration, dimensions, codec and expected streams. Decode the file to catch corrupt media. Inspect scene boundaries, representative frames and beginning/middle/end playback; listen for joins, truncation and narration/music balance. Vary sampled times to catch static or blank scenes rather than inspecting one favorable frame. Keep technical validity, perceptual quality and factual accuracy as separate results.

Preserve inputs and existing outputs unless replacement is intended. Stream-copy cuts may follow keyframes; use deliberate re-encoding when exact cuts are required. A design preview, frame storyboard or queued provider job is not a delivered playable video. Return the actual artifact and any unperformed visual/listening checks.

## Español

Mide narración real y usa intervalos de fotogramas explícitos. El auxiliar comprueba escenas y cues identificados, sin renderizar ni validar significado. Revisa el archivo final con probe, decodificación, imágenes y escucha; una animación reconstruida no es una grabación real ni prueba de ejecución.
