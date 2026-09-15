# Editable planning and asynchronous conversation

[Español](../es/planning.md)

Fénix keeps one visible conversation while independent turns, a planner and
milestone workers run as asynchronous Rust tasks. A slow model call or an unanswered
question does not hold the conversation. The assistant is instructed to surface
**every detected unresolved doubt**, including minor ones, and reuse explicit
answers. Detecting an ambiguity remains a model responsibility; the engine
persists and routes the questions it receives.

## Plans and skills

For long work, the master calls `plan_manage create` with an objective and a compact
array of milestones. Each has `id`, `objective`, `expected_output`, optional
`depends_on` IDs and suggested `skills`. SQLite rejects duplicate IDs, missing
dependencies and cycles. A plan has one fixed general/project scope. Cross-project
coordination retains the existing explicit delegation rules.

Ready milestones run through the existing subagent harness. Dependent workers
receive predecessor results, source IDs and relevant prior effects. They may inspect
journals of predecessor milestones and older generations of their own milestone;
independent branches and other projects are excluded. Each worker sees the full
skill metadata catalog and loads enabled bodies on demand. There is one delegation
level and at most three attempts for transient provider failures.

A deterministic scheduler dispatches work and consolidates completion/blockers;
it does not call a model to poll, deliver a notification or merge milestone state.
The model still consumes tokens when drafting or interpreting an edit.

## Edit at any point

Describe a change in the chat; the master can call `plan_manage steer` or
`get` followed by `revise`. An explicit control also works while a model request
is still pending:

```text
replanificar <task-or-plan-id> Change the export format and keep the completed analysis
```

When replying to a task message in the web or Telegram, use
`replanificar <change>`. Drafting-task edits preserve the same task ID and cancel
the superseded inference; the replacement context includes updated instructions
and saved evidence. A revision of an existing plan gets a separate coordinating
turn in the same conversation, with the original project's scope and delivery
channel. It takes priority over ordinary queued work.

A natural-language edit pauses pending work before interpretation. Completed
milestones survive. A concrete graph revision invalidates changed milestones and
their dependants, while unchanged work can continue. `expected_version` rejects
stale edits; a newer natural-language edit supersedes an earlier editing turn.
Old questions cannot revive superseded work. Worker generations are checked at
dispatch, after inference and before effects/results are accepted. File/process
completion barriers settle managed effects before a revision is reported applied.
Already completed or externally committed effects are not undone. Earlier worker
journals remain available for inspection and any subsequent correction.

Inspect the plan and its revisions in **Tasks → Plans**. Cancellation remains
textual: `cancelar <plan-id>` stops the plan; a child task ID cancels that child.
Questions use `responder <request-id> <answer>` or native replies. Approvals keep
their exact-action IDs and rules. The API also exposes `GET /api/plans`,
`GET /api/plans/{id}` and `POST /api/tasks/{id}/replan` with `{"text":"..."}`.

## Publication and delivery

Migration 4 adds versioned plans, worker generations, saved questions, a SQLite
event log and subscriber cursors. Events carry stable IDs and task/session/project,
plan/version and destination metadata when applicable. Message history and its
publication are written in one transaction. Wakeups are hints, not the source
of truth. Subscribers acknowledge only after their work is persisted; replay is
idempotent internally.

The web subscribes to `GET /api/events`. SSE event IDs and `Last-Event-ID` (or
`?after=`) resume delivery after reconnection. Events refresh durable history and
state without rerunning inference. OpenAI text previews keep their separate,
replaceable `/api/progress` stream; preview tokens are not stored as bus events.

Telegram output is queued for the task's captured bot and owner. A project switch,
a reply from the other linked channel or changed Telegram configuration does not
retarget those queued results. Failed deliveries remain pending with bounded
exponential backoff. Telegram transport uncertainty can still duplicate an
external message; internal deduplication does not guarantee exactly-once sending.

## Operational limits and restart

Plans also support [adaptive review](plan-reviews.md): initial strategy review for
complex work, independent specialists on demand and one evidence-backed response.
Reviews share the configured model and background worker pool.

Normal master dispatch has eight active slots, with a reserved control path;
64 ordinary live turns may wait, plus a control turn. Subagent work is bounded to
32 live jobs and four simultaneous inference requests. Questions release dispatch
capacity; waiting for an owner answer does not consume active execution time.
Execution defaults to four hours for master tasks, one hour per child attempt,
and 128 model steps per task across retries and follow-ups. These limits are
configurable in the execution settings; they do not add a cumulative token budget.

Unfinished tasks, questions and plans are marked interrupted after restart.
Persisted events and undelivered messages remain recoverable, but effects do not
resume automatically. Explicitly revise a saved plan after inspecting its earlier
evidence. Original conversation and audit data remain stored. Concurrent model
contexts exclude unfinished transcripts from other tasks; completion seals each
task's tool-call/result groups together.

Other active tasks are supplied as compact references filtered by authorized scope,
so natural-language edits can address a task still drafting without importing its
uncompleted tool transcript.
