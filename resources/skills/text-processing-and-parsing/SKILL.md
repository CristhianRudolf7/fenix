---
name: text-processing-and-parsing
description: Implement and test parsers, regular expressions and text transformations. Regex fechas registros texto.
metadata:
  version: 1.1.0
  category: engineering
  tags: text, processing, and, parsing
  aliases: parser, regex, parsear, expresión regular, texto
  required_tools: file_read,file_write,file_edit,command_run
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Text Processing And Parsing

## Scope

Implement and test parsers, regular expressions and text transformations. Regex fechas registros texto.

## Inputs

Input format, desired output, malformed cases and actual caller API.

## Workflow

Derive input/output examples from the requested format. Start with a small executable candidate and test it immediately; avoid spending the entire generation on speculative reasoning. Include positive, negative, boundary and malformed cases. For regex check capture groups and the exact consumer API such as findall versus search. Confirm the pattern matches the requested entity, not another plausible field. Check escaping through JSON and shell layers. Save small edits, run representative cases, inspect failures and revise. Do not infer success from syntax alone.

## Verification

Test valid, invalid and boundary cases through the actual consuming API.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.
