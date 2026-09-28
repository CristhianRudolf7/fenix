# Harness comparison: Terminal-Bench tasks 7–9

[Español](three-harness-tasks-7-9-luna-low-2026-09-26.es.md) · [Measured data](three-harness-tasks-7-9-luna-low-2026-09-26.json)

GPT-6 Luna low, standard service, no Flex. One attempt per combination. Results: fenix 2/3, codex 2/3, opencode 2/3.

| Task | Harness | Checks | Time (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| 7. cobol-modernization | fenix | 3/3 | 69.71 | 0.00814693 | 184,469 | 3,299 |
| 7. cobol-modernization | codex | 3/3 | 31.50 | 0.00481439 | 69,319 | 2,468 |
| 7. cobol-modernization | opencode | 3/3 | 58.11 | 0.00632944 | 126,765 | 4,369 |
| 8. kv-store-grpc | fenix | 0/7 | 25.09 | 0.00384882 | 53,001 | 243 |
| 8. kv-store-grpc | codex | 5/7 | 46.52 | 0.00482404 | 121,769 | 1,661 |
| 8. kv-store-grpc | opencode | 7/7 | 69.57 | 0.00444282 | 113,637 | 2,059 |
| 9. polyglot-c-py | fenix | 1/1 | 77.68 | ≥0.00938776 | 221,072 | 4,375 |
| 9. polyglot-c-py | codex | 1/1 | 277.49 | ≥0.00526833 | 77,744 | 3,438 |
| 9. polyglot-c-py | opencode | 0/1 | 54.75 | 0.00370724 | 43,337 | 3,793 |

**≥ denotes recorded lower bounds:** one Fenix and one Codex request in task 9 lack usage; their token totals are partial too. Codex encountered a 240.158 s upstream read timeout and retried; this is included in its 277.49 s. Do not attribute that entire delay to the harness. No HTTP 429 occurred.

Time measures agent execution, excluding container preparation/download, harness installation and the external verifier. Provider pacing is included. Input tokens accumulate across calls, not in one context window. A stopped task cost is not a cost per solved task.

[Official pricing](https://developers.openai.com/api/docs/models/gpt-6-luna): per million tokens, input $0.10; cache reads $0.01; cache writes $0.125; output $0.50. Ordinary input subtracts both cache categories. Usage estimate, not an invoice; infrastructure and taxes excluded.

## 7. Reimplement COBOL in Python

Reproduce fixed-width account, book and transaction files exactly for the same input and initial state. All three passed 3/3 official checks.

- **Fenix:** read the source and data, implemented Python, compiled COBOL in a temporary copy and compared the files; corrected input-record handling and checked Python syntax. Both observed executions reported a rejected transaction. It did not exercise successful balance/ownership/log changes; it ran Python after COBOL without restoring the copied fixture. The passing comparison therefore does not establish general behavioral equivalence. 12 API calls.
- **Codex:** implemented fixed-width records and validation/update logic, then explicitly reported that it had not executed against the data. 6 calls. The external verifier passed; its speed partly reflects less self-verification.
- **OpenCode:** created a differential check with a valid transaction, noticed mismatches, corrected its fixture setup and changed the input slice from 18 to 22 bytes. Reran and obtained byte-for-byte parity, restoring original data afterward. 15 calls.
- **Fenix improvement:** when testing a transformation, restore identical independent fixtures and exercise one successful state-changing case plus a relevant rejection. Check that the expected mutation occurred, not merely that two unchanged outputs match. This is targeted requirement/risk coverage, not a broad mandatory test suite.

## 8. Build and keep a Python gRPC KV server running

Install pinned gRPC packages, define two RPCs, generate modules, implement an in-memory dictionary and leave port 5328 serving.

- **Fenix:** loaded a package skill and inspected the environment, then asked what GetVal should do for an absent key. It classified this unspecified edge behavior as an indispensable owner requirement. Noninteractive evaluation returned owner_input_unavailable, and the task stopped before any implementation. 0/7 checks. This is an observed overblocking outcome under the new policy, not a quota, permission or model-access failure; no paired run proves the old policy would have passed.
- **Codex:** implemented the files and service. Its first nohup launch did not survive; a foreground command session then answered SetVal/GetVal during the agent turn. After the agent exited the verifier found port 5328 closed. 5/7 checks. The observed failure concerns service lifecycle, not the demonstrated RPC logic.
- **OpenCode:** generated modules, checked direct set/get with negative values and launched a detached process using start_new_session. A real RPC probe and the external verifier passed: 7/7.
- **Fenix improvement:** distinguish indispensable missing intent from unspecified edge behavior. Continue independent, explicit requirements while asking an owner in interactive use. Without an owner, use a documented reversible convention for nonessential details; do not manufacture missing requirements and abandon the entire task. Preserve required approval and acceptance checks. Regress service survival across agent completion, because this Fenix attempt never reached that stage.

## 9. One Fibonacci source file executable as C and Python

Create a single source that both Python and GCC accept and that prints Fibonacci values.

- **Fenix:** implemented arbitrary-precision addition for C and Python integers, then repaired delimiter/escape errors. Python was missing from the agent environment; an earlier GCC attempt failed. After the final patch it called task_finish completed while explicitly acknowledging that neither final execution had been verified. The official verifier installed its dependencies and passed 1/1. This success does not validate the premature completion decision.
- **Codex:** Implemented arbitrary precision, recovered automatically after a 240-second upstream timeout, detected the initial GCC delimiter error, fixed it and reran C for N=10 and N=100 successfully. Reported that Python was unavailable rather than claiming both modes were tested. The official verifier passed 1/1. No dependency repair was attempted for its Python check.
- **OpenCode:** created an invalid draft, then reported failure without running the advertised compile/execute checks or attempting repair. The official GCC check failed, 0/1. No quota or deadline forced its early final answer.
- **Fenix improvements:** rerun the exact required commands after the last edit and require fresh evidence for those known acceptance paths. Do not mark completed when the outcome explicitly says verification is pending. Repair required missing tools within existing authorization. Source inspection also found that processes.rs detects missing_dependency but command_run in tools.rs overwrites the result with program_exit; the recorded python3 failure shows that generic classification. Consolidate this classification so the actionable diagnosis reaches the model. No runtime changes were made in this audit.

## Protocol and limitations

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `2591947b0b0958ffee78150a1ca89e3272e1f2cf59939e03807b8522d6e4506c`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Fresh official container per attempt, 1 CPU, 2 GiB, 900-second agent deadline. Rotating order: Fenix/Codex/OpenCode; Codex/OpenCode/Fenix; OpenCode/Fenix/Codex. Frozen binary, adapters and configuration. No behavioral changes, extra task attempts or hidden-test exposure during measurement. The same gateway respects provider quotas. JSON retains outcomes, timings, usage, pacing and private trace locations.

No deployment or runtime modification in this round. Three tasks cannot establish broad superiority; this is not a paired rerun measuring improvement on tasks 4–6.
