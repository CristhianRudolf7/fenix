---
name: memory-and-reminders
description: Retrieve scoped memory, review candidates, maintain evidence-backed entities and relationships, and
  manage requested reminders.
metadata:
  version: 2.1.0
  category: knowledge
  tags: memory, and, reminders
  aliases: memoria, recordar, recordatorio, preferencia, relación, cron
  required_tools: memory_search,memory_manage,memory_context,knowledge_manage,schedule_manage
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Memory and reminders

## Scope

Retrieve scoped memory, review candidates, maintain evidence-backed entities and relationships, and manage requested reminders.

## Inputs

Requested fact or reminder, project scope, evidence and relevant time.

## Workflow

Use for remembering, correcting, forgetting, reviewing candidates, investigating relationships, or scheduling a reminder. SQLite is canonical; exported Markdown is a generated reference.

1. Resolve the task project before reading or writing. Global preferences are separate from project facts. Never widen the scope because a name appears in source material.
2. Use `memory_search` for active facts; exact words, names and IDs are inexpensive. `since` and `as_of` are RFC3339 dates. Enable `semantic` only for paraphrases when embeddings are configured; fallback results report degradation. Do not invoke synthesis as another LLM call: reason over the evidence in the current turn.
3. Use `memory_manage` for atomic claims. Metadata: kind (`fact`, `preference`, `decision`, `constraint`, `commitment`, `assumption`); status (`candidate`, `confirmed`, `conflict`, `expired`); assertion (`explicit`, `extracted`, `inferred`); optional entity_id, valid_from/valid_until, pinned, supersedes; evidence is a list of exact `{reference, quote}` records using `message:<id>` or `checkpoint:<id>`.
4. Explicit owner saves can be confirmed. Automatic extraction remains candidate; exact repeated preferences need three DISTINCT source messages to promote. Inferences remain labeled candidates. Repeated extraction of one message is not new support. An explicit owner confirmation may replace the assertion with explicit and provide its source quote.
5. Use `memory_context` actions `candidates` and `reviews` to inspect pending claims and possible overlap. Similarity is not contradiction. Review both sources: keep complementary facts, mark conflicting claims, or use `supersedes` for an explicit correction. `knowledge_manage review_resolve` records the pair decision with id, second_id and decision (complementary/duplicate/conflicting/dismissed); it does not silently rewrite the facts.
6. `knowledge_manage entities` and `memory_context card` resolve existing names/aliases before creating an entity. `entity_save` takes `{name, kind, aliases, description}`. Reuse an existing kind and canonical entity when appropriate.
7. Adapt relationships to the PROJECT, not the user's profession. `schema_save` takes `{name,description,source_kinds,target_kinds,status}`; use English snake_case names, start uncertain predicates as candidate, and activate a clearly defined schema when needed. Every change creates a version. An updated schema requires existing edges to be revalidated. Example research: paper `cites` paper, experiment `evaluates` hypothesis, experiment `uses_dataset` dataset. Software: module `depends_on` module, issue `fixed_by` change. Personal: person `participates_in` activity. These examples are not facts or mandatory schemas.
8. `relation_save` takes `{source_id,target_id,predicate,evidence_memory_id,status}`. Confirmed edges require active, sourced memory in the same scope. Never infer support, causality or contradiction from co-occurrence or embedding similarity. `memory_context neighbors` returns bounded paths and evidence IDs.
9. `memory_context history` exposes prior memory revisions, including withdrawn content, for deliberate auditing. Delete withdraws a claim and blocks recapture of the same normalized claim; it does NOT purge original transcripts or backups. `delta` uses an integer sequence cursor; advance only after consuming the batch and follow `has_more`.
10. Keep durable knowledge separate from scheduled actions. Use `schedule_manage` for requested reminders, resolve dates in America/Lima, and ask about ambiguous times. A commitment in memory does not itself schedule or authorize an action. Link to existing task/plan IDs instead of copying mutable progress as durable facts.

Report actual stored IDs and unresolved uncertainty. Automatic capture, maintenance, and compaction rehydration are runtime functions; do not imitate them by polling or repeatedly reading entire transcripts.

Use scheduled-workflows for recurring automation with overlap or missed-run policy. This skill remains responsible for scoped memory and individual reminders; describing a habit alone does not authorize a new background job.

## Verification

Report actual stored IDs, provenance, scope and timezone; keep candidates and confirmed facts distinct.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Resuelve el proyecto antes de consultar o guardar. Conserva hechos atómicos con procedencia exacta, tipo, estado y vigencia. Lo extraído automáticamente permanece como candidato; tres mensajes distintos pueden confirmar una preferencia repetida. Los supuestos siguen identificados como inferencias. Corrige con evidencia y conserva el historial; retirar un recuerdo no borra conversaciones ni backups.

Consulta entidades y alias antes de crearlos. Adapta los tipos de relación a cada proyecto mediante esquemas versionados: investigaciones, software y vida personal pueden coexistir. Una relación confirmada necesita evidencia vigente del mismo ámbito. La semejanza no demuestra causalidad ni contradicción. Revisa los pares sugeridos y pregunta cuando no se pueda decidir con las fuentes. Cambiar un esquema exige revalidar sus relaciones.

La búsqueda léxica, temporal, tarjetas y recorridos no requieren llamadas al modelo. La búsqueda semántica es opcional. La extracción sí utiliza el modelo configurado y se puede desactivar desde la web. Los compromisos guardados no crean recordatorios ni ejecutan acciones automáticamente; usa las herramientas de programación y aclara las fechas ambiguas.
