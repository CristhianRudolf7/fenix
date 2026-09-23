---
name: sql-analysis-and-optimization
description: Analyze SQL, preserve query semantics and measure database performance. SQL consultas equivalencia
  rendimiento.
metadata:
  version: 1.1.0
  category: engineering
  tags: sql, analysis, and, optimization
  aliases: sql, consulta, base datos, índice, optimizar consulta
  required_tools: file_read,file_write,file_edit,command_run,command_status
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Sql Analysis And Optimization

## Scope

Analyze SQL, preserve query semantics and measure database performance. SQL consultas equivalencia rendimiento.

## Inputs

Schema, original query, result semantics and performance question.

## Workflow

Inspect schema, indexes and the original query. Preserve duplicates, null handling, ordering and requested columns. Bound baseline execution time and preserve its process handle; avoid waiting indefinitely before writing a candidate. Use an isolated read-only command for analysis when the runtime supports it, never merely claim read-only execution. Save a candidate before expensive measurement, then compare columns and rows on an unchanged database or independent copies. Run EXPLAIN where supported. Measure the same environment and cache conditions; one timing does not establish a general speedup. Do not modify the source database to make a query appear faster. Verify the final artifact exists and contains the tested query.

## Verification

Compare columns, rows, duplicates and ordering on unchanged data before comparing timings.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Preserva semántica, filas y columnas. Limita mediciones lentas, guarda el candidato, compara resultados y mide en condiciones equivalentes.
