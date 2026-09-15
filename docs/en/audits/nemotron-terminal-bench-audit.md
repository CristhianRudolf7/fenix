# Fenix harness audit with Nemotron

[Español](../../es/audits/nemotron-terminal-bench-audit.md) · [Execution protocol](nemotron-terminal-bench-protocol.md)

Completed: 2026-09-15. Snapshot: Fenix 0.3.0 plus the local adaptive-memory changes recorded in the protocol.

## Scope and measurement

Twenty prespecified official Terminal-Bench 2.1 tasks are attempted with the frozen Fenix Rust runtime and stock Harbor Terminus 2, using the same RunInfra `nemotron-3-5-lightning-30b` model with default thinking enabled. The reference helps identify tasks the model can solve through a different harness; it does not isolate every architectural difference causally.

The [protocol](nemotron-terminal-bench-protocol.md) records the source and binary hashes, dataset commit, resources, timeouts, context settings, adapter corrections and task selection. These are 20 diagnostic tasks, not the full benchmark, a leaderboard submission, or a general estimate of production success. There is one baseline attempt per harness per task. No hidden verifier instructions are supplied to either agent.

Both paths use the same model and nominal output ceiling, but their context accounting, prompts, tool interfaces, retry behavior and interpretation of input limits differ. Fenix retains its real defaults. Environment setup and provisioning are separate from agent execution time. The two concurrent containers run on a shared development host; absolute latency is not a dedicated model-speed benchmark.

| Metric | Fenix | Terminus 2 |
| --- | ---: | ---: |
| Tasks solved | 1/20 (5%) | 10/20 (50%) |
| Recorded API calls | 334 | 1,130 |
| Input tokens, including cache | 3,172,921 | 15,806,466 |
| Cached input subset | 1,862,656 | 11,550,208 |
| Output tokens, including reasoning | 160,842 | 1,215,409 |
| Accumulated agent execution (s) | 917.62 | 7,742.80 |
| Median attempt duration (s) | 41.02 | 315.66 |
| Provider-reported cost (USD) | $0.108266 | $0.510626 |

Lower Fenix time/cost largely reflects early termination, not better useful throughput. Only the merger task passed in both harnesses: 44.04 s versus 98.24 s. Input totals already include cached tokens; output includes reasoning. One reference connection ended with a broken pipe at the Python/C task deadline; its complete usage was recorded. No provider HTTP failure explains the Fenix failures.

| Fenix termination category | Tasks |
| --- | ---: |
| Invalid/truncated compaction summary | 9 |
| No permissible cut for the recent batch | 2 |
| Foreground output exhausted | 3 |
| Waiting for user clarification | 3 |
| Declared completion, verifier fails | 2 |
| Official success | 1 |

The 14 context/output stops are observed termination mechanisms, not proof that fixing them would yield 14 more successes. One R-task verifier could not run semantic checks after agent-induced package-manager damage; its official reward remains zero and it is flagged below. The excluded initial adapter failure made no model calls and is not one of the 20 attempts.

Fenix issued 132 command launches and 132 status polls; 80 polls returned completed status in under 50 ms. This does not mean all 80 commands would have finished during a short initial wait, and no latency-saving intervention was tested.

## What is failing in Fenix

### 1. Compaction can terminate otherwise recoverable work

[active_context.rs](../../../crates/fenix-agent/src/active_context.rs) fixes summary output at 2,048 tokens. Several real Nemotron responses use that budget without producing visible summary text and end with `finish_reason=length`. The runtime correctly refuses to replace the original context with an invalid summary, but [engine.rs](../../../crates/fenix-agent/src/engine.rs) propagates that error out of the task. Keeping a durable checkpoint does not provide automatic continuity.

This occurred during recovery, builds, migration and service setup. Some earlier summaries succeeded; the defect is the fatal handling of a later failure, not that every 2,048-token request necessarily fails. In the WAL task, both agents had already queried the original database before preserving the WAL, which disappeared. That earlier damaging action must not be attributed to the later summary failure.

**First change to test:** a reasoning-aware summary budget with a bounded retry, preserved task/operation identity, and an explicit fallback when summarization still fails. Validate the resulting summary and the space it actually frees. Increasing a limit alone is not proof of faithful memory or successful task completion.

### 2. A valid tool batch can leave no permissible context cut

The compacting algorithm keeps whole tool-call/result batches, requires the recent tail to fit 6,000 estimated tokens, and excludes cutting at the end of all messages. Three log reads returned 40,096 content characters together. No eligible cut could retain that whole recent batch, so the task aborted before creating a compaction checkpoint. A calendar task reached the same condition after a long reasoning response and repeated calendar reads.

**Needed:** an aggregate output budget for each batch, recoverable artifact references for oversized results, and a fallback that can summarize a completed oversized batch while preserving its tool-call/result integrity. Limit accounting should reflect the actual provider payload and negotiated model capacity. Fenix's configured 32,768-token context is much smaller than the provider's advertised capacity; conservative serialized-byte accounting and reasoning replay add pressure.

### 3. Foreground output exhaustion has no recovery path

Fenix aborts on a length-limited response before executing its proposed actions. That avoids acting on incomplete tool arguments, but currently also prevents a bounded request for smaller, complete actions. Both polyglot tasks and the Vim macro task exposed this path. Terminus also encountered truncated responses; it eventually passed Rust/C++ but timed out on Python/C.

**Needed:** distinguish output exhaustion from other finish reasons, request a smaller action or adjust a supported output budget, and cap repeated attempts without progress. Do not execute incomplete calls or replay effects that already happened. Unlimited retries are not supported by these results.

### 4. Process lifetime and cancellation need clearer operational contracts

In gRPC, Fenix launched `nohup ... &` with `persistent=true`. The shell returned zero, then the process manager's normal group cleanup killed its background child. The managed runtime remained alive for verification, so this was not an adapter shutdown. Missing inspection utilities and a fallback `echo` did not prove the port was listening, yet the final answer claimed that it was.

The current working contract is a foreground service command managed with `persistent=true`. Make that explicit or provide a service operation with a defined lifetime, early-exit status and readiness check. Do not remove descendant cleanup globally.

In the R task, an attempted `R --version` was rejected while installation was running. The model cancelled the installer, restarted installation, and a later compaction abort interrupted package management again. The verifier could not prepare its dependencies because `dpkg` was left interrupted. This is agent-induced environment damage, not evidence about the numerical quality of an implemented sampler. Surface active process IDs and progress, favor waiting over cancellation for a diagnostic probe, and record recovery obligations for interrupted stateful operations.

The cap-only pMARS follow-up exposed another interaction: after two successful compactions, three identical busy rejections triggered the repetition guard even though installer output was advancing between them. The guard counts matching action/result pairs across intervening progress. Represent temporary busy states explicitly, with the active process ID and a waiting/queueing policy; distinguish them from repeated actions with no overall progress.

### 5. Unnecessary questions prevent unattended completion

Fenix asked for information already available in Git history, reconfirmed an explicitly requested server setup, and asked whether publication should be automatic when the requested push-then-fetch workflow already specified that behavior. No user answers were invented to help it finish.

The system prompt already tells the model not to repeat resolved questions. This is an instruction-compliance and harness-interaction issue, not simply a missing sentence. Preserve the user's preference to surface real doubts, while distinguishing missing facts from authorization already granted. Inspect the evidence and the task contract before asking. Unchanged repetitions below help assess the variability of the Git result.

### 6. More calls and more self-tests do not automatically improve quality

Quick commands normally require a separate `command_status` model round trip. A short local wait could return already available output in the initial tool result and retain asynchronous handling for longer processes. The counts below measure the opportunity, not a proven speedup from an implemented change.

Both harnesses missed awaited cleanup in the asynchronous cancellation task. Fenix's generated self-tests exercised synchronous cleanup and did not cover the failing awaited-cleanup interaction. The reference also produced incorrect SQLite values and an Nginx log missing a closing quote. Prioritize tests derived from the actual behavioral requirements, including relevant edge cases, instead of requiring another LLM reviewer for every action.

### 7. Auxiliary work needs compatible budgets and relevance controls

A background memory extraction used a 1,800-token cap and returned no usable JSON after exhausting it. This is separate from the calendar task's fatal context-cut error. The memory worker records extraction failures, but these benchmark tasks do not establish long-term memory quality.

The R task's web search used a fresh, bounded local lexical index. One broadened query returned an unrelated page; another returned no results. The tool disclosed its limited coverage, but the irrelevant result still consumed context. Low-coverage retrieval should favor useful evidence or abstention. Retrieval quality was not independently benchmarked here.

## Controlled follow-ups

These are additional diagnostics, excluded from the 20-task baseline. No Rust engine source was changed.

**Exact-input summaries.** Two failed baseline inputs, three repetitions per input and ceiling, alternating ceilings. Only `max_tokens` changed in the replayed payload. Acceptance means normal stop, nonempty text and no tool calls; it does not score semantic faithfulness.

| Source task | Output ceiling | Accepted | Mean output tokens | Mean API seconds | Cost USD |
| --- | ---: | ---: | ---: | ---: | ---: |
| db-wal-recovery | 2,048 | 0/3 | 2048.0 | 5.49 | $0.001563 |
| db-wal-recovery | 16,384 | 3/3 | 5242.7 | 13.28 | $0.003001 |
| build-cython-ext | 2,048 | 1/3 | 1510.3 | 4.29 | $0.001441 |
| build-cython-ext | 16,384 | 3/3 | 1078.7 | 3.25 | $0.001160 |

Across these two fixed inputs, acceptance was 1/6 at 2,048 versus 6/6 at 16,384. The sample is small and generation remains stochastic. This supports a budget-compatibility issue in these cases, not a universal success rate or a faithful-memory guarantee.

**Full pMARS follow-up.** The proxy changed only the internal summary ceiling to 16,384. Two summaries were accepted, consuming 4,850 and 502 output tokens. The task still failed: three identical busy errors activated the repetition guard while installation was progressing. No baseline success is replaced by this result.

- pMARS: reward 0; 82.80 s; 55 API calls; $0.015483.

**Unchanged Git repeats.** Both additional attempts failed. One asked for clarification again; the other stopped at compaction. Including the original, this is 0/3 on this one task. The failure path varies; three trials do not establish a universal failure probability.

| Trial | Outcome | Seconds | API calls | Cost USD |
| --- | --- | ---: | ---: | ---: |
| fix-git__EFVcckW | Waiting for clarification | 35.27 | 16 | $0.003514 |
| fix-git__EsgXbME | Compaction failure | 47.39 | 26 | $0.007063 |

**Total recorded spending: $0.65216044.** Baseline $0.61889244; three full follow-ups $0.02605965; 12 summary probes $0.00716545; direct preflight $0.00004290. All 1,573 proxied requests have provider-reported billing, plus one direct preflight. This is reported API usage cost, not a provider invoice.

## Per-task results and failure analysis

[Full per-attempt metrics CSV](nemotron-terminal-bench-results.csv). Result cells show official reward, followed by passed/total checks. Partial checks are diagnostic, not partial task credit. Times are agent-phase seconds.

| Task | Fenix result | Reference result | Fenix s | Reference s | Fenix USD | Reference USD |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cancel-async-tasks | 0 (5/6) | 0 (5/6) | 52.2 | 26.1 | $0.00769 | $0.00145 |
| fix-git | 0 (0/2) | 1 (2/2) | 43.6 | 38.9 | $0.00474 | $0.00288 |
| db-wal-recovery | 0 (0/7) | 0 (0/7) | 39.3 | 643.8 | $0.00685 | $0.06423 |
| multi-source-data-merger | 1 (3/3) | 1 (3/3) | 44.0 | 98.2 | $0.00735 | $0.00683 |
| build-cython-ext | 0 (2/11) | 0 (9/11) | 122.3 | 541.0 | $0.01444 | $0.01138 |
| build-pmars | 0 (2/4) | 1 (4/4) | 99.8 | 310.2 | $0.01068 | $0.01331 |
| cobol-modernization | 0 (1/3) | 1 (3/3) | 32.0 | 806.3 | $0.00416 | $0.06478 |
| kv-store-grpc | 0 (5/7) | 1 (7/7) | 41.4 | 61.7 | $0.00426 | $0.00197 |
| polyglot-c-py | 0 (0/1) | 0 (0/1) | 42.5 | 900.0 | $0.00276 | $0.06133 |
| polyglot-rust-c | 0 (0/1) | 1 (1/1) | 54.8 | 370.1 | $0.00277 | $0.02089 |
| git-multibranch | 0 (0/1) | 0 (0/1) | 24.8 | 281.8 | $0.00056 | $0.01769 |
| configure-git-webserver | 0 (0/1) | 1 (1/1) | 26.6 | 688.1 | $0.00212 | $0.04654 |
| nginx-request-logging | 0 (1/8) | 0 (7/8) | 37.4 | 321.1 | $0.00604 | $0.02512 |
| openssl-selfsigned-cert | 0 (5/6) | 1 (6/6) | 30.1 | 50.2 | $0.00604 | $0.00432 |
| sqlite-db-truncate | 0 (0/1) | 0 (0/1) | 34.4 | 112.2 | $0.00611 | $0.01034 |
| large-scale-text-editing | 0 (0/5) | 0 (4/5) | 53.4 | 1200.0 | $0.00448 | $0.05770 |
| log-summary-date-ranges | 0 (0/2) | 1 (2/2) | 9.1 | 22.7 | $0.00077 | $0.00227 |
| constraints-scheduling | 0 (0/3) | 1 (3/3) | 40.7 | 133.3 | $0.00331 | $0.00942 |
| adaptive-rejection-sampler | 0† | 0 (4/9) | 67.7 | 714.8 | $0.00849 | $0.04939 |
| fix-code-vulnerability | 0 (2/6) | 0 (5/6) | 21.7 | 422.3 | $0.00463 | $0.03878 |

† The R-task verifier was blocked by interrupted package management caused during the agent attempt. No semantic check count is claimed. Python/C and Vim reference attempts reached their official 900/1,200-second deadlines; WAL and R reference attempts reached the configured 128-turn limit.

| Task | Diagnosis |
| --- | --- |
| cancel-async-tasks | Both miss awaited cleanup when queued tasks exist; Fenix self-tests used synchronous cleanup. |
| fix-git | Fenix asks despite receiving the detached commit in the reflog; reference recovers the changes. |
| db-wal-recovery | Both touch the original database before preserving the WAL. Fenix later aborts at compaction; reference exhausts 128 turns without recovery. |
| multi-source-data-merger | Both pass. This is the only jointly successful task; Fenix is faster in this one case. |
| build-cython-ext | Fenix aborts at summary generation. Reference builds extensions but fails two compatibility/test checks. |
| build-pmars | Fenix aborts at compaction; reference passes. The cap-only follow-up later exposes a busy/repetition failure. |
| cobol-modernization | Fenix aborts at compaction before producing the required migration; reference passes. |
| kv-store-grpc | Fenix creates service files but its detached server is killed when the shell exits; reference passes. |
| polyglot-c-py | Fenix exhausts its first response without actions. Reference retries but times out at 900 seconds. |
| polyglot-rust-c | Fenix exhausts its first response. Reference recovers from output exhaustion and passes. |
| git-multibranch | Fenix requests blanket reconfirmation. Reference mutates repository state through inherited Git variables in its deployment hook. |
| configure-git-webserver | Fenix asks questions already answered by the requested push-to-web workflow; reference passes. |
| nginx-request-logging | Fenix aborts during compaction. Reference serves pages but omits the closing user-agent quote in logs. |
| openssl-selfsigned-cert | Fenix creates valid certificate artifacts but aborts while diagnosing an N/A parser output. Reference passes. |
| sqlite-db-truncate | Fenix aborts before creating recovery output. Reference writes JSON but only 2 of 10 recovered pairs match. |
| large-scale-text-editing | Fenix exhausts output at step 8. Reference reaches 1,200 seconds with a runnable macro that produces the wrong CSV. |
| log-summary-date-ranges | Fenix cannot compact a completed batch containing three large reads; no output CSV. Reference passes. |
| constraints-scheduling | Fenix cannot compact its recent reasoning/read batch. Reference writes the valid meeting file. |
| adaptive-rejection-sampler | Fenix interrupts package management and its verifier cannot prepare dependencies. Reference reaches 128 turns with syntactically incomplete R code. |
| fix-code-vulnerability | Fenix aborts before patch/report. Reference passes functional header checks but reports CWE-20 instead of the required CWE-93. |

## Interpretation and next validation

Prioritize continuity across compaction, handling of oversized batches and truncated output, then process/service lifecycle and evidence-aware clarification. Preserve the existing passing behavior and verify each fix with a focused regression case. Afterwards evaluate on additional unseen tasks; follow-ups on these same tasks are diagnostic and must not be substituted into the baseline score.

This audit does not prove that every failure belongs to the harness or that Nemotron can solve every selected task. It identifies concrete engine failure paths, model/harness interactions and shared solution errors. A reference success is a useful counterexample to model incapability; an individual reference failure is not an oracle about what the model can never do.

No skill loading, subagent delegation, plan-management or LSP tool calls were observed in the 20 Fenix baseline attempts. Cross-project memory, long-term conversational recall, Telegram, scheduling and those unexercised programming features require separate coverage. Passing individual structural checks may reflect behavior already present before the agent did any work.

Raw requests, responses, events, configurations and verifier output remain in the ignored local audit directory. Public tables omit credentials and raw transcripts. Reusable integration code is in [scripts/benchmarks/harbor](../../../scripts/benchmarks/harbor/README.md).

Sources: [official Terminal-Bench 2.1 dataset](https://github.com/harbor-framework/terminal-bench-2-1/tree/7131e4375048a0e408a8fb404b5f499d726b695b), [Harbor agent integration](https://www.harborframework.com/docs/agents), [RunInfra model](https://runinfra.ai/inference-api/nemotron-3-5-lightning-30b). Measured findings above come from this audit's local artifacts and source inspection.
