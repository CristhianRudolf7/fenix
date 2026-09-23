# Caption retrieval protocol

Run the bundled helper only after loading the skill so the execution service mounts its resources. Use `resource_path` returned by `skill_load`; do not guess a controller installation path. Shell-quote arguments and use a canonical video URL without user-supplied shell text.

```text
/opt/fenix-youtube/bin/python -I <resource_path> 'https://www.youtube.com/watch?v=VIDEO_ID' --languages es,en
```

Replace `VIDEO_ID` with the actual 11-character ID. Watch, short-link, Shorts, live and embed URLs are accepted; playlists/channels without a video ID and foreign hosts are rejected. HTTP input is canonicalized to HTTPS. Tracking parameters and playback fragments are not sent onward. `--strict-language` disables fallback to another available source language. A base language such as `es` includes regional tracks such as `es-419`; a specific regional request matches that region exactly.

The adapter uses `youtube-transcript-api==1.2.4`; the surrounding URL validation, transport limits, selection, normalization, provenance and error protocol are original Fénix code. There is no summarization provider in the helper. The agent uses the caption evidence to answer the user's request.

## Output

One JSON object on stdout. Exit 0 requires `status: ok`, nonempty `segments` and complete selected-track retrieval. Segment entries contain `start_seconds`, `duration_seconds`, `text` and a canonical timestamp URL. Metadata identifies source language, automatic captions, fallback, retrieval time, adapter version and a SHA-256 of normalized segments. `caption_end_seconds` is the latest caption end, not verified video duration. Overlapping and repeated captions are retained rather than guessed away.

Exit 2 reports `status: error`, a stable `code` and a safe message, with no partial transcript. Codes include `invalid_url`, `invalid_languages`, `dependency_missing`, `language_unavailable`, `no_captions`, `empty_captions`, `invalid_captions`, `access_blocked`, `video_unavailable`, `provider_changed`, `network_error`, `timeout`, `limit_exceeded` and `retrieval_failed`. Do not infer “no subtitles” from a network or format failure.

The helper limits retrieval to five requests, four MiB per decompressed response, two MiB of caption text, 20,000 segments and a 60-second Linux CLI deadline. It permits only YouTube watch/player/timedtext endpoints for the same video, refuses redirects, and does not import environment proxies, `.netrc` or browser cookies. Set a command timeout slightly above the helper deadline. The runtime's tighter limits still apply; this HTTP policy is not a substitute for runtime isolation.

Large JSON output may be shortened in a tool preview. After completion, recover `command_output` pages with the returned `process_id`, starting at `offset: 0`, then using each `next_offset` until `has_more: false`. Require `log_complete: true` and no `output_error`. The `command_status` cursor can advance over omitted preview bytes; it is not a safe starting point for recovering the full transcript. Do not claim full speech coverage just because the selected caption track was fully fetched. Subtitle gaps can omit words, sounds or entire visual demonstrations.

## Unavailable prerequisites

Use an operator-built `Dockerfile.youtube` image layered on the existing tenant/workspace image, or a workspace image that includes the pinned dependencies. Do not install packages on the controller or bypass a disabled network. If the image or network is unavailable, report that prerequisite; user-supplied captions remain a useful alternative.

## Español

Ejecuta el helper cargado mediante el servicio aislado y recupera la salida completa. El código 0 y `status: ok` indican descarga de una pista, no cobertura de todo el contenido visual o hablado. Los errores diferencian falta de subtítulos, bloqueo, cambios del proveedor y límites. No uses cookies personales ni proxies para eludir restricciones. Sin dependencias o red disponibles, comunica el requisito y trabaja con subtítulos aportados.
