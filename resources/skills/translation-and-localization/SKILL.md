---
name: translation-and-localization
description: Translate and localize content while preserving meaning, terminology, placeholders and formatting.
metadata:
  version: 1.0.0
  category: writing
  tags: translation, and, localization
  aliases: traducir, traducción, localización, inglés, español, bilingüe
  required_tools: file_read,file_write,file_edit
  requires: Source text and target locale
  effects: local-write
---
# Translation And Localization

## Scope

Use for translation of prose or product strings. Code identifiers and placeholders are not prose to translate.

## Inputs

Source and target languages/locales, glossary, audience and format constraints.

## Workflow

1. Identify protected tokens: variables, URLs, IDs, code, formatting markers and product names. Read neighboring strings to resolve grammatical context.
2. Translate meaning and tone, preserving claims and uncertainty. Use consistent terms from the supplied glossary; flag ambiguous domain terms.
3. Localize dates, number formats, units and formality only where intended. Preserve machine-readable values and interpolation syntax.
4. For UI strings, inspect plural rules, gender, text expansion and accessibility labels. Do not concatenate fragments if the target language needs a different order.
5. Check source and target side by side. Deliver parallel language files or the requested layout with any unresolved translation choices.

## Verification

Compare placeholder sets and markup structure, run the project’s locale checks if present and review rendered text when layout matters.

## Limits

Do not silently translate legal meaning into a stronger claim or invent a certified translation. Missing context should remain a visible question.
