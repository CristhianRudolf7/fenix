# Terminal-Bench 4.0: 15 tasks with GLM-5.3-Flash and Nemotron on RunInfra

[Español](terminal-bench-15-runinfra-2026-09-29.es.md) · [Trial ledger](terminal-bench-15-runinfra-2026-09-29.json)

Status: complete. Original run: **1/15** official passes. After general harness fixes,
labeled repeat attempts of the same tasks: **5/15**. Repeats never replace the original
results; they use different frozen binaries and are not a controlled ablation.

## Results (official verifier; check counts are partial diagnostics)

| Task | Original (valid) | After changes |
| --- | --- | --- |
| `atrx-vep-crispr` | 0 (12/16) | pass (16/16) |
| `batched-eval-parity` | 0 (0/5) | pass (5/5) |
| `bun-sourcemap-leak` | 0 (31/36) | 0 (27/36) |
| `cargo-flight-dispatch` | 0 (9/27) | 0 (22/27) |
| `coq-block-bound` | 0 (2/3) | 0 (2/3) |
| `embedding-drift-monitor` | 0 (10/11) | pass (11/11) |
| `fin-saccr-rwa` | 0 (0/24) | pass (24/24) |
| `foodstuff-beta-activity` | 0 (0/13) | 0 (10/13) |
| `freight-dispatch-shift` | 0 (no CLI built) | 0 (101/232 diagnostic pts) |
| `glycan-ms2-elucidation` | 0 (2/12) | 0 (11/12) |
| `heat-pump-warranty` | 0 (0/20 cases) | 0 (15/20 cases) |
| `kv-live-surgery` | 0 (0/1) | 0 (2/6) |
| `music-harmony` | 0 (no submission) | 0 (3 rule violations) |
| `mvcc-lsm-compaction` | pass (15/15) | pass (15/15) |
| `nextjs-performance` | 0 (1/5) | 0 (1/5) |

## Method

Terminal-Bench 4.0 `v4.0.0` (`452bf305…`), Harbor 0.23.0, the native Rust `fenix eval` inside
each official task container, one attempt per trial, official instructions, resources,
deadlines and verifiers unchanged, trials run sequentially on a 2-vCPU host. Selection was
fixed before outcomes: the first 15 lexicographic tasks requesting ≤2 CPU, ≤4096 MiB and no
GPU. The three FreeCAD tasks were ungradable on the evaluation date (their official verifier
image fails to build because pip cannot replace conda's `vtk 9.2.6`) and were replaced by
the next eligible tasks before any replacement outcome was known.

Roles mirror production: intelligent/master `glm-5-3-flash`, fast/conversational
`nemotron-3-5-lightning-30b`, both via RunInfra with provider-default reasoning, a 200,000-token
context and a 16,384-token output ceiling. Native evaluation now configures the fast role
(`EvaluationConfig.fast`); it was exercised by progress narration in every trial. Provider
keys stayed on the host behind a recording proxy.

## Findings and changes

Most original failures were harness failures. Reasoning often exhausted the output ceiling;
a lifetime cap of three `output_length` recoveries then aborted productive tasks, and the
provider client rejected responses whose plain chat reasoning mentioned an `Authorization:`
header. Changes: consecutive-only no-action budget, escalated guidance, the latest plain-text
reasoning carried forward once as unverified notes (8/9 executable actions in a replay of
recorded contexts versus 0/9 before), recovery of reasoning-only chat responses, masking of
unsigned chat replay, and umask-based modes for new workspace files. See
[native evaluation](../en/native-evaluation.md).

Infrastructure issues were separated from agent results: inherited `umask 077` made prepared
task files unreadable to a privilege-dropping verifier (task rerun with the original binary),
a disk-full verifier build (attempt archived), and one accidental 30-minute trial overlap
(the affected original task was rerun sequentially).

Usage across 37 trials: 1,613 GLM and 835 Nemotron requests, $6.67 provider-reported
cost (7 provider `503 limiter_unavailable` responses across all proxied traffic, 2 of them at startup and archived as infrastructure).
