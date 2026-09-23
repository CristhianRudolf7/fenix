# Native evaluation and execution recovery

[Español](../es/native-evaluation.md)

Fénix includes `fenix eval` in its Rust executable. It runs the real assistant,
providers, tools, context management and background workers with a fresh database
and built-in skills. The benchmark owns its task containers and official verifier.
A completed assistant turn is **not** a passing benchmark result.

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
  "benchmark": "terminal-bench-2.1",
  "dataset_revision": "7131e4375048a0e408a8fb404b5f499d726b695b",
  "task_name": "log-summary-date-ranges",
  "instruction": "The exact official instruction, without hidden tests",
  "api_key_file": "/opt/fenix-eval/proxy-key",
  "timeout_seconds": 900,
  "waiting_seconds": 15,
  "proxy_upstream": "172.17.0.1:48931",
  "settings": {
    "provider": "openrouter",
    "model": "nemotron-3-5-lightning-30b",
    "base_url": "http://127.0.0.1/native-trial/v1",
    "workspace": "/app",
    "command_policy": "allow",
    "file_write_policy": "allow",
    "reasoning_effort": null
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
