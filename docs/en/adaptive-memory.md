# Adaptive memory

Fénix stores memory in SQLite (schema version 7). This is the canonical store; `instructions/MEMORY.md` is an export of up to 200 active records, not a second writable database. Existing memories migrate as confirmed explicit facts without losing their IDs or project bindings. Historical versions start at migration; earlier overwritten content cannot be reconstructed retroactively.

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

Every cycle excludes expired facts, incrementally proposes lexical overlap pairs for review, and optionally updates embeddings. Overlap is a hint, not proof of duplication/contradiction; review can mark complementary/duplicate/conflicting/dismissed, while actual correction/withdrawal is explicit. No automatic destructive merging. Entity cards omit invalid evidence, and graph references are protected by foreign keys. The worker reports failed jobs through task events and maintenance status; only memory jobs resume after service restart, not interrupted task effects.

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
