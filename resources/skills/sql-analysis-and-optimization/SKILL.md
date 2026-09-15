---
name: sql-analysis-and-optimization
description: Analyze SQL, preserve query semantics and measure database performance. SQL consultas equivalencia rendimiento.
version: 1.0.0
required_tools: file_read,file_write,file_edit,command_run,command_status
---

# Sql Analysis And Optimization

## Procedure

Inspect schema, indexes and the original query. Preserve duplicates, null handling, ordering and requested columns. Bound baseline execution time and preserve its process handle; avoid waiting indefinitely before writing a candidate. Use an isolated read-only command for analysis when the runtime supports it, never merely claim read-only execution. Save a candidate before expensive measurement, then compare columns and rows on an unchanged database or independent copies. Run EXPLAIN where supported. Measure the same environment and cache conditions; one timing does not establish a general speedup. Do not modify the source database to make a query appear faster. Verify the final artifact exists and contains the tested query.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Preserva semántica, filas y columnas. Limita mediciones lentas, guarda el candidato, compara resultados y mide en condiciones equivalentes.
