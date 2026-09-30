# Harbor audit integration

[Español](../../../docs/es/audits/nemotron-terminal-bench-protocol.md) · [English protocol](../../../docs/en/audits/nemotron-terminal-bench-protocol.md)

These utilities run the real Rust Fenix runtime inside official Harbor task containers. They are evaluation adapters, not another implementation of the agent.

| File | Responsibility |
| --- | --- |
| `fenix_agent.py` | Harbor setup, submission and artifact collection; diagnostic subclass changes only the recording route. |
| `container_runner.py` | Start an isolated Fenix instance, configure it through its HTTP API, submit the original task and observe completion. |
| `provider_proxy.py` | Forward and record RunInfra chat requests for explicitly permitted models (default: Nemotron only) while keeping the real provider key on the host. |
| `recorded_terminus.py` | Assign a separate recording URL to each stock Terminus 2 trial. |
| `run_audit.py` | Original sequential paired-task driver for this audit. |
| `run_queue.py` | One-time continuation from task 8 with two occupied Harbor slots. Do not restart against an existing run. |
| `cleanup_completed.py` | Remove only audit images after both baseline harnesses have finished, protecting images present before the audit. |
| `probe_compaction.py` | Replay exact recorded summary inputs at two output ceilings. |
| `summarize.py` | Export official outcomes, runtime events and provider billing, retaining explicit exclusions and diagnostic labels. |

The private run directory contains the pinned dataset, frozen binary and libraries, `provider.key`, `selection.json`, Harbor configurations, initial image inventory and all raw recordings. It lives under the repository's ignored `.fenix/audits/` directory. Do not copy it into the public report or commit it.

Use Harbor 0.23.0 and the recorded dependency versions for this run. Set `FENIX_AUDIT_ROOT` to the run directory, add this directory to `PYTHONPATH`, and put `TMPDIR` on a filesystem with enough space. The Docker bridge address and localhost ports in the adapter match this audit host; adapt them before using another host.

Start the normal proxy with `provider_proxy.py RUN_DIRECTORY`. The optional `--summary-output-cap 16384` intervention applies **only** to `fenix-cap-*` requests with the exact active-compaction system prompt and an original 2,048-token limit. Original and forwarded payloads are retained separately. Never enable an intervention silently in baseline results.

Once all trials have finished, run `summarize.py RUN_DIRECTORY`. Costs come from recorded response usage, not from the reference harness's potentially incomplete retry accounting. Missing usage is not proof that a request was free. `baseline-exclusions.json` lists excluded setup failures by exact trial ID; task timeouts and agent-induced environment failures remain failed baseline attempts.

Official verifier files enter the environment only after the agent phase. Preserve persistent services through verification, but stop inference at the phase boundary. Keep the original instruction and grader unchanged. The protocol documents the adapter corrections made during the run and the limits of comparison.

## Native execution

For new evaluations, use `native_fenix:NativeFenixAgent` and the Rust `fenix eval` command. See [native evaluation](../../../docs/en/native-evaluation.md) / [evaluación nativa](../../../docs/es/native-evaluation.md). The original HTTP controller remains for reproducibility of the September 15 baseline; its frozen results are not results for the updated engine.

`run_native.py` runs one pinned selection produced by `../prepare.py`, including
TerminalWorld Verified and Terminal-Bench 4.0. `../gaia2/` connects the same native
Rust runtime to the official Gaia2 CLI multi-turn environment. See the bilingual
native evaluation guide for version pins, judge caveats and reproduction commands.
