# Nemotron harness evolution — final audit

[Español](../../es/audits/nemotron-harness-evolution.md) · [Per-attempt evidence](nemotron-harness-evolution-results.csv)

**The adoption target was not met.** The first context candidate improved confirmation from **0/20 to 2/20**, both repetitions of one task. Three subsequent cycles did not demonstrate an additional reproducible gain over that candidate, so the predeclared stopping rule was applied. The experimental PR remains a draft, without automatic merging.

The study ran on September 15–16, 2026: **70 attempts, 65 valid and five explicit transport exclusions**. Recorded provider spend was **US$2.13128346**, including **US$0.07315928** for excluded inferred attempts. This includes connection probes and summaries, not machine or development costs. Every provider call has numeric billing; the two zero-call setup failures incurred no inference charge.

## Protocol

Reference: main `574e390`; real Rust native evaluator; Terminal-Bench 2.1 revision `7131e4375048a0e408a8fb404b5f499d726b695b`; Harbor 0.23.0. All arms use RunInfra `nemotron-3-5-lightning-30b`, provider-default reasoning, the same settings and official instructions/verifiers. No hidden tests or reference solutions were supplied to agents or skills. Verifier output was inspected afterward for audit only.

The configured context capacity is 32,768 tokens, output ceiling 16,384 tokens and model-step ceiling 128; recovery remains bounded. Native time limit is 900 seconds, even where an official task permits more. At most two trials run concurrently. Environment setup and verification are separate from native execution. Shared-host build/Docker activity limits timing comparisons; cycle 4 ran after compilation finished. These are controlled subset results, not an official full-budget leaderboard score.

Development uses the same five tasks: `custom-memory-heap-crash`, `extract-elf`, `modernize-scientific-stack`, `schemelike-metacircular-eval`, `sanitize-git-repo`. Confirmation uses ten preselected unseen tasks, two attempts per version: A — `build-pov-ray`, `prove-plus-comm`, `regex-chess`, `sqlite-with-gcov`, `mailman`; B — `circuit-fibsqrt`, `make-mips-interpreter`, `write-compressor`, `sparql-university`, `raman-fitting`.

Acceptance required at least **two additional passes per ten tasks on average**, a positive gain in both repetitions, and passing regressions. Actual gains were **+1/10 and +1/10**. The further unseen reserve was not used: later development candidates did not improve on cycle 1. Small samples, repeated tasks and stochastic outputs do not establish broad statistical superiority.

## Development results

| Variant / commit | Passes | Calls | Input tokens | Output tokens | USD | Native seconds |
|---|---:|---:|---:|---:|---:|---:|
| main / `574e390` | 0/5 | 78 | 626,763 | 188,884 | 0.05044451 | 535.0 |
| cycle1 / `ea8b673` | 2/5 | 306 | 3,871,562 | 621,873 | 0.20582481 | 1,689.5 |
| cycle2 / `d95f8c1` | 1/5 | 158 | 1,906,808 | 253,367 | 0.09269777 | 748.0 |
| cycle3 / `f1efd24` | 1/5 | 328 | 4,293,553 | 633,872 | 0.21906277 | 2,149.7 |
| cycle4 / `4d4b534` | 2/5 | 402 | 5,304,397 | 768,083 | 0.26310238 | 2,499.1 |

Each row contains five valid attempts, including declared replacements; excluded inference spend is reported separately. Seconds are summed, not concurrent wall time. Cycles 1 and 4 passed extraction and modernization. Cycles 2 and 3 passed only modernization. Cycle 4 therefore matched the best development score at higher recorded cost and accumulated time.

No `skill_search` or `skill_load` occurred in the **25 valid development attempts**, although the catalog/schemas were available. This shows non-use in this sample, not that the skill bodies themselves are ineffective. One search occurred in main confirmation; no skill load occurred there. A broader catalog alone did not demonstrate a benefit.

## Confirmation results

| Metric, 20 valid attempts | Main | Cycle 1 |
|---|---:|---:|
| Passes | 0 | 2 |
| Provider calls | 707 | 1,387 |
| Input tokens | 6,037,494 | 17,730,132 |
| Output tokens | 1,533,464 | 2,065,696 |
| Recorded provider cost, USD | 0.42910006 | 0.79789188 |
| Accumulated native seconds | 3,979.4 | 6,907.5 |

Only `prove-plus-comm` passed, in both cycle 1 repetitions. Removing premature compaction failures allowed more work, but most tasks still failed and total cost/time rose.

| Outcome | Main | Cycle 1 |
|---|---:|---:|
| Empty-tail compaction failure | 9 | 0 |
| No-progress recovery exhausted | 8 | 2 |
| Model-call limit | 0 | 4 |
| Timeout | 1 | 1 |
| Turn completed, verifier failed | 2 | 11 |
| Official verifier passed | 0 | 2 |

## Interventions and observed limits

1. **Context admission and empty-tail compaction (`ea8b673`).** Reserve a useful reasoning allowance instead of the full output ceiling for every admission; still clamp output to remaining estimated capacity. Permit a complete cut when no recent tail fits; request concise summaries. This removed the nine observed empty-tail failures, but the held-out gain remained below target.
2. **Skill selection and procedures (`d95f8c1`).** Add up to two lexical metadata suggestions, a software-implementation skill, and revised repair/Git procedures. Bodies remain on demand. Weighted term frequency, inverse document frequency and length normalization improve lexical selection tests. Owner-edited files and disabled state survive exact-content migrations. No actual skill loads were observed in development, so procedure quality was not established by these runs.
3. **Tool evidence and replay accounting (`f1efd24`).** Preserve head/tail previews, execution/exit/cursor metadata, repository guidance and access to complete redacted tool receipts. Count the larger of normalized content and provider replay instead of both; retain hidden/signed reasoning. Regression tests pass, but development score did not improve. Byte estimation remains a heuristic, not a model tokenizer or billing meter.
4. **Evidence-first clarification and basic writes (`4d4b534`).** Check original requests and local facts before asking; still surface every unresolved doubt. Expose basic create/edit schemas immediately while keeping permissions and advanced discovery. Revise environment guidance to distinguish diagnostic failure from application failure. This recovered 2/5, without exceeding cycle 1.

Cycle 4 failures illustrate remaining gaps: C++ timed out; Scheme exhausted no-progress recovery; Git made verified edits and declared completion, but failed secret-removal and replacement checks while preserving unrelated files. Tool-level success is not task-level success. Search reported 531 scanned files and one skipped error with ignore rules enabled; code also applies a 1 MiB file filter. Complete search coverage must not be inferred from zero matches. This is a coverage limitation, not proof of which filter caused the remaining secret.

Further hypotheses, **not implemented or validated in this study**: better progress accounting for repeated failed commands (new process IDs can look like progress); explicit search coverage/exclusions; a bounded longer initial command wait to reduce polling calls. The current 100 ms initial wait returned `running` for 11 of the first 19 extraction command receipts in cycle 4. Do not label those polls redundant: the processes were still reported running.

## Infrastructure corrections and exclusions

- Two original main proof attempts (`3YsqxDh`, `74yYYdz`) assumed `/app`, although the official container used `/workspace`. Both made zero provider calls. The adapter now reads `pwd`; replacements both failed.
- A ready-marker/process-exit race prevented grading three completed native reports: cycle 1 `write-compressor__FUsx9wm`, cycle 2 `sanitize-git-repo__zeZS8N6`, cycle 3 `custom-memory-heap-crash__3jTn3wC`. The adapter checks readiness again after process exit. Same-binary replacements all failed the official verifier. Their original inference costs remain in the total. Waiting for user input alone is not an infrastructure exclusion: a modernization artifact passed despite that runtime state.
- Missing, null, boolean or non-finite billing remains unknown instead of becoming a zero-cost result. Five transport/billing tests passed.
- Explicit glibc-loader launches made old internal manifests hash the loader. The external frozen bundle/source protocol has the correct Fénix hash and is authoritative for these runs. A subsequent metadata-only correction records the program hash and both executable paths; historical manifests remain untouched. This correction is not a new benchmark candidate.

## Validation and provenance

Main passed 239 Rust tests and 13 browser tests; four environment-dependent tests were ignored. Cycle 4 passed **114 agent tests**, including 23 runtime tests and all six planning-review tests; one environment-dependent test was ignored. An earlier broad run hit two unchanged planning-review timeouts under load. Both passed subsequently, and all six passed again in isolation in 9.11 seconds, without changing code or timeouts. Workspace Clippy, formatting and the cycle 4 build passed. Browser code did not change and browser tests were not repeated.

Final reporting validation passed all three evaluation unit tests, workspace Clippy and a real loader-launched native smoke test using two local mock responses (zero provider spend). The smoke test matched the program hash and distinguished it from the loader. These reporting-only changes were made after the frozen benchmark binaries; no additional performance claim is made for them. GitHub Actions was blocked by an account billing lock during the study; local checks must not be represented as a remote CI pass.

| Variant | Source | Frozen program SHA-256 |
|---|---|---|
| main | `574e390` | `a343bd8a6b1076162ed09928b31c9e3b87504a062799acde52a54df47a10b684` |
| cycle1 | `ea8b673` | `57b21e22b3093ff88d167632588490e80d30064c1fcb8cf08c17aab14cf8fdc5` |
| cycle2 | `d95f8c1` | `0700287d8716654d86fe5e676f9da3a957e3b8b733aa9d42c3472a2e0082b9e1` |
| cycle3 | `f1efd24` | `b468ee1ad82df3eb5c873cb9b434d0c810d83a736d17a2e029a4a669b61c1bec` |
| cycle4 | `4d4b534` | `43469b49f74a9b4f9b385616adb45132d105cd9a7ea88de9d1125e74199ff3c1` |

Configuration SHA-256: `6e124d093eb1d00885682116d7fa635125295d9fb7ae1fefdd0ac86b8b5523ae`. All frozen binary/configuration hashes were retained for verification. Private prompts, artifacts and API traces remain under ignored `.fenix/evaluations/harness-evolution-20260915/`; the CSV exposes only whitelisted metrics and provenance. Later reporting fixes do not change the evaluated binaries.

## Research basis

[AHE](https://arxiv.org/html/2604.25850v4) informed trace-driven hypotheses and held-out checks; [SkillsBench](https://arxiv.org/abs/2602.12670), [SkillRevise](https://arxiv.org/html/2606.01139v3) and [Group of Skills](https://arxiv.org/html/2605.06978v1) informed cautious procedure selection/revision. [Context Compaction Theory](https://arxiv.org/abs/2608.01326) motivates testing information retention, not the chosen byte heuristic. [Ask or Assume?](https://arxiv.org/html/2603.26233v1) studies clarification with a simulated user holding missing information; no such oracle was added here. [SWE-RPG](https://arxiv.org/html/2608.09072v1) informed separating requirement, planning and artifact failures. These sources motivate experiments; none proves a Nemotron/Fénix improvement.
