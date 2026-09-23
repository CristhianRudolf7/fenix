# Persistent project contracts

Fénix keeps agreed behavior across tasks without adding a second planner or a mandatory
document workflow. `project_spec` is an on-demand tool backed by SQLite migration 11.
It works for software and other project types; general chat has no implicit contract.

- A requirement has a stable ID, statement, revision, source and author task. The
  project contract has a monotonically increasing version. Removed IDs retain
  tombstones so deleting and recreating the same text does not hide intervening edits.
- `get` reads requirements in pages of 20 and returns `version` and `next_offset`.
  `changes` lists the history; `get` with `change_id` returns complete before/after
  deltas and current conflicts. Reads respect the task's fixed project scope.
- `propose` requires `expected_version`, a reason and `deltas` with `requirement_id`,
  `action` (`add`, `modify`, `remove`), `source` and replacement `statement` for
  additions/modifications. A pending proposal never changes the current contract.
  Repeated identical pending proposals by the same task reuse the same change ID.
- Only a running master in the writable project may `apply` or `discard`, supplying
  `change_id`, the current `expected_version` and a sourced reason. Workers propose;
  reviewers/auditors only read. Coordination/report turns cannot amend contracts.
- Consolidation is transactional and retains the full change and resolution history.
  Two proposals touching the same ID require explicit reconciliation. Unrelated
  proposals survive other updates: reread the version and consolidate if each touched
  requirement still matches its original revision. A stale update never wins silently.
  Events are published transactionally through the existing event bus.

## Connection to worktrees and audits

Workers should declare intended persistent behavior changes before editing. Before
integrating a worker's Git changes, the engine checks its declared amendments for
overlap, changed requirements or discarded proposals. Conflicts block integration
and preserve files. Contract mutations and Git preflight share the existing planner
lock; no additional model call performs these checks. Later explicit amendments from
the same task supersede its older deltas for the same IDs.

This is conservative ID/revision conflict detection, **not semantic understanding**.
Two different IDs can express contradictory behavior; undeclared amendments are not
detected. It cannot undo an already completed merge. The master must reconcile those
cases using the existing clarification, plan revision and selective audit mechanisms.
Non-Git tasks still get version/conflict checks when consolidating the contract.

The contract describes **agreed behavior, not verified implementation**. Applying a
proposal never completes tasks, grants permissions, bypasses an audit or proves the
code implements it. Original owner requests and amendments remain the authority for
acceptance; speculative requirements stay pending until clarified. Structural checks
do not prove that a supplied source is genuine user consent.

Contexts include only contract version/count and retrieval instructions, not the full
catalog. Requirements load on demand. Small tasks that do not change project behavior
need no proposal. There is no new UI, external dependency, embedding or automatic
reviewer. Documentation export and semantic conflict detection are not implemented.

See [planning](planning.md), [completion audits](completion-audits.md), and
[parallel worktrees](parallel-worktrees.md). [Español](../es/project-contracts.md).
