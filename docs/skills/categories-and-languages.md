# Skill categories, languages and model roles

[Español](categories-and-languages.es.md)

## One taxonomy

`resources/skill-categories.json` is the default operator-owned catalog. Set
`FENIX_SKILL_CATEGORIES` to an external JSON file to edit it without rebuilding.
Each entry has a stable lowercase `id`, `labels.en`, `labels.es` and `aliases`.
Aliases are trimmed and matched case-insensitively; ambiguous aliases are rejected.
Keep IDs stable when renaming labels. Unknown legacy/manual categories are displayed
as `other`; learned skills must select an existing ID. Editing the catalog does not
require an agent skill or model call. The catalog is supplied only when authoring skills.

For isolated deployments, configure `skill_categories` in `/etc/fenix/execution.json`
and the same `FENIX_SKILL_CATEGORIES` path for the gateway. The gateway service user
needs read access to that file and traversal access to its parent directories.
Provisioning mounts the
root-owned file read-only into each controller. After changing its content, restart
the gateway to reclassify existing marketplace rows from their source bundles;
tenant indexes detect catalog changes automatically. Reprovision if changing the path.

Marketplace publication derives category from SKILL.md; an old client's category
parameter is accepted for compatibility but ignored. Existing publications are
reclassified without changing their source, review hash or visibility.

All tasks share the same catalog, including security. Categories and legacy
specialization metadata do not restrict skill access or grant permissions. Disabled
skills, tenant isolation and ordinary tool authorization remain enforced.

## Two persisted display languages

The agent always loads the canonical English `SKILL.md`. A server lifecycle worker
uses the configured fast model to prepare Spanish Markdown after creation, import,
updates or rollback. Manual file edits are detected within the next background scan
(ten seconds after the previous batch). Four translations can run concurrently;
failures retain a durable retry deadline and never overwrite the English source.

Each revision stores a complete `state/skill-localizations/<source-hash>/SKILL.es.md`
with Spanish title and description in frontmatter. Source hashes prevent a stale
translation from being shown after edits. The file is written atomically only after
validating metadata, protected code and the unchanged source revision. Import and
learning activation notify the worker. This service runs only with the server, not
in native benchmark workers.

Cards and previews read saved files without provider calls, including cache misses.
The UI indicates unavailable versions instead of silently showing English or
translating on click. Opening disabled skills for display does not enable execution.
Exports include the saved Spanish file when available; marketplace previews read
that snapshot and verify its source hash. Public review still covers the bundle.

## Shared model roles

The intelligent role is the existing main task model configured during setup.
The fast role is the existing account-scoped conversation model (default: GPT-6 Luna with low reasoning),
now also used for translations. `GET /api/model-roles` reports both roles without
credentials; `GET/POST /api/model-roles/fast` configures the fast role. The legacy
fast model settings endpoint is `/api/model-roles/fast`,
so existing installations require no credential migration. Configure it from the
chat's fast-model settings. Publication vulnerability reviews retain their separate
operator-managed credential boundary; tenant users cannot replace that auditor.

The fast connection persists `reasoning_effort`; changing it does not alter the intelligent model. Hosted publication review uses the operator’s private Sentinel configuration with explicit `provider` and `reasoning_effort` fields.

Translation validation preserves exact code fragments and their counts, plus fenced block order. Inline identifiers may move within a translated sentence without causing a false rejection.
