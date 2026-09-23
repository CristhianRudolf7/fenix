---
name: structured-llm-output
description: Implement structured model outputs with schema validation and bounded repair of invalid responses.
metadata:
  version: 1.0.0
  category: data
  tags: structured, llm, output
  aliases: JSON modelo, salida estructurada, schema, extracción LLM
  required_tools: file_read,file_write,command_run
  requires: Model API supporting the selected output mechanism
  effects: local-write
---
# Structured Llm Output

## Scope

Implement structured model outputs with schema validation and bounded repair of invalid responses.

## Inputs

Schema, required semantics, failure behavior and consumer contract. Prerequisite: Model API supporting the selected output mechanism.

## Workflow

1. Define a small schema with meaningful required/optional distinctions and allowed values; schema validity alone does not verify factual content.
2. Use the provider’s supported structured-output/tool interface when available. Treat returned strings as data, never executable code.
3. Validate locally and classify truncation, refusal, protocol and semantic failures. Repair only the failing boundary rather than rerunning all prior effects.
4. Keep raw/redacted evidence and validation errors sufficient for diagnosis; stop or ask when missing facts cannot be reconstructed.

## Verification

Test malformed, extra, missing and adversarial fields plus a correct response through the actual consumer.

## Limits

Do not equate valid JSON with a correct answer or silently coerce a dangerous action into validity.

## Español

Valida formato y significado por separado; repara el límite fallido sin repetir efectos ya ejecutados.
