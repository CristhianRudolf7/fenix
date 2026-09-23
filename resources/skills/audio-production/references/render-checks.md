# Audio render checks / Comprobación del audio

Inspect streams with an available `ffprobe` and decode the output using `ffmpeg` or the actual media library. Select streams explicitly when a source has multiple tracks. Match sample rate, layout and timebase before joining; inspect boundaries for clipping, silence and repeated speech. Use file arguments or structured APIs for text and paths rather than injecting untrusted script text into shell commands.

For requested loudness normalization, measure integrated loudness, range and true peak, then apply the actual platform's target and inspect the final encoded file. A two-pass `loudnorm` workflow needs first-pass measurements for that exact input; an edit invalidates them. Silence and very short inputs can produce undefined loudness statistics. No single LUFS target fits every deliverable. See [FFmpeg audio filters](https://ffmpeg.org/ffmpeg-filters.html).

Technical checks and listening answer different questions. Record which were possible. Do not label generated tones as speech-quality testing or successful decoding as evidence of intelligible narration.

ES: inspecciona streams y decodificación, alinea formatos al unir y mide el archivo final. La escucha confirma aspectos que una medición de duración o volumen no puede demostrar.
