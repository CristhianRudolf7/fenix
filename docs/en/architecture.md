# Architecture

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
| `fenix-storage` | Versioned SQLite schema, stable session ID, task queue, messages, events, FTS5 memory/history and delivery outbox. |
| `fenix-agent` | Model configuration, agent loop, context construction, permissions, tools, managed processes, schedules and optional Telegram adapter. |
| `apps/server` | Linux process entry point, loopback API, origin/host checks and embedded static assets. |
| `apps/web` | Framework-free browser ES modules, forms, chat and administration. No build server or per-agent browser is required. |

The server uses two Tokio worker threads. File-tool I/O runs on the blocking pool;
model calls, Telegram and managed commands use asynchronous I/O. There is one
active agent task in this version. More agents should reuse shared services and
HTTP pools, rather than require a process for every agent.

## Task lifecycle

1. Normalize and redact inbound text. Resolve an explicit textual control if it
   targets a pending request; otherwise persist a new task and user message.
2. Claim the oldest queued task and activate its message for model context.
3. Build instructions, relevant explicit memory, a prior summary and recent
   history. Record the request as the task objective. Instructions require the
   model to identify material assumptions visibly and verify its outcome.
4. Call the selected provider/model. Persist response IDs, usage and replay data.
5. Validate tool names, argument types, required fields and allowed values. Record
   each call before dispatch. Enforce any required approval in the runtime.
6. Execute tools serially, record their result/error and return bounded context to
   the model. Large bounded results remain recoverable through `task_inspect`.
7. Continue until a final response, cancellation, error or operational limit.
   Remaining managed processes are stopped. Failed/interrupted tool sequences
   remain in the audit history but are excluded from future model replay.

Incoming message order and model-context order are stored separately. If request B
arrives while A is running, B appears in the chat immediately but only enters model
context after A's tool results and response. This prevents a queued correction from
silently becoming part of the active turn.

The request, visible assumptions, responses and validation evidence form the basic
task record. This is not a formal proof that every natural-language requirement
has been satisfied; the stronger contract/evaluation harness remains future work.

## Approvals and commands

`command_policy` and `file_write_policy` default to `ask`. The runtime asks before
those effects even if the model omits `request_approval`. An explicit grant matches
the exact tool and argument object and is consumed once. Changing arguments needs
another approval. A denial is returned as an unexecuted action.

The owner replies with `aprobar <request_id>` / `rechazar <request_id>` or their
English equivalents. A clear yes/no also resolves the sole pending approval.
Clarifications use `responder <request_id> <answer>` or `reply ...`; unrelated text
queues. No silence, timeout or ambiguous text grants permission. `cancelar` stops
the active task; `cancelar <task_id>` can also cancel a queued task.

Dedicated file tools check canonical paths, reject escape paths and protect private
runtime data, environment files and Git internals. New files never overwrite
existing ones; edits need a unique exact match and are read back. Commands use an
explicit OS-accessible directory, including paths outside the file workspace.
They run as the service's Linux user, with bounded output, timeouts and process
group cleanup. **The file workspace is not a command sandbox.**

## Storage, memory and compaction

The private data directory contains `fenix.sqlite3`, a process lock, `instructions/`
and personal `skills/`. Database migrations are versioned under `fenix-storage`.
SQLite uses WAL, short mutex-protected operations and transactional queue/memory
writes. A process lock prevents two runtimes from dispatching the same installation.

Messages preserve provider-specific replay metadata. Browsers only receive visible
message text and references, never signed replay data or saved keys. The separate
memory table contains title, content, source and timestamps. Normalized exact
content prevents duplicate facts. FTS5 performs Unicode lexical retrieval with
escaped query tokens and ranked OR matches; no embeddings or knowledge graph.

`instructions/MEMORY.md` is a readable reference export. The database is authoritative;
edit memory through the web or tools. `AGENTS.md`, `SOUL.md`, `IDENTITY.md` and
`USER.md` are editable through settings and loaded with bounded size for new tasks.
Default skills are copied without overwriting existing personal instructions.

At a task boundary, history above approximately 64 KB triggers an attempt to
summarize older complete tasks while retaining recent tasks. A valid completed
summary advances a context cursor; original messages are never replaced. The
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
| Active agent tasks | 1; FIFO backlog remains durable during normal operation |
| Model steps | 32 per task, at most 16 tool calls per response |
| Inference retries | Up to 2 additional attempts for classified transient errors; no automatic tool-effect replay |
| Agent execution | 1 hour maximum; pending owner question times out after 30 minutes |
| Provider request | 120-second default HTTP timeout; 4,096 output tokens per agent call, 1,024 for setup |
| Active agent request | 512 KB serialized; no cumulative token-budget setting |
| Tool arguments / context result | 128 KB arguments; results above 16 KB become references with previews |
| Regular UTF-8 file | Up to 1 MiB; file search and listing also bound traversal/results |
| Managed processes | At most 8 running; 1–600 seconds per command, default 120 |
| Process output | Up to 128 KiB combined stdout/stderr; truncation reported |
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
