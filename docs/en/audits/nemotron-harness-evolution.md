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

Reference: **0/5**. Provider totals: 78 calls, 626,763 input tokens, 188,884 output tokens, **$0.05044451**. Three empty-tail compaction failures; two no-progress recovery failures. Setup/download time is not included in native seconds. Candidate evaluation is pending.
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
