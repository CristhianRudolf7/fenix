# Faster ingestion with accountable results

Use for slow backfills, exports or incremental loads. Freeze a source snapshot or explicit high-water mark for the comparison, and track arrivals after that boundary separately. An increasing live tail is not proof that historical processing failed.

Measure extraction, network, queueing, parsing, transformation, writes and publication separately. Record source files/partitions, discovered rows, accepted rows, duplicates, rejects, pending work, committed outputs and observation time. Counts need the transformation semantics: filters and joins may intentionally change totals. Matching counts and maximum timestamps alone do not prove row identity or correctness; check keys, representative values and suitable checksums or reconciliation samples.

Try measured batching, bounded concurrency, native bulk operations or partitioning according to the bottleneck. Preserve rate limits, memory bounds and downstream backpressure. Compare candidates against the same source boundary and correctness gate; do not make throughput look better by omitting rejected records or stopping delivery accounting early.

Commit sink data before advancing its checkpoint, or use the stack's atomic mechanism. A manifest must identify source version and processing configuration, not just filename. Retry after an interrupted commit by reconciling publication identity; test a changed file, late event and duplicate batch. State what remains in the live tail and what has durably reached the serving dataset.

## Español

Separa la carga histórica de los datos que llegan durante la ejecución. Mide etapas y reconcilia claves, valores y rechazados, no solo conteos. Mejora lotes o concurrencia sin perder límites, publicación duradera ni capacidad de reanudar sin duplicados.
