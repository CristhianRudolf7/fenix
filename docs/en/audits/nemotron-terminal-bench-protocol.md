# Nemotron harness audit: execution protocol

Status: complete. See the [results report](nemotron-terminal-bench-audit.md).

## Frozen baseline

- Fenix: working tree based on `b4a6709de669e64479c5833293a0033e2bc49e2f`, including the completed adaptive-memory implementation. No agent algorithm changes during baseline execution.
- Binary SHA-256: `19912a0d8158da5cc5428328c52d7cd2bd12f895a204c8be6a26fadad9ba680d`.
- Official Terminal-Bench 2.1 checkout: `7131e4375048a0e408a8fb404b5f499d726b695b`.
- Harbor 0.23.0; reference: built-in Terminus 2.
- Model: `nemotron-3-5-lightning-30b`, RunInfra, default thinking enabled (verified by reasoning returned in preflight). Same model and default sampling for both harnesses.
- Twenty distinct official tasks, one attempt per harness. The selection below was fixed before seeing any task outcome. It is a diagnostic subset, not a full Terminal-Bench score or leaderboard submission.

## Controls

Each task gets a fresh official container and Fenix data/skills directory. The runtime uses its real HTTP submission, planner, memory, tools and subagents. The controller never supplies solutions or executes task-solving commands itself. The test files are introduced only by the official verifier after agent execution. No private user memory or skills are copied into containers.

Tasks retain their original 1 CPU / 2 GiB limits and timeouts. At most two trials run concurrently. Fenix retains its default 32,768-token total context setting; reference model metadata declares 32,768 maximum input tokens. Both use a 16,384-token output ceiling and 128 foreground steps/turns. These numeric settings do not make context policies identical: Fenix estimates serialized context bytes, reserves output space and may lower the per-call output cap, while Terminus uses its own token accounting and summarization. RunInfra exposes the same actual 262,144-token model capacity to both. The internal 2,048-token Fenix compaction cap is deliberately left unchanged in the baseline. Fenix's existing subagent and background behavior is retained; those additional calls are measured, not excluded. These are baseline runtime settings, not a monetary budget. No external provider spending cap is applied.

Commands and writes are preauthorized inside the isolated container. If Fenix has only tasks waiting for user input for 15 seconds and no running or queued task, the controller ends the unattended attempt and runs the verifier. It does not invent user answers. This tests unattended completion, not an interactive user-assisted workflow.

The adapter transports model requests through a recording proxy. Fenix's HTTPS policy is preserved using a loopback tunnel; the real API key remains on the host. Both paths forward the same provider requests without modifying prompts or outputs. The initial adapter setup failure, which made zero API calls, is retained but excluded from task accuracy.

Before reaching the service tasks, adapter lifecycle handling was corrected: on normal turn completion the Fenix runtime stays alive for the verifier, preserving managed persistent servers; the controller exits and closes its inference tunnel. On an agent-phase timeout the remote controller is explicitly terminated, so cancelling a local Docker client cannot leave an agent solving tasks during verification. The first five tasks do not require persistent services. From task 6, the controller also checks active plans and waits for two seconds of terminal quiescence to let pub/sub completion handlers run. That observation delay is recorded in runner.json and is part of Harbor wall time, not model time. The first five attempts did not create plans or subagents.

The binary's dynamic loader and runtime libraries are copied alongside it to avoid replacing the container's system libraries. Python is installed only if needed for the HTTP controller. Agent-specific setup time and environment provisioning are reported separately from execution time.

## Measurements and audit

Preserve official verifier rewards and failures, task checksums, requests/responses, runtime events, tool arguments/results, task statuses, token usage and provider-reported billing. Original context is recoverable from recorded requests. Starting with task 4, the controller also exports checkpoint records directly; the first three tasks retain checkpoint metadata and API transcripts, not a separate database export. Provider API time is measured per request; its sum is not wall-clock execution time when calls overlap. Inference time includes network and proxy transport, and is not a pure model-compute measurement.

Provider rates observed at execution: $0.05 / million uncached input tokens, $0.01 cached input, $0.15 output. Prefer response `usage.cost` over inferred pricing. Reference-library cost estimates may omit cache discounts; use the shared proxy's provider figures for comparisons. Fenix's setup connection-check call is identifiable and counted in total audit spending.

Classify failures by infrastructure/provider, instruction interpretation, planning/clarification, tool discovery/arguments, terminal execution/output, context handling, verification and premature completion. A reference success with a Fenix failure is a diagnostic lead, not sufficient proof of causality. Controlled repeats must be labeled separately from the prespecified baseline.

## Selected tasks

| Task | Category | Difficulty | Agent timeout (seconds) |
| --- | --- | --- | ---: |
| cancel-async-tasks | software-engineering | hard | 900 |
| fix-git | software-engineering | easy | 900 |
| db-wal-recovery | file-operations | medium | 900 |
| multi-source-data-merger | data-processing | medium | 900 |
| build-cython-ext | debugging | medium | 900 |
| build-pmars | software-engineering | medium | 900 |
| cobol-modernization | software-engineering | easy | 900 |
| kv-store-grpc | software-engineering | medium | 900 |
| polyglot-c-py | software-engineering | medium | 900 |
| polyglot-rust-c | software-engineering | hard | 900 |
| git-multibranch | system-administration | medium | 900 |
| configure-git-webserver | system-administration | hard | 900 |
| nginx-request-logging | system-administration | medium | 900 |
| openssl-selfsigned-cert | security | medium | 900 |
| sqlite-db-truncate | debugging | medium | 900 |
| large-scale-text-editing | file-operations | medium | 1200 |
| log-summary-date-ranges | data-processing | medium | 900 |
| constraints-scheduling | personal-assistant | medium | 1200 |
| adaptive-rejection-sampler | scientific-computing | medium | 900 |
| fix-code-vulnerability | security | hard | 900 |

## Local artifacts

Raw artifacts, credential material, dependencies and official dataset remain ignored under `.fenix/audits/terminal-bench-20260915/`. Reusable integration code is in `scripts/benchmarks/harbor/`. No credentials are committed. The manifest, baseline hashes, job locks and configuration files make each attempt identifiable.

Sources: [Terminal-Bench 2.1](https://www.tbench.ai/news/terminal-bench-2-1), [official dataset](https://github.com/harbor-framework/terminal-bench-2-1), [Harbor agent integration](https://www.harborframework.com/docs/agents), [RunInfra model and pricing](https://runinfra.ai/inference-api/nemotron-3-5-lightning-30b).
