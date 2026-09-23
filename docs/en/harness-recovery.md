# Procedural skills and evidence-led recovery

[Español](../es/harness-recovery.md)

The runtime includes seventeen manually curated skills: the original five plus repository navigation, test-driven repair, environments/dependencies, process/service management, package build/install, SQL analysis, text parsing, Git recovery, software implementation, and native Word, spreadsheet and presentation workflows. These are general procedures, not benchmark answers or automatically learned skills. Installation preserves user-modified files.

`skill_search(query, limit=3)` searches enabled names, descriptions and complete bodies lexically, with higher weights for names and descriptions. Results contain metadata, not bodies. `skill_load(name)` returns the procedure and loads up to eight authorized tools declared in optional `required_tools` metadata (YAML list or comma-separated string). Existing custom skills need no new metadata. Skills do not grant permissions. The master and workers can search the entire catalog and load bodies selectively.

A guessed, authorized tool that is not exposed returns `tool_schema_loaded`, its schema and `executed=false`. Only a subsequent valid call may execute. Invalid arguments return required/received fields, the schema and a placeholder example. Examples explain shape; they are not executable task answers.

## Processes and resource conflicts

Ordinary commands keep existing approval and conservative serialization. `command_run` accepts optional `read_paths`: existing absolute inputs or paths relative to cwd. This starts a fresh Rust helper before creating the application runtime. Linux Landlock ABI 3 enforces read-only filesystem access; seccomp restricts syscalls, including network, metadata mutation and process manipulation. Standard libraries/configuration are implicit read inputs. No automatic fallback to unrestricted execution occurs. Kernels without the required support return an error. This deliberately restricted mode may not support arbitrary software, writable caches, temporary files or interactive programs.

Read-only input paths are registered for the lifetime of the process. Independent `file_write` and `file_edit` operations may proceed; overlapping input/output paths wait. Broad directory inputs reduce concurrency. Multi-file patches and unknown effects remain conservative. Inputs are live reads, not snapshots; external processes outside Fénix are not controlled. Read-only commands cannot be persistent or TTY sessions. Task completion still waits for outstanding work. The normal service mode remains available through explicit `persistent=true` and returns its descendant-lifetime policy.

## Context and recovery

Compaction checkpoints the redacted original before changes. It first tries deterministic masking of older successful tool outputs, retaining call/result pairs, exact recent batches, source roles and a runtime contract. Full outputs remain retrievable. This path makes no summary-model call. If it does not fit, the existing summary/fallback path remains; generated summaries carry the assistant role and are explicitly unverified. Exact contract observations override narrative claims. This reduces, but does not guarantee elimination of, semantic loss.

Repeated unchanged observations trigger a bounded diagnostic opportunity with process/tool/checkpoint recovery paths. Existing caps remain three recoveries per reason and eight total. Truncated responses never execute. Large action payloads receive edit-splitting guidance; other output exhaustion receives a concrete probe-and-test strategy. That distinction is a payload heuristic, not a measured reasoning-token classifier.

Planning, revision, background workers and independent plan review reuse existing machinery. The system guidance asks for reviews when local checks leave material uncertainty. It does not create an extra reviewer for every tool call, add model switching, or enable nested delegation.

## Operational snapshot and failed approaches

Before inference, the engine derives a bounded operational snapshot from the
existing plan and task contract. It includes the current plan version, active
milestones (prioritized over pending ones), constraints, completion counts,
reported blockers, owner updates and optional sourced next-step/decision notes.
It is absent for ordinary small tasks without a plan, operational notes or failure
history. This is reference data in an assistant message, never new user authority.

The snapshot is inserted once into the inference request, not accumulated in the
conversation or saved as a competing plan. Unchanged state yields identical text;
replanning, milestone transitions, new evidence or owner answers change it on the
next inference. Compaction reserves its input space and rebuilds it from durable
state. Its structured payload is bounded to 12 KB, with at most four milestones,
three own failed approaches and two scoped predecessor/prior-generation records.
Full details remain accessible through `plan_manage`, `task_contract` and
`task_inspect`. This adds ordinary input tokens when present but no separate LLM
call, embedding index or background summarizer. No billing savings are claimed.

Tool errors, nonzero command exits and explicit failed/timed-out results update
`task_contract.failed_approaches`. Up to eight distinct actions retain redacted
argument/evidence previews, first/latest call references, observation count and
plan version. Repeated observations of one failure are grouped, not mistaken for
distinct attempts. Full receipts remain in the existing task journal. Running
processes, schema-loading receipts and approval denials are not execution failures.
A later successful tool observation is recorded separately; it does not prove
task completion or validate a proposed root cause.

`task_contract annotate` additionally accepts `next_step`, `decision` and
`failed_approach`. The agent can state a hypothesized cause/abandoned strategy with
its `source` and optional `retry_when` condition. These remain agent claims.
The engine never invents a cause or executes a proposed retry. Next-step/decision
notes are shown only while their plan version and owner updates/answers match;
older notes remain inspectable. There is one current next-step note and its
earlier versions stay in annotation events. No note is required per tool call.

Planned replacement workers receive bounded historical failures only from the
existing predecessor/prior-generation evidence allowlist, with source task IDs.
Independent siblings and unrelated tasks/projects are excluded. Reopening the
runtime preserves these records but does not automatically resume interrupted
tasks or replay effects. Existing permissions, retry policy and completion audits
remain responsible for execution and acceptance.

## Searching saved tool outputs

`task_inspect` accepts `query` to search the redacted tool-result journal with
local SQLite FTS5/BM25. The current task is the default; another `task_id` must
pass the existing project/worker evidence authorization before indexing or
retrieval. Each owner uses their own store. `call_id` optionally restricts the
search to one tool call. No embeddings, model summarizer or extra provider call
is used; retrieved text still consumes normal model input tokens.

The disposable index is populated lazily in batches of 64 tool-result events,
including events saved before this feature existed. `index_complete=false`
means the search has not covered the full task: repeat with search `offset=0`
to advance indexing. An empty partial search is not evidence of absence. Once
indexing is complete, `next_offset` paginates ranked hits; results can change
when new outputs arrive. The default is three fragments, with at most eight per
request. Inspection results are excluded to prevent recursive indexing.

Fragments retain `call_id`, `event_seq`, tool name, timestamp and character
offset/length into the original saved output. Chunks contain up to 2,000
characters with 200-character overlap. These are exact historical excerpts,
not instructions or proof of current file state. To read more, omit `query`
and pass a hit's `call_id`, `offset` and `length` to `task_inspect`. Search
offsets paginate hits; fragment offsets address original characters.
If a call has multiple journal versions, only its latest version is searchable.
The original journal remains authoritative and is not rewritten by indexing.

Example: `task_inspect({"query":"authentication failure","call_id":"build-17"})`.
Queries are escaped lexical terms with OR matching; this is not semantic or
fuzzy retrieval. Check retrieved evidence rather than assuming every hit matches
the complete intent. The search cannot be combined with `checkpoint_id`.

## Evaluation workflow

Use `scripts/benchmarks/harbor/experiment.py` with `--root`, `--job`, `--tasks` (five comma-separated names), `--dataset`, `--bundle`, `--config` and `--harbor`. It freezes binary/configuration hashes, runs the native evaluator in official containers, and produces `results.json` from official grades plus recorder billing. The recording proxy must already be running. `--summarize --root PATH` refreshes completed results without launching work. Missing charges remain unknown.

Compare fixed main and candidate binaries on the same tasks and settings. Previously inspected tasks are development regressions. Confirmation uses two five-task unseen cohorts with two independent attempts per version. Record failures and infrastructure exclusions; do not treat assistant completion as a passing grade. The research branch is reviewed separately from main.

Background workers snapshot model availability/connections before asynchronous work. They never hold a model read lock while waiting for configuration, which prevents an idle worker from deadlocking a model change. Web view loads similarly discard stale responses after navigation.

Validation on September 15, 2026: 239 Rust tests passed, four environment-dependent tests remained ignored; all 13 browser tests passed, as did Clippy, formatting and documentation tests. Browser validation used an alternate `TMPDIR` because the host `/tmp` volume was full. These are regression checks, not evidence of a Terminal-Bench score improvement.

Task context highlights up to two lexical skill candidates without loading their bodies. Suggestions are optional and may be irrelevant; the complete catalog remains available. Shipped repair and Git procedures update only when their prior content matches exactly, preserving owner edits and disabled state.

Long tool receipts preserve head/tail evidence and execution metadata instead of retaining only a prefix. Complete redacted output remains in the task journal; a preview never proves that the omitted middle is irrelevant. Context estimation counts the larger of normalized content and provider replay, plus an envelope allowance, without duplicating visible text or discarding hidden reasoning. The estimate is not a model-specific tokenizer.

Basic file reading, creation and exact-content editing are exposed immediately; advanced patching and other specialized schemas still load on demand. Schema visibility never grants permission. The opening guidance checks original evidence before requesting clarification and still surfaces every unresolved doubt. The environment procedure now distinguishes diagnostic-tool setup failure from application correctness.

See [scalable skills and native documents](skills-and-documents.md) for the incremental index, bounded context and document tools.
