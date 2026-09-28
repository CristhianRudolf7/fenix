# Harness comparison: Terminal-Bench selection tasks 4–6

[Español](three-harness-next-three-luna-low-2026-09-26.es.md) · [Measured data](three-harness-next-three-luna-low-2026-09-26.json)

After stabilizing Git HOME and making verification requirement/risk based, the next three tasks were run with GPT-6 Luna low, standard service, no Flex. OpenCode passed 3/3; Codex passed 2/3; Fenix passed 2/3 in runs with valid verifiers. **Fenix required one infrastructure replacement for pMARS, and its original attempt also exposed an unnecessary request for permission.** These results do not establish equivalent general reliability for Fenix and Codex.

## Changes and local validation

- Standalone shell HOME remains stable across cwd changes, preserving configured Git identity. The allowlisted environment and remote tenant isolation remain intact; no invented or historical author identities.
- System prompt and four skills reuse existing tests/receipts, test requested behavior and concrete risks, add only missing relevant checks, and stop when sufficient evidence passes. No mandatory reviewer or generated per-tool acceptance contract.
- Known built-in skill migrations preserve customized and disabled skills.
- 12 process tests, 5 execution tests and 1 skill migration test passed; server build and whitespace checks passed. The Git regression performs real commits through pipes and PTYs with a negative control for the old environment.

Not deployed. Git was covered by the local regression, not another `fix-git` benchmark attempt.

## Measurements

| Task | Harness | Checks | Time (s) | Estimated USD | Input | Output |
|---|---|---:|---:|---:|---:|---:|
| 4. multi-source-data-merger | fenix | 3/3 | 56.38 | 0.01082240 | 182,870 | 2,130 |
| 4. multi-source-data-merger | codex | 3/3 | 22.97 | 0.00396414 | 52,868 | 1,664 |
| 4. multi-source-data-merger | opencode | 3/3 | 47.55 | 0.00395757 | 71,278 | 2,205 |
| 5. build-cython-ext | fenix | 10/11 | 900.43 | ≥0.07228532 | 2,161,739 | 15,724 |
| 5. build-cython-ext | codex | 2/11 | 343.09 | 0.01044943 | 387,553 | 3,567 |
| 5. build-cython-ext | opencode | 11/11 | 521.87 | 0.02349096 | 1,018,581 | 7,576 |
| 6. build-pmars | fenix | 4/4 | 95.34 | 0.00840589 | 173,789 | 1,583 |
| 6. build-pmars | codex | 4/4 | 94.87 | 0.01350399 | 420,290 | 2,740 |
| 6. build-pmars | opencode | 4/4 | 250.53 | 0.01170876 | 368,692 | 3,078 |

Agent execution time includes startup and its own commands, excluding environment setup, harness installation and the external verifier. Input tokens accumulate across requests; they are not a single context window. Costs derive from reported usage including cache reads/writes, excluding infrastructure and taxes. Fenix Cython has one interrupted response without usage, so its reported cost and tokens are lower bounds.

[Official pricing](https://developers.openai.com/api/docs/pricing), per million: ordinary input $0.10, cache read $0.01, cache write $0.125, output $0.50. Ordinary input subtracts both cache categories.

## Per-task audit

### 4. Merge JSON, CSV and Parquet user records

Normalize fields/dates, deduplicate by source priority and produce Parquet plus conflict report. All harnesses passed 3/3.

- **Fenix:** verified IDs, priorities, dates and conflicts. Redundant tool discovery, a weakly matched structured-output skill and two `task_finish` calls lacking structured evidence led to 12 requests versus Codex's 5 and OpenCode's 11. The third finish call succeeded.
- **Codex:** inspected sources and ran a coherent merge script; fastest result.
- **OpenCode:** merged and reread the output to verify schema, records and conflicts.
- **Remaining improvement:** better skill routing and unambiguous completion evidence reuse. Relevant behavioral checks were not the main source of extra calls.

### 5. Build pyknotid against NumPy 2.3.0

Install three native Cython extensions globally, run the example and requested repository suite.

- **Fenix:** 10/11 official checks. Compiled, installed and repaired Python/NumPy/planarity compatibility; example and repository suite passed. `np.int` remained in `ccomplexity.pyx`, breaking a native operation despite successful import. Searches/edits largely targeted `.py`. Ambiguous patches caused retries and partial closure attempts; completion controls made it continue. It reached 900 s, including 368.366 s of provider-quota pacing.
- **Codex:** 2/11. Several code-mode wrappers returned only stdout and dropped running-process metadata, followed by overlapping build commands. Two active compilations were observed in the one-CPU container. It ended honestly reporting an incomplete installation. This is an observed trajectory, not proof of a universal Codex implementation defect.
- **OpenCode:** 11/11. Waited for builds, fixed aliases including `.pyx`, `fractions.gcd` and planarity attributes, reinstalled, then checked the example and suite. Quota pacing: 23.356 s.
- **Remaining improvements:** search all file types implicated by the requirement; exercise a representative operation of each required extension instead of import-only checks; recover from ambiguous patches with focused edits; reduce repeated context/calls. It is not established that removing the provider limit alone would produce a pass.

### 6. Build Debian pMARS without X11

Retain source, install the binary and verify simulation/debugger behavior.

- **Fenix original attempt:** 47.30 s, additional $0.00419035. Downloaded sources but encountered missing `dpkg-source`; interpreted an APT warning as an environment block and asked permission rather than building. The subsequent verifier failed during Debian package installation (404), never running pytest. Its raw reward=0 is not a valid functional grade and remains recorded.
- **Fenix infrastructure replacement:** same frozen binary, skills, model and configuration; 4/4. Installed dependencies, extracted Debian sources, compiled without X11 and batched simulation/dependency/debugger checks. Completed with evidence in one call. The contrast with its original attempt shows inconsistent autonomy.
- **Codex:** 4/4. Downloaded Debian source, installed missing tools, adjusted Makefile, built and checked the result/debugger.
- **OpenCode:** 4/4. Built Debian sources and checked output. Spent 120 s on `pmars -h` until its timeout, then checked the debugger with explicit input.
- **Remaining improvement:** distinguish missing dependencies from actual permission denials and repair the environment within existing authorization; use bounded waits and explicit input for interactive checks.

## Protocol and limitations

Codex 0.157.1; OpenCode 1.18.32; Fenix binary SHA256 `9c12ff4046cf267bd0f29876cebf8650b7a3cf39622fd1f5e31b91242ee8a910`. Dataset revision `7131e4375048a0e408a8fb404b5f499d726b695b`. Fresh official containers, one CPU, 2 GiB, 900-second agent deadline. Sequential runs with rotating harness order; hidden tests were never supplied to evaluated agents. All completed responses confirmed GPT-6 Luna low/default; no HTTP 429s occurred. JSON contains tokens, costs, delays, raw grades and private trace locations.

The pMARS retry was chosen because the verifier report was absent, before knowing the replacement outcome; the original behavior and cost are preserved. No harness changes during measurement or best-of selection. Three new diagnostic tasks do not establish broad superiority or a paired before/after improvement.
