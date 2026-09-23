# Coding harness reliability

Deployed on September 22, 2026. These changes address the six concrete findings from
the September 21 Luna medium comparison; they do not claim a benchmark speedup.
Both harnesses passed that small evaluation. Repeating tests after the ledger
implementation changed was correct, not evidence of lost compaction context.

## Commands and execution contracts

`execution.command_yield_ms` defaults to 1000 ms. Recent successful short processes
from the same task and directory can extend the initial foreground wait to their
observed duration plus 200 ms, capped at 10 seconds (an explicit configured wait
can be up to 30 seconds). Interactive and persistent processes yield within
100 ms. `command_run.yield_ms` explicitly overrides the initial wait, including
zero. None of these settings shortens the process lifetime or changes approval.
Waiting is asynchronous and does not occupy a server thread.

`execution_mode: workspace` is the default for normal tests/builds, including
scratch writes. `input_paths` declares existing inputs inside the task workspace;
it does not change permissions. `execution_mode: read_only` requires explicit
inputs and preserves the existing no-write/no-network enforcement. It cannot be
interactive or persistent. Legacy nonempty `read_paths` retains read-only behavior;
combining it with mutable mode or `input_paths` is rejected, never silently upgraded.
Execution-control errors and nonzero program exits have separate structured
receipts. A program exit alone cannot prove whether a permission denial caused it.
No automatic permission relaxation or failed-command replay is introduced.

## Processes, environment and tools

New process IDs have the form `p_` plus sixteen hexadecimal characters. Existing
UUID manifests/logs remain readable. IDs remain opaque, exact, task-owned
references. Unknown IDs return accessible current process references without
approximate matching, auto-cancellation, or rerunning commands. Completed output
remains recoverable through the existing log API.

`environment_inspect` reads bounded manifest excerpts and probes executable paths
inside the actual execution environment, including project virtualenv paths. It
does not run project scripts or install packages. Manifest declarations are not
proof dependencies are installed. Results are cached per task for up to five
minutes, invalidated by manifest changes and mutable commands, and can be refreshed
explicitly. An external environment change can require refresh. Tool policies,
project boundaries, sandboxing and redaction still apply. The fixed probe does not
execute discovered programs or use a model.

Directory listing, glob matching, reading, content search, patching, environment
inspection and declarative tool programs are immediately discoverable where the
task's permissions permit them. Empty content searches are rejected by both the
published schema and argument validator. Declared scalar and array bounds are
validated before dispatch.

Independent file reads/listings/searches and public web reads can share a parallel
batch. Results remain in call order and each completed result is journaled. Mixed
batches with mutations retain sequential ordering; commands are not automatically
classified as safe to parallelize. Multi-file patches remain the coordinated edit
operation. Declarative programs preflight reference dependencies before any step;
use `{"$ref":"step_id/process_id"}` rather than transcribing process IDs. Existing
write/command completion and failure barriers continue to apply.

## Validation

Regression coverage checks foreground completion, explicit execution modes, exact
process references and task filtering, cached environment evidence, schema bounds,
preflight rejection before file writes, and model-facing read batches. These tests
use local processes and a simulated provider, not paid model inference. A repeat of
the same benchmark and an independent holdout are still needed to quantify any
change in model accuracy, tokens or latency. No mandatory per-step auditor or
context-budget change is part of this update.
