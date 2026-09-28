# Harness comparison: Terminal-Bench tasks 16–18

[Español](three-harness-tasks-16-18-luna-low-2026-09-26.es.md) · [Measured data](three-harness-tasks-16-18-luna-low-2026-09-26.json)

GPT-6 Luna low, standard service, no Flex. One attempt per combination. Results: fenix 1/3, codex 3/3, opencode 2/3.

Evaluated changes: auxiliary skill learning disabled before evaluation startup; exact repeated request text projected as references; optional skill loading for familiar operations and batched independent diagnostics. All three adapters retain identical execution-target context. Local validation: 255 agent tests and four adapter tests passed, three optional agent tests skipped, successful server build.

All three native evaluations produced no skill-extraction requests; every recorded API call returned usage. Header/routing duplicates were removed, but the persistent task contract retains another copy of the objective. The failures show that reducing calls must preserve relevant inspection and validation.

| Task | Harness | Checks | Time (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| 16. large-scale-text-editing | fenix | 5/5 | 206.02 | 0.00919554 | 125,863 | 1,753 |
| 16. large-scale-text-editing | codex | 5/5 | 208.75 | 0.00544949 | 126,526 | 2,610 |
| 16. large-scale-text-editing | opencode | 5/5 | 196.68 | 0.00322518 | 52,883 | 1,930 |
| 17. log-summary-date-ranges | fenix | 1/2 | 32.98 | 0.00514585 | 75,338 | 1,101 |
| 17. log-summary-date-ranges | codex | 2/2 | 14.14 | 0.00369942 | 46,952 | 689 |
| 17. log-summary-date-ranges | opencode | 1/2 | 22.78 | 0.00208226 | 20,106 | 408 |
| 18. constraints-scheduling | fenix | 2/3 | 34.98 | 0.00489940 | 96,511 | 658 |
| 18. constraints-scheduling | codex | 3/3 | 15.07 | 0.00352835 | 33,258 | 1,050 |
| 18. constraints-scheduling | opencode | 3/3 | 39.57 | 0.00382673 | 63,082 | 1,936 |

≥ indicates recorded lower bounds when a request lacks usage. Token totals in those cases can also be partial. Quota pacing and transport errors remain in JSON and are not automatically attributed to harness work.

Time measures agent execution, excluding container preparation/download, harness installation and the external verifier. Provider pacing is included. Input tokens accumulate across calls, not in one context window. A stopped task cost is not a cost per solved task.

[Official pricing](https://developers.openai.com/api/docs/models/gpt-6-luna): per million tokens, input $0.10; cache reads $0.01; cache writes $0.125; output $0.50. Ordinary input subtracts both cache categories. Usage estimate, not an invoice; infrastructure and taxes excluded.

## 16. One-million-row transformation with constrained Vim macros

Create three distinct nonempty registers and use only the allowed Vim commands to transform input.csv into expected.csv. All three passed the five official checks.

- **Fenix:** inspected the environment and CSV samples, corrected an invalid tool_program call that used a namespaced command name, wrote the three macros and ran the full transformation. Waited on the actual process and observed Vim exit 0 plus byte-for-byte comparison success. Discovered command_cancel but did not cancel the progressing transformation. No skill-loading ceremony or post-task learning call was needed.
- **Codex:** inspected samples, wrote macros and eventually verified matching output. Its transcript includes an invalid wait with a placeholder cell identifier, a process inspection and another Vim invocation; later it used the returned session identifier correctly. These are concrete process-handling overheads, not evidence of a model capability difference.
- **OpenCode:** wrote a macro with an incorrectly escaped wildcard, inspected the mismatch, corrected the expression and reran Vim successfully.
- **Remaining opportunity:** validate macro syntax/transformation on a small independent fixture before processing a million rows, then run the required full comparison. Batch only independent steps and call direct tools with their exact schemas; a one-step tool_program adds unnecessary failure opportunities. Native execution time and external verifier time are reported separately. These outcomes do not isolate the causal effect of prompt shortening.

## 17. Log severity counts across five date ranges

Produce a 15-row CSV counting ERROR, WARNING and INFO for today, the inclusive last seven/thirty days, month-to-date and all dates, using the supplied reference date. Codex passed both official checks; Fenix and OpenCode passed file existence but failed the count check.

- **Fenix:** listed files, inspected the environment and counted severity words anywhere in each file without first inspecting representative records. Recovered from a Python unpacking error for the unbounded total period, then checked the CSV header, row count, order and nonnegative counts. Those structural assertions never verified that a match came from the severity field. Today/ERROR was 414 instead of 370.
- **Codex:** inspected sample records before implementing a parser for the bracketed severity field, handled the date ranges and wrote the correct CSV in four API calls.
- **OpenCode:** counted all word occurrences using the same broad regex approach as Fenix and failed with the same incorrect count, also in four API calls.
- **Remaining opportunity:** combine file discovery and a small representative sample in the initial inspection; validate one adversarial record whose message mentions another severity. This is a cheap requirement-linked check, not a large generic test suite. Equal call counts produced different correctness: efficiency must preserve necessary evidence. This single unpaired run cannot establish that the prompt changes caused the failure.

## 18. Scheduling with calendar conflicts and availability constraints

Read three immutable calendars and choose the earliest one-hour meeting satisfying hard availability, conflicts and preference tie-breakers. Codex and OpenCode passed all three official checks. Fenix passed two checks but failed hard-constraint validation.

- **Fenix:** read all calendars, recovered from another invalid namespaced tool_program call, and wrote Monday January 15 at 09:00 UTC. This violates Bob's explicit prohibition on meetings before 10 AM. That instruction was present in the actual request: this was not lost context or compaction. The final evidence discussed calendar conflicts but did not validate every hard availability constraint; file creation was incorrectly treated as sufficient evidence of completion.
- **Codex:** grouped directory and calendar inspection into one command, wrote Wednesday January 17 at 11:00 UTC and displayed the file. Three API calls; official grading confirmed correctness, but the trace does not show an independent exhaustive constraint checker.
- **OpenCode:** read calendars, maintained a short todo list, wrote the same correct Wednesday slot and ran local checks. Its input-integrity assertion contained an always-true fallback, and its conflict check mixed HHMM-style bounds with minutes since midnight. These checks are not reliable evidence despite the correct output and passing official grader.
- **Remaining opportunity:** derive a compact set of hard constraints and validate the candidate against each, including existing intervals, before claiming completion. For this task a short deterministic calendar check is enough; another reviewer/model is unnecessary. Distinguish hard rules from preferences, use one time unit consistently, and do not accept assertions that are always true. Also clarify tool_program's exact callable names so batching does not produce retries.

## Protocol and limitations

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `8f42d79a3c295eba61395cc0f671421da3613bfa721ba59fe9461a5cdbd07528`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Fresh official container per attempt, 1 CPU, 2 GiB, 900-second agent deadline. Rotating order: Fenix/Codex/OpenCode; Codex/OpenCode/Fenix; OpenCode/Fenix/Codex. Frozen binary, adapters and configuration. No behavioral changes, extra task attempts or hidden-test exposure during measurement. All three harnesses retain the same task-independent execution context as the previous round. Native evaluation now explicitly disables background skill learning. The same gateway respects provider quotas. JSON retains outcomes, timings, usage, pacing and private trace locations.

No deployment or runtime modification in this round. Three tasks cannot establish broad superiority; this is not a paired rerun measuring improvement on tasks 13–15.
