# Fénix skill template

Use this when creating a skill with a new, non-duplicated purpose. Replace placeholders before installing; do not install this template as a skill.

```yaml
---
name: descriptive-capability
description: Use when the user needs a specific outcome; include concrete trigger vocabulary.
metadata:
  version: "1.0.0"
  category: "engineering"
  tags: "specific, discovery, terms"
  aliases: "términos, búsqueda, español"
  required_tools: "file_read, file_write"
  requires: "Available task-authorized Fénix tools"
  effects: "local-write"
---
```

Use these body headings: `Scope`, `Inputs`, `Workflow`, `Verification`, `Limits`, `Español`.

- Scope: intended outcome and neighboring skills that should handle other requests.
- Inputs: facts that materially affect execution, reusing context already available.
- Workflow: concrete decision points, exact tool names only when they exist, and recovery after ambiguous failures.
- Verification: observable evidence of completion, not a generic promise to check quality.
- Limits: unsupported actions, dependencies and side effects. Metadata is documentation, not permission enforcement.
- Español: a useful Spanish summary including material limitations; expand it for complex capabilities.

Optional resources belong under `references/`, `assets/` or `scripts/`. Link each resource and say when to load it. Scripts require source review and execution tests; a load operation only reads text and does not execute anything.

Test retrieval with meaningful positive and negative requests in English and Spanish. Behavioral comparisons should hold the model/settings/tasks constant and include failures, time and cost. Neither YAML validity nor a search hit proves that a model follows the workflow.

## Español

Cada skill debe tener un propósito distinto y pasos concretos. Usa metadatos uniformes, herramientas reales, límites y resumen en español. Añade recursos solo cuando sean necesarios y valida recuperación y comportamiento por separado.
