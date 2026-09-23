# Adaptive memory

Fénix stores memory in SQLite (schema version 13). This is the canonical store; `instructions/MEMORY.md` is an export of up to 200 active records, not a second writable database. Existing memories migrate as confirmed explicit facts without losing their IDs or project bindings. Historical versions start at migration; earlier overwritten content cannot be reconstructed retroactively.

## Records and evidence

A memory contains a title, atomic content, source, project/task IDs, timestamps and metadata:

- `kind`: fact, preference, decision, constraint, commitment, assumption.
- `status`: candidate, confirmed, conflict, superseded, expired, withdrawn.
- `assertion`: explicit, extracted, inferred. Inferred statements cannot be confirmed without changing their attribution following an owner confirmation.
- `entity_id`, `valid_from`, `valid_until` (RFC3339), `supersedes`, `pinned`.
- `evidence`: up to 32 exact `{reference, quote}` records. Agent writes validate `message:<id>` or `checkpoint:<id>` references against authorized originals. Owner edits through the local API are authoritative and can supply external references.

Confirmed, currently valid records enter automatic context. Candidate/expired/conflicting records remain available for review. Explicit correction can supersede a record in the same scope; the old record and revision trail remain. Similarity never automatically supersedes a fact.

Deleting a memory records a scoped withdrawal, removes its lexical index entry, and prevents recapture of the same normalized claim. This is withdrawal, not secure erasure: the original conversation, revisions, backups, and old embedding rows can retain text. Deliberate historical inspection can retrieve withdrawn content. A corrected claim can be saved separately.

## Scoped adaptive knowledge

Entities have an ID, name, English snake_case kind, description and aliases. Alias uniqueness is enforced transactionally within the project; the same name may refer to different entities in different projects. Relations require a registered, versioned schema with source/target kind constraints. Unknown predicates can be proposed as candidates; active schemas permit confirmed edges with active, sourced memory as evidence. Schema updates require revalidating edges against the new version. Disabled schemas and inactive evidence disappear from graph traversal immediately.

No profession selection is required. Agents use the bundled `memory-and-reminders` skill and the compact tool catalog to adapt a project's schema on demand. Research may use paper/hypothesis/experiment entities and `cites`, `evaluates`, `uses_dataset`; software may use modules and `depends_on`; personal organization may use people/activities and `participates_in`. These are examples, not preloaded assertions. Schema selection/expansion is performed by the conversing agent, not a classifier claiming certainty about a user's profession.

Traversal is deterministic breadth-first search, at most three hops and 100 results (the agent surface requests 30). It returns edge IDs, source memory IDs and paths. The graph stores explicit connections, not embeddings or inferred causality. It complements original documents and does not index an entire code repository.

## Retrieval and context

- `memory_search`: lexical FTS5, exact ID/title priority, lifecycle and scope filters before top-K. Optional `since` and `as_of` dates; historical retrieval uses the most recent recorded revision at that time with lexical matching. `as_of` is what was recorded then, not retroactive omniscience.
- `memory_context`: entity cards, neighbors, candidates, overlap reviews, history, delta, context_pack.
- `knowledge_manage`: entities, entity_save, schemas, schema_save, relation_save, review_resolve.
- `memory_manage`: existing get/add/update/delete API, extended with metadata.

The runtime builds a scoped context pack for master and subagents, and again after active-context compaction. It prioritizes pinned records, constraints, preferences and decisions, then query matches and entity cards. Payload packing uses a conservative bytes/3 token estimate, not an exact model tokenizer. Whole entries that do not fit are omitted with `dropped_count`; original records remain recoverable. The default memory allocation is 3,000 estimated tokens and is independent of task execution limits. Large cards can be retrieved explicitly.

`delta` uses a sequence cursor across fact, entity, relation and schema changes. Scope filtering happens before pagination. Advance `next_cursor` only after delivery; repeat the previous cursor on failure and follow `has_more`. It is a pull API; it does not enable a heartbeat or create reminders. Mutable task/plan progress remains in its existing task store rather than being duplicated as facts.

## Background extraction and maintenance

Automatic capture defaults on and is configurable from the web. A deterministic filter skips short/trivial, question-only, large and code-fenced inputs; eligible owner statements are durably queued after routing. The worker uses the configured conversation model, outside the foreground loop, at most six candidates per source and three attempts per job. Structured extraction must contain exact source quotes. All entries are validated before writes begin; retry after a partial storage failure uses source-based deduplication.

Same normalized preference plus three distinct source messages promotes an extracted candidate; replaying one message never increases its support. Other candidates require review. Wording variants are not automatically counted as repeats. The filter and extraction model can miss relevant statements; explicit `memory_manage` saves remain available. The extractor does not autonomously schedule commitments or manufacture edges.

Checkpoint capture reuses original owner-message sources rather than re-extracting model summaries or tool output as user facts. Prior task sources are also enqueued before conversation compaction. The checkpoint remains durable before capture is queued; existing recent messages and task/session IDs are preserved. This implementation does not perform unrestricted semantic harvesting of every tool output or all assistant text.

Every cycle excludes expired facts, incrementally proposes lexical overlap pairs for review, and optionally updates embeddings. The evidence-bearing retrieval section below describes selective background assessments and expanded diagnostics. Overlap is a hint, not proof of duplication/contradiction; review can mark complementary/duplicate/conflicting/dismissed, while actual correction/withdrawal is explicit. No automatic destructive merging. Entity cards omit invalid evidence, and graph references are protected by foreign keys. The worker reports failed jobs through task events and maintenance status; only memory jobs resume after service restart, not interrupted task effects.

## Optional embeddings

Disabled by default. Configure an OpenAI-compatible embeddings endpoint explicitly; HTTPS is required except for loopback HTTP. Keys are stored privately and scoped to the endpoint. Example local API request:

```json
{
  "automatic_capture": true,
  "context_budget_tokens": 3000,
  "extraction_interval_seconds": 30,
  "embedding_endpoint": "http://127.0.0.1:8080/v1/embeddings",
  "embedding_model": "your-embedding-model",
  "embedding_api_key": "your-optional-key"
}
```

POST to `/api/memory/settings`. GET omits credentials. Background indexing processes at most eight confirmed current memories per cycle. Content and endpoint/model identity invalidate stale vectors. `memory_search` with `semantic:true` embeds the query and fuses lexical/cosine result ranks using RRF. This costs an embedding request; regular context packing makes no embedding request. An unavailable service returns lexical results with `degraded_reason`. The initial semantic index can be incomplete; search reports its incremental nature. Temporal searches use historical lexical evidence, not current vectors. There is no mandatory reranker or separate synthesis model call.

## Local web/API

The memory page exposes scope, type, lifecycle state, source quotes, history, expiry, entity cards and automatic-capture settings. Entity/schema authoring and overlap resolution are available conversationally and via POST `/api/memory/knowledge`. GET/POST `/api/memory` preserve the existing CRUD shape with optional metadata. The server remains loopback-only; task tools additionally enforce fixed project write scope.

## Validation

Regression coverage includes migration, candidate promotion with unique provenance, withdrawal replay, temporal revisions, scoped aliases and typed edges, invalidated evidence, bounded graph traversal, context packing, incremental cursors, durable retries, overlap review, semantic invalidation, and mocked extraction/embedding providers. These tests establish behavior; they are not a claim of measured long-term memory quality on real-user benchmarks.

## Evidence-bearing retrieval and review

The agent's `memory_search` now consistently returns `{results, diagnostics, warnings, reviews}`. Each result contains `memory`, `score`, `branches`, `evidence`, `create_safety` and `freshness`. The administrative `/api/memory` list remains unchanged.

- Lexical candidates and optional vector candidates are fused with a deterministic relational arm. Explicit entity names/aliases anchor a scoped walk, two hops by default, at most three. Up to four anchors and 40 edges per anchor are considered; truncation is reported. Edge evidence memories and their paths participate in ranking. Alias resolution does not guess entities from semantic resemblance. `graph:false` disables this arm. Historical `since`/`as_of` searches exclude current graph and vector state.
- Exact content/ID matches are marked `create_safety: exists`. An exact title is only `possible_match`; vectors and loose keywords remain `unknown`. These labels concern record identity, not truth. The existing canonical content deduplication remains the write-time backstop. Similarity never authorizes merging or overwriting.
- `recency` accepts `auto` (default), `on`, or `off`. Auto recognizes a small English/Spanish current-state vocabulary. Commitments, assumptions and facts use 14-, 30- and 90-day half-lives for a relevance nudge capped at 15%; preferences, constraints and decisions are evergreen. Age uses explicit `valid_from`, falling back to creation time, so a maintenance write does not rejuvenate a fact. Age beyond two half-lives suggests revalidation, not deletion or falsity. These are initial engineering defaults, not calibrated probabilities.
- Context packing uses the same deterministic retrieval, preserves priority memories, and adds compact notices for relevant pending reviews. Full claims, evidence and revision history remain retrievable. No additional model or embedding request is required for ordinary retrieval or context packing.

Relations now bind both their schema version and the exact memory revision that supported them. Editing that evidence hides the edge until explicitly revalidated with `relation_save`. Migration 13 records the current revision as a baseline for existing edges; it cannot certify their past semantic correctness.

### Selective contradiction review

Incremental overlap discovery proposes same-scope, same-kind and same-entity pairs (including the general/no-entity group). It uses lexical overlap/title hints and can miss paraphrases and very short claims. Explicit non-overlapping validity intervals yield a deterministic temporal-change proposal, not proof that one fact supersedes another. Other ambiguous pairs can enter the background reviewer.

`automatic_review` defaults to true and has an independent checkbox in the memory page. The existing configured runtime model reviews at most one pair per worker cycle, with two attempts per exact revision pair, a 60-second reservation, a 45-second request timeout and at most 1,400 output tokens. The input is restricted to the two claims and bounded source excerpts. There is no new provider, mandatory reranker, or full-history review. This background feature consumes model tokens; turn it off to use local hints and conversational review only.

The reviewer must return `duplicate`, `complementary`, `conflicting`, `temporal_change`, or `ambiguous`, an explanation and exact quotes from both input claims. Ambiguous proposals require a clarification question. Results are version-fenced and never change claim status/content or automatically supersede anything. Relevant notices reach search and context packing; the conversational agent inspects sources and asks the owner when needed. A successful assessment is not rerun unless a claim changes. Failed attempts survive restarts and exhausted pairs remain available for manual review.

`memory_context reviews` exposes both original claims and the proposal. `knowledge_manage review_resolve` records complementary/duplicate/conflicting/temporal_change/ambiguous/dismissed; supply `first_revision` and `second_revision` from the reviewed pair so a delayed decision cannot resolve changed claims. It remains separate from explicit correction or withdrawal. Reviews are capped at 100 per call. Source uncertainty and reviewer judgments are not calibrated truth guarantees.

### Health and maintenance

`memory_context health` (also POST `/api/memory/knowledge` with `action: health`) reports scoped lexical gaps, pending embeddings for the configured endpoint/model, orphan entities, relations invalidated by evidence/schema changes, exhausted reviews and aging-memory candidates. Freshness inspection samples up to 500 confirmed records and returns at most 50 IDs, with coverage fields; it is not a full semantic audit.

The background worker repairs up to 50 derived lexical entries per cycle and checks one memory scope per cycle using a persisted round-robin cursor. Health reports are persisted per scope. Canonical memories, relations, history and orphan entities are not deleted or silently merged. Optional embedding indexing continues to process only changed eligible records through the existing worker.

### Reproducible evaluation

Run `cargo run -p fenix-storage --example memory_benchmark` to evaluate 26 fixed English/Spanish development queries in a temporary database. No production data, provider credentials, external service or model call is used. `memory_eval::MemoryEvalCase` and `Store::evaluate_memory_retrieval` support caller-supplied query/relevance corpora.

Both configurations receive the same corpus and queries: lexical baseline versus the production deterministic retrieval path. JSON output includes per-query IDs, precision@k, recall@k, MRR, nDCG, forbidden-memory hits, p95 retrieval latency and estimated serialized-result tokens (UTF-8 bytes/3, not billed tokens). Duplicate IDs cannot inflate metrics. Empty-ground-truth queries report correct abstention separately. This benchmark does not measure answer generation, review-model accuracy or live-user task success; embeddings are not part of this offline comparison. Expanded evidence may increase result tokens, so a recall gain is not automatically a cost reduction.

Recorded [development evaluation](../benchmarks/memory-retrieval-2026-09-21.json): recall@5 92.3% → 100.0%, nDCG@5 0.710 → 0.769, zero forbidden hits. Estimated result tokens averaged 158 → 393; evidence costs context. Latency is one local pass with baseline first, so cache/order effects prevent a speed claim. This is a development corpus, not held-out validation.
