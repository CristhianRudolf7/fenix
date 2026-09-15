# Interactive execution and on-demand tools

[Español](../es/execution-harness.md)

The Rust runtime supports interactive processes, persistent services, incremental
redacted logs, reusable subagents and deterministic tool discovery. These features
are shared by the web chat and Telegram; no separate user conversation or terminal
interface is required.

## Interactive processes

`command_run` accepts `command`, `cwd`, optional `tty`, `persistent` and
`timeout_seconds`. Pipes now expose stdin. `tty: true` uses a real controlling PTY
through the Rust `pty-process` library, initially 24 rows by 100 columns.
`command_input` sends text, can resize a PTY with `rows`/`columns`, or send EOF.
For a pipe, EOF closes stdin; for a terminal it sends Ctrl-D. Control characters
such as Ctrl-C can be sent as text. Each input follows the same exact textual
approval policy as a command. A partial/timed-out write is not automatically retried.

Foreground commands must finish or be cancelled before a successful task ends.
An explicit `persistent: true` starts a service/session that survives successful
turn completion. Use `command_list` to find it and `command_status`,
`command_input` or `command_cancel` from a later authorized master turn. Subagents
may control only their own processes; master access also checks project scope.
Persistent processes stop on explicit cancellation, their lifetime limit, task
failure/cancellation while their owner is active, or runtime shutdown. They do
not restart after a service restart; their saved logs remain readable by ID.

Foreground commands retain the existing effect lease. Persistent sessions release
it after launch and intentionally overlap later work; they are not isolated
worktrees or sandboxes. Their commands retain the owner's OS permissions. Inspect
readiness before using a server and avoid duplicate services. At most 16 managed
processes may run simultaneously. PTY session members and the original process
group are terminated during cleanup; a program deliberately escaping the session
is outside this process manager's isolation guarantees.

## Output and recovery

`command_status` returns new output by default, with a cursor per reading task.
An explicit `cursor` replays a byte position. `next_cursor` is the next position.
`wait_ms` waits at most 30 seconds, and `limit` bounds the returned bytes to 12,000.
Large unread output returns its beginning and end, with `omitted_bytes` and a
reference to `command_output`. Repeated empty polling should be avoided.

`command_output` reads the stored log using byte `offset`/`limit`, returning
`next_offset` and `has_more`; it does not change the incremental status cursor.
Later master turns can read foreground logs within their authorized scope without
gaining control of those processes. Saved logs also report `log_complete` and
`output_error`; an interrupted run is marked incomplete after restart.
Logs are redacted before writing to private files under `DATA_DIR/processes`.
Output capture retains incomplete secret fragments across reads. Oversized
unfinished tokens are replaced and marked incomplete; invalid UTF-8 is decoded
lossily. Logs combine stdout/stderr in observed arrival order. A 64 MiB log limit
or a write failure stops the process with an explicit error rather than silently
losing its final output. Persistent logs currently remain until the owner removes
the installation's data. They are not filesystem snapshots or rollback records.

## Execution settings

Configure these under **Settings → Advanced → Execution limits**. Existing
settings without `execution` receive these defaults:

| Setting | Default |
| --- | ---: |
| Model steps per task | 128 |
| Master active execution time | 4 hours |
| Subagent attempt time | 1 hour |
| Foreground command timeout | 30 minutes |
| Persistent session lifetime | 24 hours |
| Maximum model output tokens | 16,384 |

Time limits can be configured up to seven days; model steps up to 4,096 and output
up to 131,072 tokens. These are bounds, not a guarantee that a provider supports
that output size. The configured context capacity and remaining estimated space
further cap output per request, reserving 1,024 tokens of headroom. Waiting for
human answers does not consume active execution time. Steps persist across
replanning, compaction and subagent follow-ups; none silently reset the counter.
The existing repetition guard and three-attempt transient-provider retry limit
remain. This update does not implement automatic crash recovery or a spend budget.

## Subagent follow-ups

`subagent_send` accepts `subagent_id` and a concrete `message`. Only an active
master can send it. The SQLite inbox records source and order and deduplicates an
identical message from the same sender. Delivery and the updated worker context
are saved atomically at a complete tool boundary. A message arriving during
inference suppresses remaining stale tool actions; already completed effects are
retained. A waiting user question remains pending until it is answered/cancelled.

A completed or blocked standalone worker can continue with the same ID, context,
project, purpose and retry/step counters. Failed workers can continue only while
attempts remain. Cancelled/interrupted workers and workers with exhausted attempts
cannot reopen. A follow-up is not a way to reset retries. There is a limit of 100
follow-ups per worker. Reopened completions produce a new report while preserving
old report history. The master remains available throughout.

Plan reviewers cannot receive side-channel messages. A completed plan milestone
must be changed through plan revision; running milestone workers can receive
clarifications but must retain their assigned objective. Delegation remains one
level deep.

## Tool discovery

Each task begins with essential schemas for reading/searching files, starting and
checking commands, asking questions and finding tools. Workers also receive their
finish tool; assigned reviewers receive their evidence reader. The compact
`tool_search` description lists other permitted names.

`tool_search` accepts a purpose in `query`, exact `names`, or neither for a compact
catalog. It ranks the authorized catalog with local BM25 and English/Spanish
keywords, with no embeddings or model calls. Up to eight selected schemas load
for the next inference; at most 24 nonessential schemas remain loaded. Older tools
remain discoverable. Loading a schema never expands role/project permissions;
every call is still checked by the runtime. A known permitted tool call can be
validated even if the model recalled its name without searching first.

Skills keep their independent on-demand `skill_load` workflow. This change does
not add MCP connections or a JavaScript tool-composition engine.
