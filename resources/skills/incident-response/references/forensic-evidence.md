# Forensic evidence and timelines

Use for supplied logs, packet captures, host artifacts or memory images. Work from preserved copies and record source identity, acquisition method/time, hash, tool version and transformations. A matching hash establishes unchanged bytes, not authenticity, completeness or who produced them. Document handling rather than claiming a formal chain of custody you did not maintain.

Keep original timestamp text and timezone/offset alongside any normalized UTC value. Separate event, ingestion, acquisition and detection times. If a timezone is absent or a clock is skewed, retain uncertainty; do not sort an inferred chronology into false precision. Correlate independent sources with stable identifiers and distinguish a repeated event from duplicate ingestion.

Build a bounded timeline: observation, source/artifact reference, timestamp interpretation, supported inference and unresolved alternatives. Absence from logs is meaningful only when collection, retention and event generation were verified. A suspicious string, process name or IOC hit is not attribution by itself.

For packet evidence, identify capture coverage, dropped packets and direction; reassemble flows before inferring application messages. For memory/host tools, check the exact format and available symbol support. Tool failure or unsupported images remain unresolved evidence, not clean findings.

Preserve originals before containment where operationally feasible. Do not delay an urgent authorized mitigation merely to complete an ideal evidence collection. Record what was changed and what evidence was lost. Do not execute extracted binaries, contact observed endpoints or upload private samples as an implicit analysis step.

## Español

Conserva copias, hashes y procedencia. Distingue hora del evento, recepción y adquisición, manteniendo incertidumbre de zona o reloj. Correlaciona fuentes antes de atribuir causas; ausencia de registros no prueba ausencia de actividad. Documenta las pérdidas de evidencia durante mitigaciones necesarias.
