# Coding skills and prompt efficiency — GPT-6 Luna low

[Español](../../es/audits/luna-low-skill-efficiency.md)

On September 23, 2026, two instruction revisions and a paired confirmation preserved all 95 external checks. The confirmation reduced model requests by 23.1%, input tokens by 27.6%, estimated cost by 15.6%, and wall time by 7.0%. This is an efficiency improvement on four small tasks, not evidence of better general intelligence or superiority to Codex.

## What changed

- Reuse visible tool schemas and suggested skills; discover only missing capabilities.
- Gather independent skill loading, environment inspection and known file reads together. Use relative file paths, coherent multi-file edits, and direct overwrite rather than delete/recreate sequences.
- Use declarative tool programs only for supported dependent steps. Environment inspection and skill loading remain direct calls. Completed command results already establish exit status; do not poll unnecessarily.
- Diagnose a newly written test's setup, expected state and actual implementation separately. Stateful fixtures must initialize the same resource used by the operation. Preserve supplied acceptance tests and the user's requirements.
- Upgrade only exact untouched bundled skill bodies through existing hash migrations. Preserve custom instructions and enabled/disabled state. Versions: software-implementation 1.6.0, test-driven-repair 1.4.0, python-development 1.1.0.

Execution permissions, validators, model, reasoning effort and evaluation deadlines were not relaxed. No additional model reviewer was added.

## Method

The same four fixtures and external checkers from the preceding comparison were reused: TTL/LRU cache (17 checks), transactional ledger (28), dependency planner and CLI (23), atomic JSON persistence and CLI (27). All runs used GPT-6 Luna `low`, standard/default service, fresh workspaces and state, the native Fénix evaluator, a 300-second task deadline and a 330-second process deadline. None reached those deadlines. The recording proxy supplied authentication and recorded requests and provider usage without changing model payloads.

The first candidate addressed three skills and the coding prompt. Its improvement was modest and included an unsupported tool composition and an incorrect absolute path. Candidate 2 reconciled the master prompt with the coding workflow, including discovery, batching and completed-process handling. Its binary was frozen before confirmation. Confirmation alternated original/new execution order by task, sequentially, using both frozen binaries. No Codex rerun was needed to isolate the effect of the Fénix changes; its prior result remains historical only.

All fixture hashes matched. Checkers were compiled and independently rerun, and the original unfinished fixtures still failed. Actual API records confirmed model and reasoning effort with complete usage records. Tasks were not changed, and no benchmark answers or task-specific code were embedded in the prompts or skills.

## All measured rounds

| Run | Checks | Requests | Input tokens | Output tokens | Wall seconds | Estimated USD |
|---|---:|---:|---:|---:|---:|---:|
| Original reference | 95/95 | 52 | 620,766 | 9,304 | 208.0 | 0.02039 |
| Candidate 1 | 95/95 | 46 | 544,988 | 9,098 | 212.6 | 0.01824 |
| Candidate 2 | 95/95 | 41 | 473,809 | 8,557 | 205.2 | 0.01660 |
| Confirmation: original | 95/95 | 39 | 427,904 | 8,676 | 201.2 | 0.01608 |
| Confirmation: candidate 2 | 95/95 | 30 | 309,987 | 7,991 | 187.0 | 0.01357 |

Candidate 1 is retained in the report rather than omitted. Candidate 2's first round reduced input tokens by 23.7% against the original reference but wall time by only 1.3%. Confirmation independently reproduced an input-token improvement against a fresh control. These are observed samples, not confidence intervals.

## Paired confirmation by task

| Task | Original requests | New requests | Original seconds | New seconds | Checks each |
|---|---:|---:|---:|---:|---:|
| ttl-cache | 9 | 7 | 52.8 | 38.9 | 17 |
| ledger | 8 | 6 | 41.7 | 45.1 | 28 |
| dependency-plan | 13 | 6 | 58.7 | 41.0 | 23 |
| atomic-json | 9 | 11 | 48.0 | 62.1 | 27 |

Atomic JSON regressed in requests and time; ledger used fewer requests but took longer. The change does not speed up every task. Residual failures include an unnecessary read of a nonexistent file; earlier candidate runs still produced malformed test fixtures. Nonzero command results are not themselves proof of a harness defect: a deliberately failing regression test can be useful.

## Accounting and limits

Input counts include cached input and are cumulative across requests, not unique context size. Wall time includes cold startup, tool execution, model latency and shutdown. Estimated USD uses the previous audit's fixed per-million rates: input $0.10, cached input $0.01, cache writes $0.125, output $0.50. Cache-write premium is added only above the input rate already counted. Actual invoices and cache state can differ.

This is a tuned, four-task Python sample with no new held-out suite. It does not establish performance on SWE-bench, Terminal-Bench, production traffic, long tasks or other domains. Broadly shortening unrelated assistant instructions was deliberately avoided. Both versions passed all checks; the demonstrated improvement is efficiency, not a higher success rate.

[Machine-readable results and binary/fixture hashes](../../benchmarks/luna-low-skill-efficiency-2026-09-23.json). Full requests, traces, checkers and frozen binaries remain in the local evaluation archive (`luna-skill-efficiency-*20260923`). Prior accumulated work was pushed to main as `d385b1e` before experiments. These prompt/skill changes have not been deployed to production.

## Validation

The server binary built successfully. The existing `hash_upgrade_only_replaces_the_exact_pre_library_release` regression passed, and all three prior stock skill hashes were checked against the migration registry while customized bodies were excluded. Sixteen newly scored task attempts covered two candidates and an eight-attempt paired confirmation; these reuse four fixtures, not sixteen distinct tasks. Documentation links and result JSON were validated.
