# Nemotron: five new native Terminal-Bench tasks

[Español](../../es/audits/nemotron-native-five-task-audit.md)

Date: 2026-09-15. Fénix 0.3.0 working tree, including the native evaluator and preceding harness changes. The engine was frozen for all five attempts.

**Result: 1/5 tasks passed (20%), costing US$0.04838435.** These are five new tasks, disjoint from the previous 20-task audit. They were selected for distinct failure surfaces, not as a random or representative leaderboard sample.

| Task | Official reward | Native seconds | Provider calls | Input tokens | Output tokens | USD |
|---|---:|---:|---:|---:|---:|---:|
| `git-leak-recovery` | 1 | 34.36 | 22 | 182,517 | 7,387 | 0.00640414 |
| `headless-terminal` | 0 | 58.52 | 18 | 136,810 | 19,783 | 0.00737083 |
| `pypi-server` | 0 | 75.28 | 32 | 260,973 | 22,712 | 0.01140713 |
| `query-optimize` | 0 | 130.14 | 20 | 162,642 | 26,752 | 0.00918554 |
| `regex-log` | 0 | 178.65 | 11 | 71,214 | 78,991 | 0.01401671 |


Provider totals: **103 calls; 814,156 input tokens, including 391,680 cached tokens; 155,625 output tokens.** Cached input is included in input, not additional. All responses contained usage and cost; no HTTP errors were recorded. Output includes reasoning as reported by the provider. Cost is the sum of actual `usage.cost`, not an estimate from a price list.

Native execution summed to 476.95 seconds across tasks. Harbor job wall time was 1892.81 seconds, including setup, image/container operations, the official verifiers, and concurrency. It is not model latency. Native timing starts after the setup connection check; billing includes that check, foreground calls, summaries, and auxiliary calls. The native report's narrower usage scope must not be substituted for total provider billing.

## Protocol and limits

- Official Terminal-Bench 2.1 dataset revision `7131e4375048a0e408a8fb404b5f499d726b695b`, Harbor 0.23.0.
- One attempt per task, no job retries, at most two concurrent trials. No model/engine changes between tasks and no hidden verifier inputs given to the agent.
- `nemotron-3-5-lightning-30b` on RunInfra through the OpenAI-compatible `openrouter` provider adapter and recording proxy; provider-default thinking, no explicit reasoning override.
- Official task containers: one CPU and 2 GiB RAM each; agent timeout 900 seconds. Verifier timeout 900 seconds except SQL, whose official timeout is 1,800 seconds.
- Native Rust execution; Python only orchestrates Harbor/proxy. Fresh isolated runtime state and builtin skills per task. `/app` workspace, allowed commands and writes. No language server configured.
- Configured context window 32,768; output ceiling 16,384 per response; effective input budget 15,360 after output reservation and margin; 128 model steps. An output ceiling is not a monetary budget.
- The native evaluator does not start the interactive server's background web crawler. Explicit available tools and the normal worker/compaction machinery are used.
- Official verifier reward determines passing; the agent saying completed does not. Verifier tests were inspected only after their respective agent attempts stopped.

Executable SHA-256: `ff895b861c517db5d4b975906bab3b530ff41f12515fba74e9639fbdc705c637`. All 100 recorded source hashes and the executable hash were unchanged after the run. The working-tree fingerprint matters because the previous implementation work was not represented by a new commit.

## What failed and why

### Tool discovery and recovery: headless-terminal

The agent installed `pexpect` and wrote an implementation, but initially omitted inheritance from `BaseTerminal`. It then tried `file_edit` with `old_text/new_text`, then `old_content/new_content`, and `file_patch` without `patch`. The runtime returned missing-argument errors. The recorded requests exposed only `file_read`, `files_search`, `command_run`, `command_status`, `tool_search`, and `ask_user`; the edit/patch schemas were not loaded. The model guessed callable names advertised in the discovery catalog instead of using `tool_search`. Runtime dispatch still accepted those names for validation.

Repeated reads of the unchanged implementation eventually triggered the loop guard. The official verifier passed import and cancellation but failed the other five behavioral tests. The output passed raw newline and Ctrl-C characters to `pexpect.sendcontrol`, which expects a control-key name. A separate local check of the same dependency versions (`pexpect` 4.9.0 / `ptyprocess` 0.7.0) confirmed both inputs write zero bytes. Enter was therefore not delivered. The cancellation test only checks that an output file does not exist; its pass does not establish that cancellation actually worked. The failures were broader than inheritance alone. Evidence: `diagnostics/terminal-control.json`. No behavioral test command was run by the agent before it stopped.

**Recommended change:** when an authorized but unloaded tool is requested, load and return its exact schema before any side effect. On malformed arguments, return required fields, received fields, and a minimal valid example. Keep permission checks separate from discovery. Give a bounded recovery opportunity before terminating a repeated-read loop. This is a model/tool-interface failure with an additional recovery weakness, not proof that the guard alone caused the bad implementation.

### Service lifetime and false completion: pypi-server

The package built successfully after several unsuccessful package/build commands. However, the agent launched `python -m http.server ... &` with `persistent=false`. The parent shell ended and the process manager's normal process-group cleanup removed the background server. `persistent=true` was documented in the schema but was not used. This was not a failure of the evaluator's keep-services handoff: no persistent service had been registered.

The generated index also linked directly to the wheel under `/simple/`, without the project-specific `/simple/vectorops/` page required for this installation route. The agent's installation check returned `Requirement already satisfied` because it had previously installed the package from the working directory. That command never established that a fresh installation through the HTTP index worked. The official verifier uninstalled the package first and its installation failed. Fénix nevertheless marked the task completed.

**Recommended change:** return explicit descendant-cleanup/lifetime information for transient commands and guide service launches toward persistent execution. For package-serving tasks, validate a fresh install from a separate environment and inspect the project endpoint. Associate completion claims with the relevant observed behavior rather than a generic exit code. Do not silently turn arbitrary commands into persistent services.

Two auxiliary memory-extraction attempts returned invalid JSON. They were nonfatal and are included in billing; they did not explain the missing service.

### Coarse execution locking: query-optimize

The agent read the SQL and database schema, then launched the slow original query. It subsequently produced a candidate CTE/window-function rewrite and attempted to save `/app/sol.sql`. The engine waited for the existing foreground command and returned `process_busy`, `retryable=true`, `executed=false`. The candidate was never written. The official verifier subsequently passed its two baseline/database checks and failed all four solution checks because `sol.sql` did not exist. Its pytest run took 1,405.36 seconds (23 min 25 s), largely in the first baseline comparison, independently of the 130.14-second native agent attempt. The lock serialized an unrelated output-file write behind the long database read.

The agent continued polling and rereading the original SQL. The guard stopped the task after repeated identical file reads. The absence of output from a live query was not evidence that it had stopped making CPU progress. The task was stopped before the 900-second agent timeout. The candidate was not materialized or executed in the official attempt. After it stopped, a separate deterministic diagnostic extracted the attempted SQL from the trace and ran it against a copied, read-only database: its 500 rows and all eight column names exactly matched the reference. The database hash matched the verifier's expected hash and remained unchanged. One host run took 5.21 seconds for the candidate and 2.83 seconds for the reference; these are not official performance measurements and do not establish that the runtime threshold would pass. No model call, change to the graded container, or second agent attempt was involved. Evidence: `diagnostics/sql-result.json`. This strengthens attribution of the missing artifact to the coarse execution lock, while preserving the official failed-delivery outcome.

**Recommended change:** replace task-wide effect serialization where feasible with conservative resource/dependency tracking. Permit independent file writes when non-conflict is established; retain serialization when an arbitrary shell command's effects are unknown. Teach bounded baseline measurement and distinguish waiting on a running process from a failed action. Before a terminal loop stop, surface the blocking process and offer a bounded alternative path without cancelling useful work automatically.

### Output exhaustion: regex-log

Four foreground generations reached the configured 16,384-token output ceiling. Fénix preserved them as unexecuted truncated responses, tried three recoveries within the same task, then stopped on recovery-budget exhaustion. The response metadata showed output consumed by reasoning rather than a large finished tool payload. A generic instruction to split large file creation did not address that failure mode.

The one artifact produced contained only an IPv4 matcher. The official test expected dates and received IP addresses. No `command_run` test of `re.findall` occurred. More identical retries would increase cost without demonstrated benefit.

**Recommended change:** distinguish reasoning exhaustion from oversized tool arguments; use small, concrete implementation-and-test steps and a bounded change of strategy. Keep partial generations non-executable. Evaluate changes to reasoning/output allocation separately instead of assuming that a larger limit or another model will fix it.

### Compaction: mechanically successful, semantically imperfect

All 15 recorded compactions completed; no fatal summary/cut error occurred in this sample. Exact checkpoints remained available. However, a headless-terminal summary invented a dated user correction (`2026-06-10`) for an action taken by the assistant and described external/user validation as the next step. In SQL, a later summary described `sol.sql` as partially written even though the tool explicitly returned `executed=false` and the file did not exist. The task contract still contained the original objective. This is an observed summary-fidelity defect; the run does not isolate its causal contribution to the final failure.

Each foreground request carried a static system message of 22,960–24,625 characters, including general assistant/routing/memory instructions. This is a character count, not a token estimate. Consider loading only relevant operational context for coding tasks; this run does not isolate prompt size as a cause of failure.

Several compactions preserved zero recent messages. Summaries consumed 58,868 output tokens across the five tasks, so summary size and frequency deserve attention. Preserve recent failures, tool schemas, unresolved blockers, and source roles as structured fields; prevent summaries from upgrading assistant inferences into user instructions. Checkpoint durability alone does not validate a summary.

## Priority after this audit

1. Make on-demand tool schemas self-correcting and give actionable argument errors.
2. Avoid blocking independent output writes behind slow foreground reads where non-conflict can be established.
3. Clarify transient/persistent service outcomes and verify fresh consumer behavior before completion.
4. Add bounded loop recovery that considers blockers, recent compaction, and actual process state.
5. Preserve provenance and critical recent state through compaction while reducing summary overhead.
6. Differentiate reasoning exhaustion from oversized actions and test an adaptive, bounded recovery policy.

These are recommendations, not changes applied during this run. Git recovery passed all five official checks, demonstrating one complete successful path. This five-task sample cannot establish that the updated harness is better or worse than the earlier 20-task run, nor attribute every failure solely to the harness rather than Nemotron.

## Reproduction and evidence

[CSV metrics](nemotron-native-five-task-results.csv), [native evaluator](../native-evaluation.md). Private local artifacts: `.fenix/evaluations/native-five-20260915/` (`protocol.json`, `selection.json`, `source-hashes.json`, `integrity.json`, `metrics.json`, `api/`, and `jobs/five-new-tasks/`). Per-trial `agent/native/report.json`, checkpoints, and `verifier/` preserve the audit trail. Raw credentials, synthetic secrets, provider reasoning, and benchmark test contents are not published in this report.
