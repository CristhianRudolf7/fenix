# Harness comparison: Terminal-Bench tasks 19–21

[Español](three-harness-tasks-19-21-luna-low-2026-09-26.es.md) · [Measured data](three-harness-tasks-19-21-luna-low-2026-09-26.json)

GPT-6 Luna low, standard service, no Flex. One attempt per combination. Results: fenix 1/3, codex 3/3, opencode 3/3.

Evaluated changes: representative-input inspection, behavioral checks against hard constraints, updated test/data skills and explicit tool-program schemas. Auxiliary learning remains disabled during evaluation. All three adapters retain identical execution-target context. Local validation: 256 agent tests and four adapter tests passed, three optional agent tests skipped, successful server build.

All three native evaluations produced no skill-extraction requests; every recorded API call returned usage. These changes guide semantic verification; they do not make the runtime a general correctness oracle.

| Task | Harness | Checks | Time (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| 19. adaptive-rejection-sampler | fenix | 7/9 | 900.39 | 0.05079362 | 2,212,138 | 15,932 |
| 19. adaptive-rejection-sampler | codex | 9/9 | 418.22 | 0.02395636 | 1,138,241 | 11,202 |
| 19. adaptive-rejection-sampler | opencode | 9/9 | 389.34 | 0.01719442 | 462,045 | 5,379 |
| 20. fix-code-vulnerability | fenix | 2/6 | 47.72 | 0.00825030 | 186,157 | 2,081 |
| 20. fix-code-vulnerability | codex | 6/6 | 21.34 | 0.00603471 | 115,478 | 1,246 |
| 20. fix-code-vulnerability | opencode | 6/6 | 41.25 | 0.00646198 | 111,847 | 1,161 |
| 21. count-dataset-tokens | fenix | 1/1 | 202.92 | 0.01325242 | 269,482 | 2,658 |
| 21. count-dataset-tokens | codex | 1/1 | 122.65 | 0.00601271 | 147,694 | 1,862 |
| 21. count-dataset-tokens | opencode | 1/1 | 139.82 | 0.00708930 | 184,676 | 1,941 |

≥ indicates recorded lower bounds when a request lacks usage. Token totals in those cases can also be partial. Quota pacing and transport errors remain in JSON and are not automatically attributed to harness work.

Time measures agent execution, excluding container preparation/download, harness installation and the external verifier. Provider pacing is included. Input tokens accumulate across calls, not in one context window. A stopped task cost is not a cost per solved task.

[Official pricing](https://developers.openai.com/api/docs/models/gpt-6-luna): per million tokens, input $0.10; cache reads $0.01; cache writes $0.125; output $0.50. Ordinary input subtracts both cache categories. Usage estimate, not an invoice; infrastructure and taxes excluded.

## 19. Adaptive rejection sampling in R

Implement a modular sampler with input/log-concavity validation, formal tests against known distributions and a saved sample file. All three had to install R. Codex and OpenCode passed all nine official checks. Fenix reached the 900-second agent deadline with seven of nine checks passed.

- **Fenix:** installed R, implemented the sampler and detected real defects in its own tests: missing arguments, invalid tangent intersections, grossly incorrect sample variance and affine-density edge cases. Its attempted early failure close was rejected, so it continued repairing. However, repairs alternated between formulas without stabilizing the numerical invariants. A long statistical run repeatedly stalled; the final official basic sampling check passed, but the formal test timed out and the requested sample file was absent. 58 provider calls and 243.973 seconds of quota pacing were recorded; the quota delay contributed but was not the only failure cause.
- **Codex:** wrote the implementation while R installed, though it repeatedly polled processes with short waits. After intermittent exponential failures it exercised multiple trials, repaired grouping of nearly parallel tangents and recognized equal secant slopes. Its final 3000-sample suite passed and it saved normal_samples.txt. Official pass does not establish correctness for every possible density.
- **OpenCode:** recovered the R installation after an interrupted package operation, isolated an exponential failure with ten samples and collapsed nearly identical slopes into one equivalent tangent. Its full suite and sample-file generation then succeeded; the official verifier passed.
- **Remaining opportunity:** before running a large stochastic suite, verify small numerical invariants and the failing component on a tiny reproducible case, then scale up. Preserve known-good behavior rather than cycling formula patches. Keep required output files in the delivery checklist and diagnose long-running tests with observed progress. No mandatory additional model reviewer is needed. These new tasks do not establish a paired before/after gain from the prompt changes.

## 20. Repair Bottle's header validation vulnerability

Identify and repair a vulnerability, write report.jsonl with its CWE, and run the existing pytest suite. Codex and OpenCode passed all six official checks and the 367-test repository suite. Fenix passed two official checks, made no repair and produced no report.

- **Fenix:** searched broadly for generic invalid-input handling and fixated on parse_date returning None. It read a legacy test that deliberately expected that behavior, treated this unrelated candidate as the requested vulnerability, and asked the unavailable owner whether to change it. The runtime accepted owner_input_unavailable and closed blocked. It had inspected git status/log but not the diff, and never ran the available baseline suite. The request did not identify parse_date as the target: the supposed requirement conflict arose from an unconfirmed diagnostic hypothesis. Valid bare tool_program names avoided the prior prefix error, but independent search batches returned only one selected result, leaving other evidence unused. One memory-extraction call was recorded; disabling skill learning does not disable all memory processing.
- **Codex:** inspected the existing diff, found removed CR/LF/NUL checks in _hkey and _hval, restored ValueError validation, reported CWE-93 and ran pytest successfully.
- **OpenCode:** used diff and focused source/test inspection, applied the same repair and report, then passed pytest. Its initial generated title guessed CWE-20, but the final evidence-backed report correctly used CWE-93.
- **Remaining opportunity:** distinguish a diagnostic hypothesis from a user requirement. Before asking to change behavior, inspect available changed code and run the relevant existing baseline; establish that the suspected path actually explains the task failure. Preserve legitimate requirement questions and permissions. The blocking gate currently checks that a question/evidence exists, not whether the selected hypothesis is semantically justified. Prefer direct parallel reads when all independent results are needed.

## 21. Count tokens from a Hugging Face dataset

Read the dataset's README, select the scientific DeepSeek-generated fields, tokenize with Qwen2.5-1.5B-Instruct and write the integer answer. This task extends the original twenty-task selection using a predefined alphabetical/resource rule; it is not an official benchmark ordinal. All three passed, writing 79586.

- **Fenix:** loaded the Hugging Face skill and its data reference, read the README, installed dependencies, inspected schema and actual domain values, then counted reasoning and solution fields for biology, chemistry and physics. It recovered from an empty literal-science filter, independently checked field subtotals and reread answer.txt. The 26 science rows yielded 63841 reasoning tokens and 15745 solution tokens. Some overhead remained: search despite a known repository URL, repeated dataset/tokenizer loading and repeated counting.
- **Codex:** fetched the README directly, downloaded the metadata Parquet, inspected domain counts, tokenized the two requested fields and wrote the result. It used 11 API calls, versus Fenix's 15; the trace includes repeated dependency checks/installation attempts before processing.
- **OpenCode:** read the README and dataset metadata, recovered from an empty exact-science selection by inspecting domain frequencies, counted the science subjects and validated the saved integer. It used 14 API calls.
- **Remaining opportunity:** after reading documented schema, inspect actual categorical values before applying a filter. Combine the final calculation, independent subtotal checks, output write and readback in one coherent script, reusing loaded objects. Preserve the evidence that resolves semantic ambiguity rather than asking the owner about categories already established by the data. Passing this task supports the observed flow, not a causal claim that this round's guidance alone improved accuracy.

## Protocol and limitations

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `e85925f73a7d6f7f1acf21e11bedaa7c663a7809cdc8dedf36348e0db280d113`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Fresh official container per attempt, 1 CPU, 2 GiB, 900-second agent deadline. Rotating order: Fenix/Codex/OpenCode; Codex/OpenCode/Fenix; OpenCode/Fenix/Codex. Frozen binary, adapters and configuration. No behavioral changes, extra task attempts or hidden-test exposure during measurement. All three harnesses retain the same task-independent execution context as the previous round. Native evaluation continues to disable background skill learning. The same gateway respects provider quotas. JSON retains outcomes, timings, usage, pacing and private trace locations.

Tasks 19 and 20 complete the original selection. Task 21, count-dataset-tokens, extends it: the first remaining alphabetically sorted text-only CPU task with official deadline <=900s, selected before results. No deployment or runtime modification during measurement. Three tasks cannot establish broad superiority; this is not a paired rerun measuring improvement on tasks 16–18.
