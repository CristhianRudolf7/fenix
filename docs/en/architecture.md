# Architecture

[Interactive execution, updated limits and on-demand tools](execution-harness.md)

[Español](../es/architecture.md)

Fénix is a modular Rust workspace, deployed initially as one local process. The
browser uses the loopback HTTP API; Telegram feeds the same runtime. Neither
channel owns a separate conversation or agent loop.

## Boundaries

| Component | Responsibility |
| --- | --- |
| `fenix-core` | Serializable conversation contracts, tool calls/results, reasoning effort and provider IDs. |
| `fenix-credentials` | Redacted API-key values and exact default/project resolution. A missing project key never falls back. |
| `fenix-providers` | Reusable HTTP clients, five protocol codecs, native authentication headers and bounded inference transport. |
| `fenix-storage` | Versioned SQLite schema, projects, fixed task bindings, scoped history/memory, conversation queue, child jobs/reports, events, FTS5 and delivery outbox. |
| `fenix-agent` | Master and background child loops, context, permissions, effect coordination, tools, processes, schedules and Telegram. |
| `fenix-web` | Independent public crawler, disk-backed FTS5 index, bounded HTTP extraction, robots/DNS checks and source reading; [contract and budgets](web-search.md). |
| `apps/server` | Linux process entry point, loopback API, origin/host checks and embedded static assets. |
| `apps/web` | Framework-free browser ES modules, forms, chat and administration. No build server or per-agent browser is required. |

The server uses two Tokio worker threads. File-tool I/O runs on the blocking pool;
model calls, Telegram and managed commands use asynchronous I/O. The master serves
concurrent conversation turns while children run as Tokio tasks in the
same process, reusing the configured provider client and HTTP connections. No
additional process, browser or model configuration is created per child.

## Task lifecycle

1. Normalize and redact inbound text. Resolve an explicit textual control if it
   targets a pending request; otherwise persist a new task and user message.
2. Claim the oldest queued task and activate its message for model context.
3. Resolve the destination before project context or execution tools are available.
   Bind readable scopes, workspace and context version to the task. Build instructions,
   applicable memory, a scope-specific summary and history. Record the request as
   the task objective and preserve material assumptions and outcome evidence.
4. Call the selected provider/model. Persist response IDs, usage and replay data.
5. Validate tool names, argument types, required fields and allowed values. Record
   each call before dispatch. Enforce any required approval in the runtime.
6. Execute tools serially, record their result/error and return bounded context to
   the model. Large bounded results remain recoverable through `task_inspect`.
7. Continue until a final response, cancellation, error or operational limit.
   Remaining foreground processes are stopped; explicit persistent sessions may survive success. Failed/interrupted tool sequences
   remain in the audit history but are excluded from future model replay.

Incoming message order and model-context order are separate. Independent turns can
run concurrently; each context excludes other unfinished transcripts. Completed
tasks are sealed as contiguous tool-call/result groups. Textual plan controls can
interrupt drafting or revise active work without changing task/project scope.
See [editable planning and durable events](planning.md).

The request, visible assumptions, responses and validation evidence form the basic
task record. This is not a formal proof that every natural-language requirement
has been satisfied; the stronger contract/evaluation harness remains future work.

## Projects and general conversation

One visible chat can contain general discussion and work on several projects.
`projects.rs` resolves intent before admitting project context. Clear references
use rules; ambiguous or compound requests use a bounded JSON routing request to
the configured model. Model routing consumes tokens and can fail; unresolved
selection requires a textual clarification before execution tools are used.

`project_tools.rs` enforces the resolved destination and read scopes, independently
of tool descriptions. Project history and memories are filtered before retrieval.
Dedicated file tools use the task's captured workspace. Coordination can consult
explicitly requested project records but delegates execution to children with a
specific destination. Report turns can inspect evidence and notify the owner,
without starting replacement work or new effects.

General conversation has its own history and summary. Project master turns also
receive a bounded copy of recent general user/assistant text, not tool transcripts
or other projects' detailed history. Project briefs and relevant task state supply
project continuity; related task references preserve their source IDs. Global
preferences remain explicit and project facts cannot be promoted by a scoped write.

Bindings survive topic switches and identify late reports and pending approvals.
Reply references are captured when the message arrives; the current foreground
project is a hint, not a mutable workspace for running tasks. A background report
does not change foreground focus. All original messages remain in the same visible
chat. Legacy records stay general during migration; no historical project ownership
is guessed. See [Projects in one conversation](projects.md) for API and UI details.

## Background delegation

`subagent_spawn` persists a precise objective and expected output, with optional
context and constraints, then returns a child ID immediately. The master can
continue and serve subsequent conversation requests while the child runs or waits
for approval. One delegation level is enforced in both tool dispatch and storage:
children cannot create children. Tool catalogs vary by role and scope: children
exclude conversation-history and delegation controls and require `subagent_finish`.
Coordination and report turns have further restrictions. Child task inspection is
limited to its own ID; master inspection must respect its resolved read scopes.

A separate worker claims child jobs independently of the conversation dispatcher. SQLite stores
parent links, attempt counts, isolated transcripts and completion reports. A child
starts with its scoped handoff, project brief, stable instructions, applicable
project/general explicit memories and bounded task-local skill candidates. It does not receive
the main conversation transcript. Memory tools retain the same scope checks. Finished reports are
persisted and queued back to the master for integration; detailed tool evidence
remains in the child's task journal.

Retryable failures allow at most three total child attempts, preserving the
transcript and completed tool results without replaying actions. Each attempt is
limited to 128 steps per task and one hour per attempt by default. A structured `subagent_finish` is required;
blockage, cancellation or non-retryable failure can stop earlier. Restarting marks
unfinished children interrupted instead of automatically restoring execution.

`GET /api/status` exposes child state and a `pending_requests` collection alongside
the legacy `pending` field. The Tasks panel refreshes IDs, purpose, expected output,
attempts and results through durable event-triggered refresh plus polling. Only master previews use
the single SSE progress snapshot, so children cannot replace its live response.

## Approvals and commands

`command_policy` and `file_write_policy` default to `ask`. The runtime asks before
those effects even if the model omits `request_approval`. An explicit grant matches
the exact tool and argument object and is consumed once. Changing arguments needs
another approval. A denial is returned as an unexecuted action.

New workspace files and directories created by `file_write`, `file_patch`, library
exports and file-history restores follow the process umask like ordinary tools
(typically 0644/0755), so services, tests or verifiers running as another local user
can read them. Editing preserves existing modes. Fénix's own runtime and browser
files remain owner-only.

The owner replies with `aprobar <request_id>` / `rechazar <request_id>` or their
English equivalents. A clear yes/no also resolves the sole pending approval.
Clarifications use `responder <request_id> <answer>` or `reply ...`; unrelated text
queues. Multiple requests may be pending across children; each has its own ID and
resolves independently. No silence, timeout or ambiguous text grants permission.
`cancelar` stops the active master task; `cancelar <task_id>` can also cancel queued
work or a child. A waiting child does not pause the master worker.

Dedicated file tools check canonical paths, reject escape paths and protect private
runtime data, environment files and Git internals. New files never overwrite
existing ones; edits need a unique exact match and are read back. Commands use an
explicit OS-accessible directory, including paths outside the file workspace.
They run as the service's Linux user, with bounded output, timeouts and process
group cleanup. **The file workspace is not a command sandbox.**

A runtime effect coordinator serializes file, memory and schedule changes and holds
an exclusive lease for the full lifetime of each managed command. Read operations
and model calls can proceed in parallel. This protects cooperating tasks in this
process; it is not a sandbox or a lock shared with external processes or separate
Fénix installations. Configuration cannot change model or permission policies
while master or child work remains active.

## Storage, memory and compaction

The private data directory contains `fenix.sqlite3`, a process lock and `instructions/`.
Manual skill packages default to `~/.fenix/skills`, shared independently of data
directories; `--skills-dir` overrides this. Enablement stays in each runtime database.
Database migrations are versioned under `fenix-storage`.
SQLite uses WAL, short mutex-protected operations and transactional queue/memory
writes. A process lock prevents two runtimes from dispatching the same installation.

Messages preserve provider-specific replay metadata. Browsers only receive visible
message text and references, never signed replay data or saved keys. The separate
memory table contains title, content, source, project/task references and timestamps.
Normalized exact content prevents duplicate facts within the same scope. FTS5 performs Unicode lexical retrieval with
escaped query tokens and ranked OR matches; no embeddings or knowledge graph.

`instructions/MEMORY.md` is a readable reference export. The database is authoritative;
edit memory through the web or tools. `AGENTS.md`, `SOUL.md`, `IDENTITY.md` and
`USER.md` are editable through settings and loaded with bounded size for new tasks.
Default skills are copied without overwriting existing personal instructions.
Startup first copies legacy `<data-dir>/skills` resources into the selected library,
skipping symlinks and existing destinations and retaining originals. Migration has
file, total-byte, entry and depth limits; see [local operation](local-operation.md).

Master and children discover the full library through `skill_search` and paginated
`skill_list`; only up to two task-local candidates enter the initial context.
Descriptions are bounded to 512 bytes and versions to 80. `skill_load` reads selected
enabled instructions or linked references up to 32,000 bytes. A persisted incremental
lexical index keeps unchanged entrypoints from being reread and parsed. See
[scalable skills and native documents](skills-and-documents.md).

At a task boundary, history for the current scope above approximately 64 KB
triggers an attempt to summarize older complete tasks while retaining recent tasks.
General and each project use distinct summary records/cursors; coordination uses
its own task context. A valid summary advances only that scope's cursor; original
messages are never replaced. The
summary uses Objective; Constraints and preferences; Progress (Completed, In
progress, Blocked); Key decisions; Relevant files; Next steps; Critical context.
Incomplete summaries do not advance the cursor. A very large active task still
hits the explicit request-size limit and may need a smaller follow-up request.

## Scheduling and Telegram

The scheduler is independent of model inference. It accepts once, interval and
cron triggers, calculates dates in `America/Lima`, persists the next execution and
advances it before dispatching effects. Reminders append text directly; scheduled
work enqueues a task. Active identical schedules and concurrent executions of one
schedule are rejected. Missed executions are skipped after restart; ongoing and
queued tasks from a previous process are marked interrupted, never resumed.

Telegram uses `getMe`, `getUpdates` and `sendMessage`. A locally displayed random
pairing code binds one owner; groups, other senders and non-text payloads cannot
enqueue work. Incoming update offsets prevent normal redelivery. The outgoing
outbox records bounded delivery attempts. Telegram delivery is not guaranteed
exactly once: network ambiguity or a crash after remote acceptance can duplicate
an outgoing message. Failed deliveries remain in the database for inspection.

## Initial operational bounds

| Resource | Bound |
| --- | --- |
| Active agent tasks | 8 normal master dispatch slots, reserved control capacity, 64 ordinary live turns + 1 control; 32 live child jobs / 4 child inferences |
| Model steps | 128 per task by default, configurable up to 4,096; at most 16 tool calls per response |
| Master inference retries | Up to 2 additional attempts for classified transient errors; no automatic tool-effect replay |
| Child attempts | At most 3 total for retryable failure, preserving transcript; 128 steps per task / 1 hour per attempt by default |
| Master execution | 4 hours by default, configurable; owner response time is excluded for master and child deadlines |
| Provider request | 120-second default HTTP timeout; up to 16,384 output tokens per agent call by default, bounded by remaining context; 1,024 for setup |
| Active agent request | 512 KB serialized; no cumulative token-budget setting |
| Tool arguments / context result | 128 KB arguments; results above 16 KB become references with previews |
| Regular UTF-8 file | Up to 1 MiB; file search and listing also bound traversal/results |
| Managed processes | Foreground effects serialized; explicit persistent sessions may overlap later work; cap 16; command default 30 minutes, service default 24 hours |
| Process output | Redacted logs up to 64 MiB per process; 12,000-byte incremental previews and recoverable pages; output failures stop the process explicitly |
| Skills / instructions | 32 KB per skill; 8 KB per stable instruction file |

Relevant primary references: [Axum](https://docs.rs/axum/0.8.9/axum/),
[rusqlite](https://docs.rs/rusqlite/0.37.0/rusqlite/),
[cron](https://docs.rs/cron/0.15.0/cron/),
[Telegram Bot API](https://core.telegram.org/bots/api), and
[GPT-5.6 Luna](https://developers.openai.com/api/docs/models/gpt-5.6-luna).

## Live progress and response previews

`GET /api/progress` is a loopback-protected Server-Sent Events endpoint. A Tokio
watch channel retains one replaceable snapshot, containing task/response IDs,
a phase, a known tool name and at most 32 KB of provisional text. Slow clients
receive the current snapshot, without an unbounded per-client token queue.
The browser reconnects independently of the worker; reconnecting never submits
or repeats a model request. Normal polling still refreshes durable history,
approvals and queued tasks.

`ProviderClient::complete_with_preview` enables SSE for OpenAI Responses. It
accepts visible text deltas but obtains tool calls, usage and opaque reasoning
replay data from the terminal response. Other adapters retain their buffered
transport, and a gateway returning JSON is decoded without a second request.
Adding streaming to the other four protocols is future work.

The runtime publishes previews at most once per 100 ms, holding the unfinished
last token and known-secret prefixes before redaction. Raw previews never enter
the database, model context or Telegram outbox. Only a terminal response is persisted; incomplete terminal responses still
fail the task and never authorize tools. Tools execute after decoding that response, never from partial JSON.
EOF without a terminal event fails the task. Transport failures after text deltas
do not trigger automatic inference retries; cancellation closes local I/O and
clears the preview. Provider-side billing may already have occurred.

Tool progress comes from runtime events and uses no additional model calls.
The UI distinguishes preparing context, generating, using a tool, waiting for a
textual answer, retrying and reconnecting. The provisional bubble is replaced by
the durable message. No verification-contract feature is introduced by this work.

Protocol reference: [OpenAI streaming responses](https://developers.openai.com/api/docs/guides/streaming-responses).

## Document evidence

The private [document library](document-library.md) stores versioned originals and searchable evidence separately from memory. Native extraction, isolated PDF/OCR jobs, OpenAI `text-embedding-3-small`, scoped hybrid retrieval and source citations share the existing account runtime and conversation.
