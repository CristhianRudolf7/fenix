# Nemotron harness evolution

[Español](../../es/audits/nemotron-harness-evolution.md)

## Protocol

The reference is main commit `574e390`, with the real Rust native evaluator. Terminal-Bench 2.1 is pinned to `7131e4375048a0e408a8fb404b5f499d726b695b`; Harbor is 0.23.0. Both arms use RunInfra `nemotron-3-5-lightning-30b`, provider-default reasoning, identical runtime settings, and official task instructions/verifiers. No hidden tests or reference solutions enter the agent prompt. Two trials run concurrently. Environment download/setup time is separate from native execution and verification time.

The development cohort contains `custom-memory-heap-crash`, `extract-elf`, `modernize-scientific-stack`, `schemelike-metacircular-eval`, and `sanitize-git-repo`. Development results guide general harness changes; they do not establish unseen-task improvement.

Confirmation uses two preselected five-task cohorts, with two attempts per version. Cohort A: `build-pov-ray`, `prove-plus-comm`, `regex-chess`, `sqlite-with-gcov`, `mailman`. Cohort B: `circuit-fibsqrt`, `make-mips-interpreter`, `write-compressor`, `sparql-university`, `raman-fitting`. Success requires at least two additional passes per ten tasks on average, positive gain in both repetitions, and passing regressions. Three consecutive cycles without reproducible gain stop the loop. Infrastructure exclusions must be explicit; model/tool failures count as failures.

Each arm freezes binary/configuration hashes. Costs use recorded provider usage; missing billing stays unknown. Private traces remain under ignored `.fenix/evaluations/harness-evolution-20260915/`. The experimental branch is reviewed separately and is not automatically merged.

## Evidence and interpretation

[Agentic Harness Engineering](https://arxiv.org/html/2604.25850v4) motivates recording a predicted mechanism before each edit and checking task-level regressions afterward. [SkillsBench](https://arxiv.org/abs/2602.12670) motivates concise, curated procedures, while its negative task deltas caution against treating a larger catalog as a guaranteed improvement. These are research hypotheses, not evidence that this implementation improves Nemotron.

## Validation and infrastructure

Reference validation: 239 Rust tests and 13 browser tests passed; four environment-dependent Rust tests were ignored. Clippy, formatting and documentation tests passed. The host `/tmp` volume was full, so browser and benchmark temporary files use a private directory on the main filesystem. GitHub Actions did not start because the account was locked for billing; this is not a CI test pass.

## Results

Reference: **0/5**. Provider totals: 78 calls, 626,763 input tokens, 188,884 output tokens, **$0.05044451**. Three empty-tail compaction failures; two no-progress recovery failures. Setup/download time is not included in native seconds. Candidate development and completed cycle 1 confirmation results are below.
| Task | Main pass | Native seconds | Input tokens | Output tokens | USD |
|---|---:|---:|---:|---:|---:|
| custom-memory-heap-crash | 0 | 64.1 | 95111 | 24587 | 0.007225 |
| extract-elf | 0 | 208.1 | 203788 | 66755 | 0.016982 |
| modernize-scientific-stack | 0 | 48.1 | 66619 | 20857 | 0.005850 |
| sanitize-git-repo | 0 | 112.0 | 173451 | 40970 | 0.011685 |
| schemelike-metacircular-eval | 0 | 102.6 | 87794 | 35715 | 0.008702 |


## Cycle 1: context admission

Before testing the candidate: the first two reference tasks failed (0/2). Extraction stopped with `No completed batch to compact`; the C++ repair reread the same files after compaction and exhausted recovery. The candidate caps the input-admission output reservation at 8,192 tokens, while retaining the configured output ceiling and clamping each actual request to available capacity. It also accepts a complete-context cut when no recent tail fits and asks for a concise summary. Prediction: fewer compactions and repeated observations, no empty-tail failure. Risk: less output room on large-context turns can increase truncation. Provider, model, configured context capacity and maximum output remain unchanged.

Related checks: [ARC](https://arxiv.org/abs/2607.25066) separates archived observations from active context, but its reported tasks are long-context QA, not this coding benchmark. [Context Compaction Theory](https://arxiv.org/abs/2608.01326) formalizes selection versus generated summaries; it does not prescribe an 8,192-token reservation. That value is an explicit engineering hypothesis tested here. The 2025 antecedent [The Complexity Trap](https://arxiv.org/abs/2508.21433) also motivates measuring masking against costly summaries rather than assuming summarization is superior.

## Cycle 1 development results

The candidate resolved **2/5**, against **0/5** for main. Totals: 306 provider calls, 3,871,562 input tokens, 621,873 output tokens, **$0.20582481**. Accumulated native time was 1,689.5 seconds versus 535.0 seconds. Longer successful/unsuccessful attempts cost more than early harness failures. This is not a speed improvement overall.

| Task | Candidate pass | Native seconds | Input tokens | Output tokens | USD |
|---|---:|---:|---:|---:|---:|
| custom-memory-heap-crash | 0 | 576.1 | 1250914 | 187942 | 0.062188 |
| extract-elf | 1 | 454.2 | 903475 | 163516 | 0.052119 |
| modernize-scientific-stack | 1 | 14.8 | 80982 | 2585 | 0.002783 |
| sanitize-git-repo | 0 | 588.1 | 1400213 | 251774 | 0.079315 |
| schemelike-metacircular-eval | 0 | 56.3 | 235978 | 16056 | 0.009420 |

Remaining failures: C++ compiled but still failed release execution and memory checks; the evaluator task ended in a mistaken clarification without creating the requested file; Git cleanup exhausted no-progress recovery. Inspecting code also found that `command_run` is counted as an effect even if its exit code is nonzero; this limitation remains unchanged in the frozen candidate. No hidden verifier content was added to prompts or skills.

The configured native task timeout is 900 seconds in both arms, even where the official outer task allowance is longer. Scores describe this controlled subset/configuration, not an official full-budget leaderboard result. The shared host was not reserved exclusively; wall time is descriptive. Frozen `source.json` files associate binary hashes with their actual source commits, independently of the runner checkout.

## Confirmation transport correction

`prove-plus-comm` uses `/workspace`, not `/app`. The original adapter failed before any provider request. The adapter now discovers the official container working directory with `pwd`; it does not create or modify task files. Two transport unit tests passed. The two zero-inference setup failures were excluded and replaced after the paired cohorts; both replacements failed. The original setup failures are not counted as model failures or evidence of improvement. Binary hashes and inference settings remain fixed.

## Skill audit

The user explicitly included skill improvements in the experimental scope. Across the ten development executions (five per arm), the native journals contain no `skill_search` or `skill_load` calls. Recorded inference requests confirm that both schemas and the manual catalog were available. This shows non-use in this sample, not that the skill bodies are ineffective. Adding more skills alone cannot establish a benefit when none are loaded.

A subsequent candidate may test clearer selection triggers, compact relevant recommendations, or procedural revisions supported by execution evidence. It must preserve owner-edited skills, on-demand loading, and fresh confirmation data. The current confirmation binaries and skills remain frozen. [SkillRevise](https://arxiv.org/html/2606.01139v3) motivates diagnosing whether a failure is attributable to a skill before editing it and validating the revision through execution. [Group of Skills](https://arxiv.org/html/2605.06978v1) motivates making the entry procedure and supporting checks explicit. Neither paper establishes a gain for Fénix or Nemotron; no benchmark-specific solutions will be embedded in skills.

## Cycle 2 candidate: skill selection and procedures

Prepared from development traces while cycle 1 confirmation remains frozen: add up to two task-local lexical metadata suggestions, keeping full skill bodies on demand; add a software-implementation procedure focused on the requested runnable artifact; refine repair guidance for observed memory faults and Git guidance for current files versus history. Prediction: more relevant skill loads, fewer unchanged observations, and fewer missing deliverables. Risks: irrelevant suggestions, extra prompt/tool cost and skill-induced regressions. This is a combined skill intervention; it cannot isolate the effect of selection from content.

Unmodified shipped repair/Git skills migrate by exact-content comparison; user edits and disabled state are preserved. The generic Codex skill validator rejects Fénix's existing `version` and `required_tools` extensions, so it is not a passing validation result; Fénix's parser and migration tests cover its native format. No current confirmation binary or task configuration changes. Benchmark outcomes for this candidate are pending.

The skill retrieval test exposed generic description words outranking a distinctive body term. The candidate now saturates weighted term frequency jointly and applies inverse document frequency and document-length normalization. This is deterministic lexical retrieval, with no embeddings or extra inference. Concurrent local builds on the shared host are an additional latency confound; timing is descriptive, not a throughput claim.

Candidate validation: 60 agent unit tests passed (one environment-dependent test ignored), 22 runtime integration tests passed, and workspace Clippy passed without warnings. The final five skill tests, including bounded Unicode queries and disabled/customized skills, passed after the last selector edit. Browser code is unchanged; browser tests were not repeated for this candidate.

## Clarification audit

Some cycle 1 confirmation runs ended with plain-text clarification questions instead of the requested artifact; the runtime marked those turns completed, while the verifier rejected the result. A tool-based question instead produced `requires_user_input`, also unresolved in this unattended benchmark. These are distinct from setup failures. Some questions concern facts available in task files or ordinary implementation choices; genuine unresolved owner requirements must still be surfaced.

[Ask or Assume?](https://arxiv.org/html/2603.26233v1) evaluates missing requirements using a simulated user supplied with withheld original information. It supports appropriate clarification, not suppressing all questions, and does not validate a Nemotron-specific remedy. [SWE-RPG](https://arxiv.org/html/2608.09072v1) motivates diagnosing requirements, planning and artifact submission separately from final pass/fail. A future intervention should improve evidence gathering and distinguish user intent from technical investigation without inventing requirements or supplying verifier answers.

## Cycle 1 confirmation result

All 40 valid attempts are complete: ten tasks, two repetitions per version. Main resolved **0/20 (0%)**; cycle 1 resolved **2/20 (10%)**, both repetitions of `prove-plus-comm`. Each repetition improved from 0/10 to 1/10. This is a narrow repeated gain, but it **fails the predeclared target of two additional passes per ten tasks on average**. The experiment continues; the development gain of 2/5 must not be presented as general performance.

| Metric, 20 valid attempts per version | Main | Cycle 1 |
|---|---:|---:|
| Passes | 0 | 2 |
| Provider calls | 707 | 1,334 |
| Input tokens | 6,037,494 | 16,999,044 |
| Output tokens | 1,533,464 | 1,954,413 |
| Recorded provider cost, USD | 0.42910006 | 0.76266231 |
| Accumulated native seconds | 3,979.4 | 6,603.2 |

All recorded calls have numeric billing. Native seconds are summed across attempts, not elapsed wall time for the concurrent study; environment setup and verifier time are separate. Binary hashes were rechecked.

| Outcome category | Main | Cycle 1 |
|---|---:|---:|
| Empty-tail compaction failure | 9 | 0 |
| No-progress recovery exhausted | 8 | 2 |
| Model-step limit | 0 | 4 |
| Timeout | 1 | 1 |
| Requires user input | 0 | 1 |
| Turn completed, verification failed | 2 | 10 |
| Official verifier passed | 0 | 2 |

The unresolved interactive attempt counts as a failure despite having no numeric verifier reward; it is not an infrastructure exclusion. Removing premature harness failures allowed more work, but did not make most results correct and increased total cost/time. Candidate 2 skill evaluation is underway on the original five development tasks.

## Cycle 3 candidate: preserve tool evidence and count replay once

The process layer preserves a head/tail preview, but the engine previously cut that serialized receipt a second time using only its prefix. This could hide final compiler/test errors. The candidate preserves both ends, explicit execution/exit/cursor metadata, complete repository guidance, and a link to the full redacted journal. Regression tests cover a final diagnostic, omitted middle recovery, secrets and UTF-8 boundaries.

Context estimation also serialized normalized assistant content plus its opaque provider replay, although provider codecs send the replay instead of both representations. The candidate counts the larger representation plus a wrapper allowance. Hidden/signed reasoning is still counted and retained; this remains a conservative byte estimate, not token billing. Prediction: fewer unnecessary compactions and fewer lost diagnostics. No model, configured context size, output limit, permissions or timeout changes. This is a combined mechanical intervention and remains unconfirmed.

The trace-driven iteration follows [AHE](https://arxiv.org/abs/2604.25850); [Context Compaction Theory](https://arxiv.org/abs/2608.01326) motivates testing information retention, not a specific byte heuristic. Neither source proves this candidate improves Nemotron. Development evaluation must precede any new held-out confirmation.

Cycle 3 validation: 64 agent unit tests passed (one environment-dependent test ignored), all 23 runtime integration tests passed, and workspace Clippy passed without warnings. The new integration test confirms that the model sees the final diagnostic while the complete redacted middle remains retrievable from the journal.

## Final-report transport race: pending correction

A later audit found that an evaluator can write `ready.json` and exit between the adapter's readiness and liveness probes. Two runs had complete reports but were never officially graded: cycle 1 `write-compressor__FUsx9wm` and cycle 2 `sanitize-git-repo__zeZS8N6`. The adapter now checks the final marker again after process exit; four transport tests pass. These two attempts will be replaced using their original frozen binaries/settings. The earlier cycle 1 table is therefore provisional (one ungraded attempt conservatively counted as failed), not the final accepted comparison. Preserve the excluded attempts' provider spend separately; do not treat waiting for user input itself as an infrastructure failure.

Cycle 2's preliminary development score is 1/5, with Git ungraded pending replacement. Its five initial attempts used 179 provider calls, 2,186,996 input tokens, 294,779 output tokens, $0.10604281 and 856.8 accumulated native seconds. None loaded or searched a skill. The modernization artifact passed even though the runtime requested clarification. No confirmation/adoption is justified by these preliminary results. Cycle 3 is frozen at `f1efd24` and running the same five development tasks.

## Cycle 4 hypothesis: evidence before clarification, basic writes visible

Development traces show final questions whose answers are already present in examples or diagnostics, and guessed `file_write` calls without content. A further candidate moves evidence checking into the opening instruction, preserves every genuinely unresolved doubt through `ask_user`, and exposes basic file creation/edit schemas immediately. Advanced tools remain discoverable; authorization, exact-content edits and workspace scope stay unchanged. The environment skill distinguishes a failed diagnostic from a faulty program and permits only authorized local recovery; unmodified shipped copies migrate while owner edits and disabled state survive.

Prediction: fewer premature question-only endings and fewer schema-discovery detours before producing artifacts. This combines prompt, schema visibility and procedural guidance, so gains cannot be attributed to one component. [Ask or Assume?](https://arxiv.org/html/2603.26233v1) motivates treating clarification as an information need; its simulated user and different model are not evidence that this candidate will work. No oracle, task-specific patch or hidden answer is provided. Tests/evaluation are pending; the cycle 3 binary remains frozen.

Cycle 4 agent validation passed all 114 tests (one environment-dependent test ignored), including all six planning-review tests and 23 runtime tests. An earlier broader cycle 3 run hit two unchanged planning-review timeouts under concurrent Docker/compile load; neither timeout was expanded and no planning code was changed. Their subsequent pass is recorded rather than hiding the initial failures. Workspace Clippy also passed without warnings. Final build and benchmark evaluation remain pending.
