---
name: skill-authoring
description: Create or improve a focused Fénix skill with discoverable metadata, original instructions and realistic
  validation.
metadata:
  version: 1.1.0
  category: knowledge
  tags: skill, authoring
  aliases: crear skill, mejorar habilidad, catálogo skills, estructura skill
  required_tools: file_read,file_write,files_search
  requires: Fénix skill library and an observed reusable task
  effects: local-write
---
# Skill Authoring

## Scope

Use for manual skill creation or revision. Do not convert every single-use task into a new skill or duplicate an existing purpose.

## Inputs

Target task, failure examples, available tools, existing related skills and intended user language.

## Workflow

1. Search the current library by purpose and aliases. Extend the best existing skill if the outcome is the same; split only when discovery or workflow needs a distinct capability.
2. Write name and a specific trigger description, then version/category/tags/aliases/required_tools/requires/effects metadata. Keep metadata values strings and tool names real.
3. Use Scope, Inputs, Workflow, Verification, Limits and Español sections. Put bulky optional detail in a relative references file and state when to load it.
4. Write original instructions aligned to Fénix capabilities. Do not copy upstream install commands, provider schemas or hidden permission rules.
5. Validate format, discovery on realistic EN/ES requests, irrelevant-query behavior and tool availability. For behavior changes, compare baseline/candidate on representative tasks and retain failures.

## References

Load [the authoring template](references/template.md) when creating an entrypoint or checking the common format.

Before adding a new entry, read references/routing-and-boundaries.md when its outcome overlaps an existing skill. Split only where decisions, prerequisites or verification differ; add aliases for synonyms and references for optional depth.

## Verification

Check all links and resource boundaries, duplicate purposes and dependency honesty. Syntax and retrieval checks do not prove model task accuracy.

## Limits

Do not install unreviewed scripts or treat a skill as permission to use unavailable tools. Keep customized user skills and activation state intact.

## Español

Busca duplicados antes de crear. Escribe instrucciones propias con metadatos uniformes y valida formato, recuperación y comportamiento por separado.
