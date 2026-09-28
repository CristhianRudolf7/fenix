# Three-harness Terminal-Bench diagnostic — 2026-09-26

[Español](three-harness-terminal-luna-low-2026-09-26.es.md) · [Measured data](three-harness-terminal-luna-low-2026-09-26.json)

Same GPT-6 Luna low, standard processing; official Terminal-Bench 2.1 task containers and verifiers, revision `7131e4375048a0e408a8fb404b5f499d726b695b`. One valid attempt each, fresh state, 900-second official task deadline, 1 CPU and 2 GiB memory. Fénix source and executable were frozen before running. No agent behavior was changed during the comparison and nothing was deployed. API credentials remained on the host recording gateway.

Fénix executable SHA-256: `d347bb35a50af58f0c2482672c4e222b82019d4a72a27848157ed01b43d93174`. Codex 0.157.1 and OpenCode 1.18.32. Source revision `7a0b28e6697abf3d71af001463da744ea5aee2e6` plus the preserved working-tree patch and new source files. All successful provider responses confirmed the same model, low effort and default service tier. No provider 429 or missing usage was observed.

Time is Harbor's agent execution phase, including runtime startup, excluding image installation, setup, grading and teardown. Fénix also reports inner evaluator time: 23.63 / 42.09 / 31.11 seconds, in task order. The Git measurement includes waiting for a user answer; this is not a successful resolution.

USD cost is a token-derived estimate, not an invoice. Per million tokens: uncached input $0.10, cache reads $0.01, cache writes $0.125, output $0.50. Cache writes are separated from ordinary input. [Official pricing](https://developers.openai.com/api/docs/pricing). All prompts stayed within the short-context tier.

| Task | Harness | Pass | Execution (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| cancel-async-tasks | Fénix | 1 | 38.82 | 0.003814 | 46,458 | 1,025 |
| cancel-async-tasks | Codex | 0 | 13.93 | 0.002918 | 29,079 | 741 |
| cancel-async-tasks | OpenCode | 0 | 39.70 | 0.003192 | 54,360 | 1,998 |
| fix-git | Fénix | 0 | 55.79 | 0.004766 | 86,245 | 832 |
| fix-git | Codex | 1 | 24.76 | 0.005387 | 98,502 | 1,817 |
| fix-git | OpenCode | 1 | 45.22 | 0.003987 | 93,614 | 1,676 |
| db-wal-recovery | Fénix | 1 | 44.51 | 0.005379 | 114,384 | 1,617 |
| db-wal-recovery | Codex | 0 | 38.33 | 0.006013 | 147,579 | 2,912 |
| db-wal-recovery | OpenCode | 0 | 49.67 | 0.004034 | 96,299 | 2,371 |

## Task audit

### 1. cancel-async-tasks

Implement bounded asynchronous concurrency while preserving cleanup on cancellation.

- Fénix: used `asyncio.TaskGroup` and a semaphore, ran a concurrency and cancellation test, and passed all 6 official checks.
- Codex: used `gather` plus explicit cancellation and only verified the import. Passed 5/6; failed cleanup when tasks outnumbered concurrency slots.
- OpenCode: used a similar `gather` approach, repaired a synchronization issue in its self-test and passed that test, but still failed the same queued-task cleanup case (5/6).
- Fénix improvement: retain this implementation approach; expand risk-focused tests to asynchronous cleanup with fewer/equal/more jobs than slots and an actual SIGINT. Its own single-job cancellation test did not cover the hardest case, despite the correct implementation. Measure initialization overhead separately before attributing the full wall time to model reasoning.

### 2. fix-git

Find a detached lost commit and merge it into master, resolving the website biography conflict.

- Fénix: loaded the Git recovery skill, found `c499730`, inspected its content/ancestry and attempted the merge. Git reported missing committer identity; it asked the user for name/email and stopped with `requires_user_input`. Both official checks failed.
- Root cause confirmed in source: `processes.rs` passes command cwd into `fenix_execution::local_environment`, which assigns `HOME` to that path and clears other environment values. The official image already initializes `/root/.gitconfig` with a test identity, which the other two tools could use. Thus the missing identity was introduced by Fénix's execution environment.
- Codex: recovered the commit via reflog, merged, encountered an unsuccessful patch attempt, inspected again and corrected the file. Final contents passed 2/2.
- OpenCode: recovered via reflog, tried fast-forward first, then merged and resolved the conflict. Passed 2/2.
- Fénix improvement: use a stable authorized per-user execution home; preserve or explicitly inject only the user's approved Git identity. Do not inherit unrelated host credentials or invent an identity. Before asking, distinguish missing configuration from configuration hidden by isolation. Validate merge ancestry, conflict markers and final working-tree state.

### 3. db-wal-recovery

Repair an obscured SQLite WAL and export all 11 exact records, including updates to existing rows.

- Fénix: inspected bytes with Python, detected XOR `0x42`, queried read-only, preserved the original WAL before rewriting it, decoded it, verified SQLite integrity and reread the exported JSON. Passed 7/7.
- Codex: opened the original database in writable mode and lost the WAL; attempted to locate it, then wrote a guessed sequence. Its final answer acknowledged the unverified rows. Passed structural checks but failed exact contents and WAL recovery (5/7).
- OpenCode: likewise opened the original before preserving the WAL, lost it and generated guesses. Its final answer explicitly acknowledged that they were guesses. Passed 5/7. Both missed WAL updates of IDs 1 and 2 (150/250 rather than base values 100/200).
- Fénix improvement: make backup of database, WAL and SHM precede any SQLite connection, even read-only, and perform recovery on a copy. Fénix succeeded here but its backup happened after an initial read-only query. Keep source provenance and compare exact values, not just schema or record count.

## Evaluation qualifications

Only three tasks and one valid sample per cell: outcomes can change with sampling and this is not a leaderboard score. Native harness instructions/tools differ intentionally. No task solutions or hidden verifier content were supplied to any agent.

Two preliminary Codex 0.154.0 runs were discarded because stock Harbor's temporary CODEX_HOME blocked helper aliases and that CLI lacked Luna metadata. Both preliminary runs passed, but neither was selected for the final score. The complete current npm distribution (including helpers) and a non-temporary container home fixed the setup. These preliminary runs cost an additional estimated $0.007494315 and remain in `discarded-setup-codex0154`.

OpenCode attempted one auxiliary request with a model other than the allowed evaluation model in each task. These were rejected locally before provider inference; they incur no recorded provider charge. No different-model response contributed to the task, but this auxiliary restriction is a configuration limitation of the comparison.

The machine also hosts other services. Attempts were sequential; infrastructure startup/install/verification are excluded from the execution column. No secret or production state was provided to task containers. Full scripts, raw responses, source snapshot and official verification artifacts are stored privately in the evaluation directory linked in the JSON report.
