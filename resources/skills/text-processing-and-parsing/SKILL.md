---
name: text-processing-and-parsing
description: Implement and test parsers, regular expressions and text transformations. Regex fechas registros texto.
version: 1.0.0
required_tools: file_read,file_write,file_edit,command_run
---

# Text Processing And Parsing

## Procedure

Derive input/output examples from the requested format. Start with a small executable candidate and test it immediately; avoid spending the entire generation on speculative reasoning. Include positive, negative, boundary and malformed cases. For regex check capture groups and the exact consumer API such as findall versus search. Confirm the pattern matches the requested entity, not another plausible field. Check escaping through JSON and shell layers. Save small edits, run representative cases, inspect failures and revise. Do not infer success from syntax alone.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Crea un candidato pequeño y pruébalo. Incluye casos negativos y límites; verifica capturas, escapes y la API que consume el patrón.
