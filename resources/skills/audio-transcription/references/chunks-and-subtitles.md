# Chunked speech and subtitles / Audio por segmentos y subtítulos

Choose model, precision and chunk size from actual memory and input limits; no model choice guarantees a machine cannot run out of memory. Preserve a manifest of input hash, chunk start/end and processing status. Keep original audio; failed chunks remain visible rather than disappearing from the transcript.

Use overlap/context where the recognizer needs it and inspect joins for repeated or dropped speech. Convert each chunk's local timestamps to the original timeline before merging. Speaker labels can change between chunks; do not equate identical label strings with a verified person. Silence, music and unrecognized speech are different coverage states.

Run `python3 <skill-directory>/scripts/check_srt.py subtitles.srt --duration 120.5` to validate UTF-8 SRT indices, timestamps, ordering, cue text and an optional media bound. Overlapping cues are reported because they can represent simultaneous speech. This helper does not recognize speech, merge chunks or assess reading speed and transcription accuracy. An empty file fails rather than passing vacuously; genuinely silent source material should be reported as such.

ES: conserva el mapa de segmentos y suma offsets a los tiempos. Revisa uniones, solapamientos y cambios de etiquetas de voz. El validador comprueba estructura y tiempos, no lo dicho en la grabación.
