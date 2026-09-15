# Native harness integration check — 2026-09-15

[Español](../../es/audits/native-harness-smoke.md) · [Evaluator guide](../native-evaluation.md)

After implementing native evaluation and the six execution improvements, the
workspace passed **234 tests**, with four optional tests ignored. Clippy passed
with warnings denied, documentation tests passed, and the frontend syntax check
passed. Regression coverage includes oversized complete tool batches, failed
summaries, unexecuted truncated actions, bounded recovery, persistent background
jobs, process waiting/readiness, progress-aware loops, durable replies and task
pagination beyond the interface's 200-task limit.

A separate integration attempt used the official Terminal-Bench 2.1
`log-summary-date-ranges` task, pinned dataset revision
`7131e4375048a0e408a8fb404b5f499d726b695b`, Nemotron
`nemotron-3-5-lightning-30b` through RunInfra, the native Rust evaluator and Harbor
0.23.0. Official resources, instruction and verifier were retained. The native
mode uses a fresh search corpus without the interactive server's background
crawler; no web tools were used in this attempt.

| Observation | Result |
| --- | --- |
| Official reward | 0/1 |
| Native measured execution, including terminal quiescence | 20.825 s |
| Provider requests, including setup | 6 |
| Input tokens, including cached input | 46,687 |
| Cached input tokens | 17,408 |
| Output tokens | 3,810 |
| Provider-reported cost | $0.00220953 |
| HTTP errors | 0 |
| Recorded tool calls | 4 |
| Active context compactions | 1, successful |

The evaluator exported its report and handed off to the official verifier.
Compaction reduced estimated context from 16,843 to 12,819 tokens. The model
listed the log directory, tried an unsupported `limit` argument, then repeated
the same listing. The guard stopped the third identical result without observed
progress. The requested output was not produced. This is a failed task and a
working integration handoff, **not evidence of an improved benchmark score**.

The native report covers five foreground/summary responses: 46,667 input and
3,717 output tokens. Provider totals above additionally include the setup check.
The first orchestration launch stopped during Docker preflight without an API
request; the service was confirmed active and the launch was retried without
restarting it. The recorded job took about 2m30s including environment lifecycle,
which is distinct from agent execution time.

The original 20-task audit is unchanged. This single attempt is neither a repeat
of that audit nor a comparison with Terminus 2. A new paired evaluation and
unseen tasks are required to measure a quality change.

Private local artifacts are under
`.fenix/evaluations/native-smoke-20260915/`: source fingerprints, executable
manifest, official verifier results, native traces and provider accounting. They
are ignored by Git. The temporary proxy and task container were stopped; only
the image introduced for this check was removed.
