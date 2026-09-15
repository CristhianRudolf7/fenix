# Repository-aware coding harness

[Interactive execution, updated limits and on-demand tools](execution-harness.md)

[Español](../es/coding-harness.md)

The master and its one-level workers share repository search, exact-context
patches, scoped repository guidance, language intelligence, and active-task
compaction. These capabilities preserve the single user conversation. Configure
language servers and context limits in **Settings → Advanced → Coding tools and
context**. Existing installations receive conservative context defaults and no
language servers; no provider migration is required.

## Search and patches

`files_search` keeps its literal, case-insensitive default. Set `regex: true`,
`case_sensitive: true`, a `glob` such as `**/*.rs`, an `exclude` glob, and a result
`limit` when needed. `files_glob` locates paths. Searches respect `.gitignore` and
`.ignore`, exclude private/runtime/generated paths, and do not follow symlinks.
Results disclose truncation and skipped errors. Limits are 500 returned matches,
20,000 visited entries, depth 32, 64 MiB scanned text and 1 MiB per UTF-8 file.
A limited or empty search is not proof that a symbol does not exist.

`file_patch` accepts this explicit marker format (not numeric unified-diff hunk
headers):

```text
*** Begin Patch
*** Update File: src/example.rs
@@
-fn old_name() {}
+fn new_name() {}
*** Add File: notes.txt
+Remember to update callers.
*** End Patch
```

Use `*** Move to: new/path` immediately after an Update marker, or
`*** Delete File: path` with no hunks. Context lines start with a space. Each old
hunk must match uniquely in the remaining source; the tool will not guess when
context is ambiguous. Updates preserve LF/CRLF and the presence of a final newline; mixed line endings require exact file_edit. Read the
current files before constructing a patch. Maximum: 32 files and 128 KB of tool
arguments. The engine validates every path and prepares every change before the
first write, checks stale content, preserves permissions, and reuses exact
single-use textual approvals and the existing effect lease. Coordination turns,
plan reviewers and background reports cannot bypass their write restrictions.

A multi-file patch is **not a filesystem transaction**. A later disk failure may
leave some completed changes. The journal and error identify completed files and
the failing path; inspect them before retrying. There is no automatic rollback.

## Repository instructions

The workspace root's `AGENTS.md` joins the owner configuration at task startup,
including for workers. Ancestor directories outside the bound workspace are not
loaded. For a file, rules are collected from the workspace down to its parent;
deeper rules apply only in that subtree. Explicit owner requests and runtime
permissions retain precedence. Rule files are bounded to 32 KB each / 64 KB per
chain, redacted, hashed, and checked again when files are accessed.

File reads return new or changed guidance. A write or patch encountering guidance
that the model has not consumed returns `executed: false` and the instructions
before requesting approval or changing files. It must be retried after the next
model response. Merely queuing a read and edit in the same response does not count
as reading the guidance. Compaction invalidates this tracking so nested rules can
be loaded again. Guidance is instruction text, not executable policy: arbitrary
shell commands retain their existing operating-system access.

## Language servers

Fénix implements stdio JSON-RPC LSP with initialization, document open/change/close,
UTF-16 positions, server requests and cancellation. It does not install servers,
execute repository-supplied launch commands, build an embedding index, or invoke
an LLM for language queries. Owner-configured servers execute as the local user;
LSP is not a sandbox. Use installed commands, for example:

```json
[
  {"name":"rust","command":["rust-analyzer"],"extensions":["rs"]},
  {"name":"python","command":["/absolute/path/to/pylsp"],"extensions":["py"]}
]
```

Entries also accept `initialization_options`. Extensions must not overlap. Sessions
start lazily and are isolated by canonical workspace and server configuration.
The pool keeps at most four sessions and 32 open files per session, serializes
queries, bounds framing/output, and kills failed or cancelled sessions and their
process groups. Configuration changes and shutdown close the sessions.

`code_query` accepts `path`, `operation` (`definition`, `references`, `hover`,
`symbols`, `implementation`, `diagnostics`), and optional 1-based `line` and
`character` (UTF-16 code units). Capabilities depend on the configured server.
Writes, edits and patches request diagnostics for up to four changed files when
a matching server is configured; larger patches disclose deferred files. Reads
refresh document contents before queries. Pull diagnostics are supported; push
diagnostics may return `pending` while the server works. Server failure is
reported separately from an already-completed file mutation. Diagnostics do not
replace compilation, tests or checking the user's intended behavior.

## Active-task compaction

The configurable defaults are a 32,768-token context capacity, compaction at
22,000 estimated tokens, and a 6,000-token recent tail. Set capacity to the chosen
model's actual limit. The byte-based estimate includes tool schemas and provider
replay metadata; it is not a billing tokenizer. Output and safety headroom are
reserved. This is a context budget, not a spending limit.

Before a new inference, if the threshold is exceeded, the engine:

1. Selects a boundary that never separates a tool call from its results.
2. Saves the redacted original context, task/session IDs and project binding in
   SQLite, with a SHA-256 hash and checkpoint journal event.
3. Summarizes older semantic content without sending encrypted provider replay
   blobs to the summarizer. Large old tool outputs are represented by previews
   and checkpoint references; the original remains available.
4. Preserves system instructions, the explicit task contract, recent complete
   batches and their provider metadata; inserts a structured summary and source
   checkpoint reference.
5. Continues with the same task/session and existing retry/step/deadline counters.

`task_inspect` with `checkpoint_id`, `offset` and `length` reads the exact saved
JSON in character pages with the existing project/task authorization. A summary
is lossy; critical omitted details must be recovered from that artifact. A failed,
empty, cancelled or truncated summary does not replace the original context.
There is no crash-resume or rollback feature implied by these checkpoints. If
instructions alone, a single complete batch, or the summary cannot fit, execution
fails with preserved evidence instead of silently dropping required context.

## Verification

Deterministic tests cover regex/glob filtering, ignore rules, traversal rejection,
ambiguous patches, multi-file preflight, move/create behavior, scope and approval
gates, rule ordering/refresh and same-response read/edit handling. An LSP fixture
covers initialization, incremental sync, reuse, project isolation, diagnostics and
cancellation. Compaction tests verify complete tool pairs, preserved originals,
checkpoint access controls, summary failure, and active-loop continuation without
replaying effects. The optional real Python LSP test requires a separately
installed `pylsp` with its `pyflakes` plugin:

```sh
FENIX_TEST_PYLSP=/absolute/path/to/pylsp cargo test -p fenix-agent \
  real_python_language_server -- --ignored
```

Validation on 2026-09-15: the workspace tests, the final agent suite (82 tests),
all 11 browser tests, and workspace Clippy with warnings denied passed. The real
Python LSP test passed separately, checking definition lookup and a syntax-error
diagnostic after a file change. These checks used mock model responses; no new
paid-model benchmark or coding-quality score is claimed.
