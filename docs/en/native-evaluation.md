# Native evaluation and execution recovery

[Español](../es/native-evaluation.md)

Fénix includes `fenix eval` in its Rust executable. It runs the real assistant,
providers, tools, context management and background workers with a fresh database
and built-in skills. The benchmark owns its task containers and official verifier.
A completed assistant turn is **not** a passing benchmark result.

## Context configuration

New configurations default to a 200,000-token context window with compaction at
160,000 tokens. Explicit saved settings are preserved; operators must use a
capacity supported by their configured model. This does not increase benchmark
execution time or guarantee task completion.

The production and local development settings were explicitly aligned to the
200,000-token window and 160,000-token compaction trigger on 2026-09-28. Updating
the source default alone does not migrate an existing database. When deploying
an existing instance, also verify `settings.coding` in the authenticated status
endpoint and update its saved settings. New benchmark runs inherit these defaults
when their evaluation configuration omits `coding`. Historical evaluation results
and deliberately small context fixtures used to test compaction remain unchanged.

## Delegation and blocked evaluation

Workers receive the original parent request and its owner updates as reference
inside the durable contract only when both bindings are resolved and share the
same general/project scope. The assignment remains the execution boundary;
parent transcripts are not copied. Changing scope removes the inherited source.
A running plan with no queued/running tasks and a worker awaiting user input
ends evaluation as `requires_user_input` after the configured waiting interval.
The evaluator does not invent answers to continue the task.

## Event-driven command waiting

Foreground commands wait in Rust for process completion, without model polling.
A review is returned after 120 seconds or 60 seconds without output. Silence is
only advisory; neither threshold cancels the process. Command timeouts, task
cancellation and shutdown remain authoritative. Interactive terminals and
persistent services yield quickly. Use `yield_ms=0` for an immediate command
return and `command_status` with `wait_ms=0` for a snapshot. Output continues
to be collected in the process log while the tool waits. This is an
in-process notification, not an external HTTP webhook.

## Explicit execution outcomes

Tasks that run commands or modify files close through the master-only `task_finish`
tool. The model reports `completed` with observed evidence, or `blocked`/`failed`
with a concrete blocker. The engine rejects a plain final response after execution
and asks the model to continue or report an explicit outcome; this recovery is
bounded by the existing recovery policy. It also rejects finishing while a
foreground command is running. The summary is delivered directly, without an
additional model call. Ordinary conversation keeps its normal completion path;
subagents continue to use `subagent_finish`.

This is an outcome protocol, not an independent correctness check: supplied
evidence can still be wrong. Official benchmark verification remains necessary.

## Run one isolated attempt

Build the frontend first (`pnpm --dir apps/web install --frozen-lockfile` and
`pnpm --dir apps/web build`), then build with `cargo build --locked -p fenix-server`:

```bash
./target/debug/fenix eval /absolute/path/config.json /absolute/path/new-trial
```

The output directory must not exist. This prevents accidental reuse of personal
memory or prior attempts. Run this command **inside the official task container**
for Terminal-Bench; running it on the host does not constitute that benchmark.

Example configuration (replace model, endpoint and dataset revision):

```json
{
  "benchmark": "terminal-bench-4.0",
  "dataset_revision": "452bf305c6daa62fc59061d22133a7cbc7c1572e",
  "task_name": "atrx-vep-crispr",
  "instruction": "The exact official instruction, without hidden tests",
  "api_key_file": "/opt/fenix-eval/proxy-key",
  "timeout_seconds": 28800,
  "waiting_seconds": 15,
  "proxy_upstream": "172.17.0.1:48931",
  "settings": {
    "provider": "openai",
    "model": "gpt-6-luna",
    "base_url": "http://127.0.0.1/native-trial/v1",
    "workspace": "/app",
    "command_policy": "allow",
    "file_write_policy": "allow",
    "reasoning_effort": "low"
  }
}
```

The optional `proxy_upstream` opens a native Rust TCP tunnel to the recording
proxy. The model endpoint remains loopback HTTP; normal provider HTTPS validation
is unchanged. The tunnel closes before the verifier starts. Without a tunnel,
use a normal HTTPS endpoint or an existing loopback proxy.

The evaluator starts the core workers but does not start background public web
crawling. Explicit web tools remain available; their search corpus is fresh.
Record this distinction when comparing with the interactive server.

Settings use the normal schema and validation. Optional `coding` and `execution`
objects override normal defaults; evaluation does not silently enlarge limits or
supply answers. Configuration performs the normal connection check before the
measured agent phase. The timeout begins after that check. Host tools and tokens
remain outside the task image when using the recording proxy; upload only a
throwaway proxy credential. The agent can execute arbitrary commands within its
container, so putting the real provider key there does not isolate that key.

Generated artifacts:

- `manifest.json`: executable SHA-256, package version and dataset revision.
- `progress.json`: lightweight live task/plan states.
- `report.json`: final events, model and tool counts, input/output usage, elapsed
  time, process states, recoveries and stop reason.
- `ready.json`: written only after the inference workers stop and the report is
  complete; this is the verifier handoff marker.
- `runtime/`: fresh database with checkpoints, task contracts and process logs.
- `skills/`: isolated built-in skills.

`report.json` deliberately leaves `reward` and `cost_usd` null. Its token counts
cover recorded foreground and active-compaction responses. Connection checks,
auxiliary memory/routing calls and transport retries are not a complete billing
ledger; use provider-side recording for total charges and cached-token pricing.
Missing usage is counted rather than presented as known zero usage.

## Harbor transport

`scripts/benchmarks/harbor/native_fenix.py:NativeFenixAgent` uploads the executable
and calls its native evaluator. Python is needed by Harbor on the host, not as
an HTTP controller inside the task image. Configure:

```bash
export FENIX_EVAL_BUNDLE=/absolute/path/bundle
export FENIX_EVAL_CONFIG=/absolute/path/evaluation-template.json
export PYTHONPATH="$PWD/scripts/benchmarks/harbor"
```

The bundle contains an executable `fenix`; a compatible dynamic loader and its
libraries may accompany it (`ld-linux-x86-64.so.2`). The template is the same
configuration above. Harbor supplies the exact instruction and trial name.
The adapter supplies a disposable proxy key; its endpoint must be a reachable
recording proxy, not a provider requiring a real key. An optional `{trial}` in
`settings.base_url` is replaced by the unique trial identifier.

Use `native_fenix:NativeFenixAgent` as `import_path` in the Harbor agent config.
Keep official resources, timeouts and verifier unchanged. The earlier
`fenix_agent.py` and frozen audit artifacts remain historical evidence.

`--keep-services` keeps the Rust process alive after a terminal attempt, with
inference workers stopped, so the official verifier can inspect persistent
services. The adapter uses this mode and returns when `ready.json` appears.
SIGTERM after verification shuts services down; container deletion also ends
isolation. Timeout or an unanswered clarification ends inference and shuts down
services immediately. No user response is fabricated. `all_tasks_terminal` may
include a failed task: consult the task traces and official reward.

## Architectural changes

1. **Context admission.** A shared budget reserves the configured output ceiling
   (including reasoning) and 1,024 tokens of headroom within the operator-supplied
   capacity for the selected model. Compaction happens at the smaller of that
   input allowance and the configured threshold. Tool output previews share a
   batch budget; full redacted outputs remain paginated artifacts. Capacity is
   configured, not guessed from a model name. Token counts remain conservative
   byte estimates rather than exact provider tokenization.
2. **Recovery.** Checkpoints are durable before cut selection, even when the most
   recent complete tool batch is too large. Summaries use the configured output
   ceiling and available room, rather than a fixed 2,048-token limit. Invalid,
   oversized or unavailable summaries fall back to bounded exact excerpts and a
   checkpoint reference. This fallback is explicitly incomplete. Truncated
   foreground generations are retained as audit evidence, never executed or
   replayed as unmatched tool calls. The next generation is asked for a smaller
   action. Provider filtering remains a terminal condition.
3. **Process supervision.** A dependent effect waits up to 30 seconds for earlier
   foreground work, without asking the model to poll repeatedly. If still busy,
   it returns `process_busy`, `retryable`, `executed: false` and process IDs. Fast
   commands return their status/output after a short initial wait. Persistent
   shells wait for ordinary background jobs; cancellation/timeouts still reap
   their groups. Prefer foreground services: double-forked daemons, `setsid` and
   explicit shell exit are not a supported supervised lifetime. `command_status`
   accepts `ready_port` for a bounded localhost TCP probe, not a semantic health
   check or proof of which process owns the port.
4. **Progress-aware loops.** Repetition tracking resets on observed process
   identity/state/output progress or completed effects. Identical results without
   progress still stop after three occurrences. Status polling stays bounded by
   command and task deadlines. A model's claim of progress does not reset it.
5. **Task contract.** The original objective, scope, policies, explicit replies,
   grants, denials and recent tool evidence are durable. `task_contract` supports
   optional source-labelled constraints, assumptions, criteria and verification
   annotations; annotations are not authority. Exact normalized questions reuse
   prior explicit replies, but different questions remain unresolved. Compaction
   preserves the contract; `task_inspect` exposes it with execution state. Semantic
   equivalence of differently worded questions is not decided by a fuzzy heuristic.
6. **Continuous evaluation.** The native report separates observed execution from
   grading. Regression tests cover the audit's concrete failure mechanisms and
   run in the existing workspace CI without API credentials. Compare quality,
   time and provider-recorded cost on the pinned set **and unseen tasks** after
   engine changes; unit tests do not establish a new Terminal-Bench score.

Recovery stages and per-reason counters are durable. There are at most three
recoveries per reason and eight total per task, inside existing step/time bounds.
Completed effects are not replayed by recovery. Completed tool batches are saved
as execution context. This is in-turn recovery, not automatic resumption of shell
side effects after restarting the application.

Original redacted evidence remains recoverable, but lossy summaries/excerpts do
not guarantee that the model retrieves every relevant detail. Persisted contracts
and instructions that alone exceed the minimum context budget cause an explicit
error with a checkpoint rather than silently dropping requirements.

When started explicitly through the bundled `ld-linux` loader, `binary_sha256` hashes the evaluated program, not the loader. `binary_path` and `process_image_path` record both identities. Earlier loader-launched manifests lack this correction: use the external frozen bundle/source protocol for their program hash; do not rewrite historical reports.

## Externally supervised evaluations

For an evaluation whose outer runner owns cancellation, set `settings.execution.externally_supervised` to `true`. This opt-in disables the foreground task deadline, model-step ceiling and total provider HTTP request timeout. Defaults remain bounded. An evaluation may then set `timeout_seconds` to `0` to disable its own timer. Cancellation and reporting still work. Connection establishment timeouts, provider capacity, tool safety/size checks, recovery policies and the external benchmark deadlines still apply; this is not unlimited computation.

Set context capacity to the provider's verified capacity and choose a smaller output ceiling; requests are still clamped to remaining estimated context. The supported output configuration ceiling is 1,048,576 tokens. Do not claim a controlled model-only comparison when changing these settings together. Keep configuration, binary hash, infrastructure failures and all known costs in the evaluation record.

### Delegated turn completion and evaluation credentials
A master may release its conversational turn with a progress message while its own subagents or plan remain active. The runtime records a delegated outcome; the background work keeps its independent status and report path. Foreground commands still prevent closing, and unrelated or completed workers cannot justify a handoff. Skill learning is skipped for these incomplete handoffs.
The local evaluation recorder can alternate two host-only credentials. It forwards rate-limit headers and honors a shared cooldown on HTTP 429; rotation does not increase shared account quotas. Subagent retries honor the full provider Retry-After duration.

Event pagination with task_inspect(after=0, then next_after) returns event-only pages without repeating the full contract. For tightly coupled package repairs, keep diagnosis, repair and consumer verification in one execution assignment and retain final substantial-work auditing.

### Optional Jev and independently scoped OpenAI credentials

Set `decisions` to `{"enabled":true,"shadow":false}` in the evaluation configuration to enable typed decisions. This option requires `proxy_upstream`. The evaluator forwards Jev traffic to the same loopback tunnel at the model endpoint plus `/jev`, using only the disposable evaluation credential. The host gateway supplies the real Jev key. No real provider key is uploaded to the task container.

`openai_decision_proxy.py OUTPUT CONFIG` records GPT-6 Luna and Jev traffic. The private gateway configuration contains `openai: [{"file":"/private/key-path","quota_group":"verified-organization-id"}, ...]` and `jev_key_file`. Assign a distinct quota group only after confirming independently scoped quotas; keys from the same organization must share a group. The gateway reserves tokens, observes OpenAI rate-limit headers, honors 429 cooldowns, and distributes work across available groups. It does not race duplicate requests or retry tool side effects. Pricing and latency must include gateway waits and Jev requests.

Repeated inspection rounds may trigger an advisory Jev decision: try a supported repair, perform a focused diagnostic, continue inspection, or abstain. This is an optional recommendation, not authorization, a plan rewrite, or proof of completion. Failure and low confidence preserve the existing flow. The programming instructions prioritize an observed failure → targeted repair → check loop and the complete build/install/consumer path before broad regression work.

### Completion requirements backed by tool receipts

The task contract persists mandatory verification checks separately from free-form annotations. Explicit repository-suite requests recognized by the engine create a pending suite check; `task_contract(action="require")` registers additional immutable command checks with a verbatim quote from the assignment. Only observed command results in that task can satisfy them. A missing runner, zero collected tests, filtered single-test run, unrelated successful example, or prose evidence cannot clear a required suite check. Recognized exclusions must appear in the assignment.

`task_finish(completed)` and ordinary worker completion reject missing, failed or stale checks, returning their IDs for repair or an honest blocked result. Asynchronous command receipts retain the original command and revision. Managed file edits and recognized build/install commands invalidate older evidence. The compact operational context preserves check status and recovery IDs. Plan-review and completion-audit workers keep their dedicated validators.

This is a conservative guard, not a semantic proof of every requirement: natural-language recognition, test-runner output recognition and shell mutation detection are bounded heuristics. Arbitrary scripts, wrong working directories for inferred suites, unrecognized writes and a weakened test suite still need review. Exact registered command checks may additionally constrain the working directory. The official benchmark grader remains independent.

For a no-Jev evaluation, omit `decisions` and `jev_key_file`; the host gateway rejects the Jev route without loading its key.

Single official evaluation (2026-09-25), GPT-6 Luna low, Jev disabled: reward **0**, **2/11 checks passed**, native task time **18.69 seconds** (72.05 seconds total Harbor job). The model cloned the source then voluntarily reported `failed` with no concrete blocker. The new `completed` guard was not exercised; non-success terminal states still accept textual blockers without proving recovery is exhausted. Five provider requests including startup all returned HTTP 200; no quota failure or compaction occurred. No second attempt was made. Local validation: 216 agent tests passed, 3 ignored; 4 gateway tests passed; server build passed. See [record](../benchmarks/luna-low-completion-gate-2026-09-25.json). This is not evidence that recovery is solved.

### Grounded non-success closure and bounded persistence

Execution workers now request `failed`/`blocked` with a `stop_reason` and a task-local `blocker_call_id`. The engine validates an unresolved observed denial/external failure, or three distinct observed repair contexts followed by failure of the same check. Unsupported prose, successful preparation, another task's receipt and an already resolved failure cannot justify abandonment. Missing dependencies get repair guidance rather than an automatic terminal outcome.

Recovery observations survive compaction in the task contract. Async polling of one completed process counts once. Repeating an unchanged failing check does not count as a different recovery strategy. After three identical failures without an observed repair, or three rejected closes without observed progress, the runtime pauses with `runtime_stagnation`; it explicitly reports unfinished work and requests owner/supervisor intervention, rather than asserting that the original task is impossible. Cancellation and genuine permission denials retain their existing control paths; nothing automatically broadens permissions or replays uncertain effects. Ordinary children share the gate; dedicated plan and completion reviewers keep their existing validators.

No additional classifier model is called. Diagnostic and repair recognition are conservative heuristics, not a security boundary or a semantic proof: arbitrary shell writes or equivalent commands may not be recognized, and relevance of external failures still needs agent judgment. Denials or missing user decisions must never be worked around. Local tests cover unsupported closure returning to work, cancellation, scoped/stale evidence, dependency recovery, distinct repairs, async deduplication and runtime stagnation.

One official attempt, 2026-09-25 (Luna low, no Jev): **9/11 checks passed; reward 0; 900-second external timeout**. The agent recovered missing dependencies and ran the suite rather than abandoning. Failures remained in Cython `numpy.int` usage and planarity compatibility. Quota pacing consumed **379.095 seconds**. Repeated ambiguous edits and 14 tool-discovery calls also consumed time. No finish rejection was needed, so this trial alone cannot establish a causal effect of the new gate. After this attempt, failed patches were grouped by target/diagnostic/repair epoch so cosmetic variations no longer evade stagnation detection; this adjustment has local validation only, with no second benchmark or deployment. [Audit record](../benchmarks/luna-low-stop-gate-2026-09-25.json).

The post-attempt adjustment passed all seven dedicated stop-gate regression tests, including equivalent patch failures. The benchmark result above remains attached to its original frozen binary.

### Self-authored comparisons and completion

The completion gate now records failed shell `cmp`/`diff` comparisons even when the request does not mention a test suite. A failure persists in the task contract across compaction and restart. Unrelated successful commands, inspection output and replayed process output cannot clear it. A successful rerun of the same comparison clears it; later managed edits invalidate that evidence.

For an invalid fixture or changed comparison command, `task_contract resolve_check` requires the observed requirement ID, a reason explaining the correction, an exact task quote and a successful comparison receipt from the same task, recorded after the failure and at the current revision. The old failure is retained alongside the replacement. Requested suites and explicitly registered checks cannot be waived with this action. Common masked comparisons such as `cmp ... || true` do not produce passing receipts. This uses code and existing tool results, with no extra model or mandatory reviewer.

The programming guidance requires validating record widths, numeric formats, actual script paths and identical independent starting data before differential testing. Malformed-input exploration must remain distinct from the requested valid-input behavior. The runtime checks receipt provenance and execution, not semantic equivalence: command recognition and shell masking detection are conservative heuristics, arbitrary scripts and shell writes are not fully analyzed, and the model still must justify that a replacement test covers the original objective. Local regressions cover the seventh task's contradictory successful close; no new benchmark attempt establishes an improved score.

## Evaluation isolation and prompt efficiency (tasks 16–18)

Native evaluation disables background skill learning before workers start. The immutable evaluation policy blocks queue creation, worker startup and re-enabling through a tool call; report.json exposes this policy. Manual skills and production learning are unchanged. This avoids cancelling a billed learning request after the task result, rather than pretending its missing usage is zero. Provider accounting is still authoritative for other auxiliary calls or transport failures.

Prompt projections reference exact duplicate request text instead of repeating it in the current-request header and routing JSON. The original user message stays intact; differing intents, clarifications, scope and durable records are preserved. Skill guidance permits familiar direct operations without ceremonial loading and encourages batching independent diagnostics. This reduces duplicated input without summarization calls or discarding evidence.

## Requirement-linked checks and tool composition

Before transforming unfamiliar input, inspect representative records and validate field meaning and units. Completion guidance separates hard constraints from preferences and requires a small relevant behavioral check, rather than relying on file existence or format. Test-strategy and data-pipeline skills reinforce this without another model reviewer. Tool programs expose step schemas, require bare callable names, explain whole-result references and reject invalid programs before any side effect. These are guidance and validation improvements, not a general semantic correctness proof.

## Evidence-led diagnosis and repair (tasks 22–26)

Environment inspection includes a bounded, read-only Git status/stat snapshot for a workspace-root repository. It excludes file contents and parent repository discovery; cached evidence is not a live view. Repeated failure of the same check across distinct observed repair contexts adds a small-reproducer hint to the existing stop context. This is advisory, with no new model, retry policy, reviewer or permission bypass. Successful rechecks clear the hint. Prompt and skill updates distinguish hypotheses from missing owner requirements, require phased relevant verification, and reuse data instead of repeating loads. Legitimate goal/permission questions and completion gates remain unchanged. These mechanisms improve available evidence; they do not prove that a diagnosis is semantically correct.

## Clarification provenance and outcome consistency

Noninteractive clarification no longer turns a failed question delivery into terminal blocker evidence. Indispensable owner decisions, private inputs and authorizations must identify their type and cite the task request or owner update. Technical information is routed back to authorized investigation. Interactive scope questions and approval policies are unchanged. Narrow English/Spanish contradiction checks reject an explanation that itself says the question was unnecessary.

The existing completion path rejects explicit unverified_requirements. An admitted verification gap prompts reconciliation; non_required_limitations can explain a caveat outside the request but cannot waive an executable check or an explicit pending requirement. Both ordinary workers and the master use this path; specialized reviewers retain their existing structured validators. No additional model is called. These are provenance, structured-state and conservative text checks, not a general semantic proof: dishonest or incorrectly classified claims can still escape them. Prompts and existing skills now prioritize actual format metadata, independent checks of risky conventions, data coverage and readable visual representations.

## Durable diagnostics and meaningful recovery (tasks 27–31)

Stored stop/completion diagnostics now retain bounded head and tail excerpts, reusing the existing receipt preview helper. The full receipt remains available through task inspection. This preserves terminal errors after long warning prefixes; it does not make omitted middle text complete evidence. Identical verified file_edit before/after hashes no longer advance recovery, reset the loop guard or stale a passing check. Missing hash evidence remains conservative.

A quiet-process advisory includes up to 1,200 recent redacted log bytes, explicitly marked as replayed. Incremental cursors are unchanged. This keeps an earlier input prompt visible when a later status has no new output; it is not automatic prompt classification, input submission, cancellation or a new wait limit. Ownership, cancellation and explicit command deadlines still apply.

Existing skills cover unattended setup, restarting edited owned services, minimal cross-target I/O, proven numerical primitives, failing assertions and query-only information boundaries. These changes add no model/reviewer. The five measured trials use the earlier frozen binary; post-round changes have local regression validation only. See the [audit](../benchmarks/fenix-tasks-27-31-luna-low-2026-09-26.md).

## TerminalWorld, Terminal-Bench 4.0 and Gaia2

The shared native engine now supports two evaluation lifecycles: isolated terminal
attempts and an opt-in interactive Unix-socket session. Production chat behavior is
unchanged. The latter preserves one runtime/database between externally delivered
notifications, deduplicates delivery IDs, forwards genuine clarification replies
through the normal runtime and remains available after a completed turn. It ends
only on external stop, disconnect, cancellation or its configured deadline. A
transport failure is never recorded as a passing task.

### Reproducible selection

`scripts/benchmarks/prepare.py` downloads one task from an immutable upstream
revision into a **new** directory. Its `selection.json` records the selection rule,
revision, task ID and SHA-256 of every downloaded file. Indexes start at zero:

- `terminalworld`: official `verified` JSONL order (200 tasks in the pinned release).
- `terminal-bench-4.0`: sorted task directory names in the upstream Git tree.
- `gaia2-cli`: sorted scenario IDs within an explicit split (default: `execution`).

Example, using the official public datasets:

```bash
python scripts/benchmarks/prepare.py terminalworld \
  --revision dda7c099cc076735aef28c03bf8d3624dc0564e1 \
  --output /private/evaluations/terminalworld-first
python scripts/benchmarks/prepare.py terminal-bench-4.0 \
  --revision 452bf305c6daa62fc59061d22133a7cbc7c1572e \
  --output /private/evaluations/tb4-first
python scripts/benchmarks/prepare.py gaia2-cli \
  --revision 240ac47bfc62b24c934f95d014ffa5a3cab8b04c \
  --split execution --output /private/evaluations/gaia2-first
```

Preparation needs Python 3.12+ (`pyarrow` additionally for Gaia2). Archives reject
path traversal and links. They include official private grading inputs: never copy
these directories into the agent workspace, production skills or its context.

### Harbor datasets

`harbor/run_native.py` takes `--selection`, `--template`, `--bundle`, `--harbor`
and `--output`. It writes a one-attempt job with no task retry and invokes the
existing `NativeFenixAgent`. The prepared task's official resources, deadline,
instruction, artifacts and verifier remain unchanged. Harbor 0.23.0 supports the
separate verifier container used by Terminal-Bench 4.0. A 28,800-second agent
ceiling is a limit, not an expected duration; do not replace it with an arbitrary
shorter timeout and report a comparable official score.

For constrained development hosts, `--override-cpus N` explicitly records a
resource deviation in the Harbor job. It never changes the task or grader, but
timings must not be presented as directly comparable with the official allocation.

### Gaia2 CLI transport

This integration targets the **official Gaia2-CLI variant**, not a claim of
identity with the original ARE Python tool-call leaderboard. Pin the ARE checkout
and build the native image with `scripts/benchmarks/gaia2/build.py --upstream ...
--revision ... --bundle ... --image ...`. The initial supported revision is
`87ebd38f31aafae0f11e14f55617903196236cfb`. The upstream base image, CLIs, scenario
initializer, event daemon, prompts and judge remain upstream implementations.

Run `gaia2/run_native.py` with `--selection`, `--template`, `--output`, `--image`
and `--judge-url`. Its host Python environment must expose the upstream
`gaia2-cli/runner`, `cli` and `core` packages and dependencies. Docker replaces
Podman; the adapter port is published to loopback on a bridge network. The
transport calls the real `fenix eval` binary, not an alternate Python agent.

The agent runs under its own Unix user; the scenario, oracle and mutable benchmark
state belong to the separate `gaia2` user. Only the upstream setuid CLI wrapper
mediates application actions. Fenix's sanitized command environment is retained;
the environment instructions explain the absolute CLI paths. Only final responses
and clarification questions mark Gaia2 turn boundaries; progress messages remain
in Fenix's native trace. The benchmark daemon supplies subsequent notifications.

Use `interaction_socket` only with a local evaluation controller. The JSON-lines
protocol is `message {run_id,text,notification?}` / `stop` in and `ready` /
`response {run_id,state,message}` out. Duplicate `run_id`s do not replay effects.
Notifications are queued as new work and cannot answer a pending clarification.
The controller owns the scenario end, including idle periods awaiting events.

The judge may need its own model calls. The recording gateway also accepts Chat
Completions for that judge, with the same quota reservations as Responses. Keep
agent, judge, setup checks and missing usage separate in cost reports. Our first
integration uses GPT-6 Luna low for both agent and configurable judge (temperature
1 is required by this model). This is **not** the calibrated reference judge and
must be labelled when interpreting the result. A successful native turn does not
supply or override the official verdict.

The judge startup probe now exercises the actual upstream LiteLLM engine before
starting the agent. `judge_daemon.py` drops unsupported transport defaults so the
upstream fixed temperature 0 cannot override the configured temperature 1 for
Luna reasoning. Grading rules and prompts remain upstream. Judge setup probes
consume a small number of tokens and must be distinguished from task inference.

`gaia2/rejudge.py` can regrade an immutable **single-turn** saved trace following a
judge transport failure. Run it in the pinned image with `--scenario`, `--trace`
(events.jsonl and user_details.json), a new `--output`, `--model`,
`--reasoning-effort` and `--judge-url`. It hashes the input, never runs the agent or
replays actions, and writes a separate verdict. Preserve the original result.

The [first three-task audit](../benchmarks/three-benchmarks-luna-low-2026-09-26.md)
reports the frozen binary, local resource deviation, judge compatibility repair,
token accounting and the distinction between a grader pass and complete execution.

Sources: [TerminalWorld](https://github.com/EuniAI/TerminalWorld),
[Terminal-Bench 4.0](https://github.com/harbor-framework/terminal-bench/releases/tag/v4.0.0),
[Gaia2 CLI](https://github.com/facebookresearch/meta-agents-research-environments/tree/main/gaia2-cli).

## Recovery evidence and quota estimates

A delegated worker's finish tool closes its entire assignment, not an intermediate
investigation step. Unfinished analysis alone is not a terminal blocker. Workers
can checkpoint meaningful findings with `task_contract` and request
`task_inspect` with `recent: true` to retrieve the latest execution receipts before
resuming a previous worker. This bounded view excludes provider transcripts and
recursive inspection outputs; existing task/project authorization still applies.
Historical evidence must be reconciled with current files before replaying effects.
Forward event pagination and lexical output search remain available unchanged.
The existing explicit-outcome gate also applies after read-only tool work and
discovery, preventing a diagnostic-only plain-text response from being recorded
as completed. Tool-free conversation and active delegation handoffs are unchanged.

Incomplete milestone audits enter the existing scoped repair-review path once per
executor. They remain blocked until a plan revision resolves the findings; neither
success nor a replay of prior effects is automatic. Explicit `needs_user` verdicts
and unanswered auditor questions keep their pause. Missing observation references
identify the exact call and entry so the auditor can correct its citation.
Plan reviews distinguish future work from completed deliverables: missing outputs
of a pending milestone do not by themselves make its plan invalid. Coordinators
receive the assigned workers' evidence references and exact `role/finding_id`
resolution keys. Validation errors identify missing keys or invalid evidence;
successfully applying a revision/resolution is the coordinator's explicit outcome,
so it can end normally without an unavailable execution-finish tool. These changes
do not waive completion audits, owner questions or task/project isolation.
Completion-audit overviews explicitly identify milestone versus integrated scope
and include the assigned outcome immediately. Pending ledger entries are inputs
for verification, not proof that a milestone failed; later repair work is checked
when that work completes and again at the integrated gate.
Coding guidance also checks changing-input invariants and interacting constraints
with small contract-derived probes, rather than treating one passing example or a
failed implementation as proof of a general rule or an impossible requirement.

The optional evaluation gateway treats serialized request size as a quota estimate,
not an exact token count. An oversized estimate reserves at most one full observed
quota bucket and waits for availability; it does not manufacture a local provider
failure. OpenAI's response headers and Retry-After remain authoritative. Record
both estimated and reserved tokens when auditing transport delays.
Admission waits are serialized so later small requests cannot indefinitely consume
the quota needed by a waiting large request. Already-admitted API calls remain
concurrent. Infrastructure interruptions must be reported separately from agent
failures and repeated in fresh environments; do not pause a live task container
to drain the gateway, since its process probes require an available container.
