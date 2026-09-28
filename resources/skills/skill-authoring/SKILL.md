---
name: skill-authoring
description: Create, improve or evaluate a Fénix skill using discoverable metadata and baseline comparisons, including pressure scenarios. Crear y evaluar skills frente a una línea base.
metadata:
  version: 1.5.0
  category: knowledge
  tags: skill, authoring
  aliases: crear skill, mejorar habilidad, catálogo skills, estructura skill, evaluate skill, pressure scenarios, evaluar skill, línea base, escenarios de presión, bounded revision, skill regression, mejora basada en fallos, procedimiento repetible, skill stocktake, auditoría catálogo, inventory skills, library maintenance
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
2. Write name and a specific trigger description, then version/category/tags/aliases/required_tools/requires/effects metadata. Keep metadata values strings and tool names real. Select category from the current category catalog supplied at load time; never create category IDs or synonyms. All categories share the same catalog; categories never grant permissions. Write the executable source in English; Fénix creates Spanish display translations separately.
3. Use Scope, Inputs, Workflow, Verification and Limits sections in the canonical English SKILL.md. Store the complete Spanish translation separately as SKILL.es.md; do not append a bilingual summary to the canonical file. Put bulky optional detail in a relative references file and state when to load it.
4. Write original instructions aligned to Fénix capabilities. Do not copy upstream install commands, provider schemas or hidden permission rules.
5. Validate format, discovery on realistic EN/ES requests, irrelevant-query behavior and tool availability. For behavior changes, compare baseline/candidate on representative tasks and retain failures.

## References

Load [the authoring template](references/template.md) when creating an entrypoint or checking the common format.

Before adding a new entry, read references/routing-and-boundaries.md when its outcome overlaps an existing skill. Split only where decisions, prerequisites or verification differ; add aliases for synonyms and references for optional depth.

When evaluating a changed procedure, load [behavioral evaluation](references/behavioral-evaluation.md). Keep format, retrieval and model behavior as separate claims; preserve baseline failures and negative cases instead of measuring compliance with the new wording alone.

For an observed failure, identify whether it came from selection, ambiguous instructions, missing evidence, tools or the runtime before rewriting. Propose a small edit tied to that cause, remove superseded guidance, and check conflicting instructions and newly required resources. A repeated example or an uncorrected answer does not establish a global preference. Preserve the owner's explicit choices.

Use experiment-and-benchmark-design for executing and scoring paired trials. Its evaluation resource checks task identity and repeated runs; it does not optimize or install skills automatically. Keep selection cases used for tuning separate from a final unseen task set. Count instruction size separately from actual context usage.

When turning a successful workflow into reusable automation, start from an observed execution and its inputs, outputs and permissions. Separate stable procedure from one-off identifiers, account state and page selectors. Exercise replay with an independently checked fixture and a changed-input or failure case; a stale captured page passing its own test does not prove the live site still works. Keep credentials and user content out of fixtures. Publish a usable complete resource set, and preserve customized installed copies and enabled state during stock upgrades. Do not turn a successful read workflow into authority for recurring writes.

For a broad catalog review or incremental maintenance, load [library maintenance](references/library-maintenance.md). Map canonical capabilities and review depth, hash complete bundles, preserve previous evidence, and distinguish unavailable usage records from proven non-use. Do not import runtime hooks or a background learning service as a side effect of adding instructions.

## Verification

Check all links and resource boundaries, duplicate purposes and dependency honesty. Syntax and retrieval checks do not prove model task accuracy.

## Limits

Do not install unreviewed scripts or treat a skill as permission to use unavailable tools. Keep customized user skills and activation state intact.
