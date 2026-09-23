# Context, tool programs, recovery and procedural learning

[Español](../es/harness-continuity.md)

These additions reuse the account runtime, project bindings, tool permissions and SQLite journal. They do not require GitHub or a new model provider. They are local implementation changes, not a production deployment or a measured benchmark improvement.

## Context accounting and stable prefixes

The runtime stores the provider's reported input usage with a SHA-256 fingerprint of the complete priced message prefix, model/provider/endpoint identity and tool schemas. Subsequent estimates add only newly appended messages. Missing usage, edited history, compaction, different tools or a changed model invalidate that reference and use the existing conservative byte estimate. Output reservation remains independent. Transient operational data is conservatively retained in the usage reference, so estimates can overcount; this is not a billing estimator.

The master's fixed instructions are separated from changing task data. Operational snapshots are appended to the request tail instead of inserted near its beginning. Provider caching remains provider-dependent; no cache hit, percentage saving or performance gain is guaranteed.

## Tool programs

`tool_program` composes the existing workflow tool allowlist in Rust, without a model call between steps. It is a declarative program, not an arbitrary Python/JavaScript interpreter. Each step has `id`, `tool`, `arguments` and optional `foreach`, `when` and `select`. `output` chooses the final result. Data references use the existing workflow `{ "$ref": "step_id/JSON/pointer" }` convention. During a loop, `item` refers to its current element. A step's `select` projects each result before accumulation. Conditions use `reference`, `operator`, `value` as in workflows.

```json
{
  "steps": [
    {
      "id": "read",
      "tool": "file_read",
      "arguments": {"path": {"$ref": "item"}},
      "foreach": ["notes/a.md", "notes/b.md"],
      "select": {"$ref": "read/content"}
    }
  ],
  "output": {"documents": {"$ref": "read"}}
}
```

Programs permit at most 50 calls, reject recursion/delegation and persistent/interactive commands, and retain normal account/project/profile checks and per-action approval. Remembered denials cannot be bypassed. Newly discovered repository instructions pause writes and are delivered to the model. Failed steps stop the program without retrying completed effects. Invalid later references can fail after earlier effects, so programs are **not transactions**.

Intermediate full redacted results are journaled under unique call IDs and remain searchable through `task_inspect`. Only selected results return to the model, with a recoverable preview above 24 KB. A selected loop result is capped at 2 MB. This reduces model round trips for mechanical work; inference is still needed to author the program or handle ambiguous outcomes.

## Restart recovery

Before inference, the engine persists a complete message/tool-result boundary. It invalidates that boundary before publishing or executing a generated response. Recovery preserves task and session IDs, fixed scope, model-step counters, completed effects and subagent context. It can admit fresh queued tasks and interrupted inference at a safe boundary. Restart admissions are capped at three per task to avoid a crash loop.

Active managed processes, unresolved write failures, incomplete tool batches, pending approval/questions and independent review workers are not automatically replayed. Plans resume only when their prior state was running and all their active tasks qualify; other plans remain interrupted for explicit review/replanning. Previously completed milestones remain recorded. This does not restore terminal processes, browser downloads, sockets or external transactions. A model continuing after restart must inspect mutable state before dependent writes; the runtime never replays the saved tool calls.

If a crash occurred between a write and its durable receipt, the outcome is uncertain. Inspect the journal and actual destination before creating a continuation. Checkpoints cannot establish whether an external API committed an operation whose response was lost.

## Private file history

`file_history` provides `list`, `inspect` and `restore`. `task_id` selects an authorized task (current by default); `id` selects one file version. List pages contain at most 50 records and support `offset`. Inspect provides bounded before/after previews and hashes. Restore without an ID undoes the selected task's recorded file changes; with an ID it restores that version's before-image.

Before managed file writes, edits, patches, native document edits/creation and library exports, Fénix stores the original bytes in a private content-addressed store under the account's data directory. SQLite records task/path/workspace, before/after hashes, permissions and completion state. This supplies version-history functionality **without creating a Git repository** or touching the user's `.git`. Identical byte contents share one blob.

Restoration requires the same writable project/workspace and master authorization. It compares current bytes against the selected after-image and refuses later changes. Expected write receipts are checked against captured images so an intervening external edit is not silently attributed to the agent. All selected paths are preflighted before mutation; each write rechecks expected content, and an undo has its own before-images. Multi-file restoration is not atomic: cancellation or I/O failure may leave a documented partial restore.

Automatic history covers managed file tools, not arbitrary shell modifications, connector downloads, design-specific writers or an entire machine. Use the existing encrypted `workspace_checkpoint` for supported host-managed workspace snapshots. History retains exact bytes, including binary documents; secrets in preview output are redacted, but private originals remain on disk. Symlinks, special files, credential paths and runtime internals remain excluded by existing file policies. Initial limits are 32 MiB per file and 512 MiB of stored blobs; exhaustion fails before new managed writes when the before-image cannot be saved. There is no automatic history deletion.

## Selective skill learning

Successful nontrivial master tasks can enqueue a durable learning review. Greetings and simple tasks do not: eligibility requires at least three substantive tool receipts with a failed attempt, or eight substantive receipts. This is an engineering heuristic, not proof that a lesson exists. Each task queues at most once; identical previously reviewed candidate content is skipped.

A background worker runs when the master has no active turn. It uses the configured model, at most two review attempts, a 90-second attempt deadline and two bounded model requests: draft a reusable procedure, then check it against original evidence, existing skill coverage and at least two regression examples. Usage is recorded in task events. The review can still be wrong and is **not an executed benchmark**.

`skill_learning list/inspect` exposes candidates and their sources. Accepted candidates **activate automatically without owner approval**. The background worker searches a bounded shortlist of existing skills before drafting, asks the reviewer to reject redundant procedures, and rejects exact procedure duplicates even when their name/frontmatter differ. This is not exhaustive semantic deduplication of the entire library.

New skills are private copies, never automatic marketplace publications. Only unchanged, enabled, unpinned **previously learned** skills may be rewritten automatically. Bundled/manual skills are read-only references; a changed original blocks activation rather than triggering another model review. Updates retain support files, existing resource references, category and required-tool metadata. The review is semantic, not an executed regression test or proof that all behavior is preserved.

Every activation records its before/after content and previous version in a durable journal before writing. Interrupted applications reconcile the actual content instead of overwriting divergent files. `activate` retries a validated blocked candidate without an approval request. `rollback id` restores the previous revision, or disables a newly created skill without deleting it. It refuses newer revisions and owner edits. Revisions created before this journal was introduced cannot be rolled back through this new action.

Deterministic maintenance runs at most daily while learning is enabled and the master, subagents and workflows are idle, without a model request. Unprotected learned skills become stale after 30 days and are disabled after 90 days without use. Pinned, owner-modified and schedule/workflow-referenced skills (including paused schedules) are protected. Reference protection uses literal names, so indirect natural-language dependencies may not be detected. `maintenance` reports state; `pin`, `archive` and `unarchive` provide manual control. Unarchiving starts a new grace period. No files are automatically deleted or packages automatically merged.

`reject` discards an inactive draft. `configure enabled:false` stops future background reviews, activation and maintenance; active skills remain installed and rollback remains available. Pending changes and blocked candidates are inspectable, and successful activations/restorations are recorded in task events.

## Validation scope

Regression tests cover usage invalidation, program references/loops and denied actions, recovery admission/counters, incomplete writes, binary restoration and later owner edits, repository guidance, review deduplication and mocked candidate review. They do not establish a live-model quality increase, a cost reduction, or crash recovery for arbitrary external side effects.
