# Fenix: tasks 27–31 audit

[English](fenix-tasks-27-31-luna-low-2026-09-26.md) · [Español](fenix-tasks-27-31-luna-low-2026-09-26.es.md)

Official result: **3/5 tasks passed (60%)**. Fenix only, GPT‑6 Luna low, no Flex or Jev; one attempt per task. This subset is not an overall Terminal-Bench score.

## Corrections before the five tasks

- Optional clarification or uninvestigated technical questions no longer establish owner unavailability by themselves in noninteractive mode. Dependency type, task quote and explanation are required; legitimate clarification, permissions and requirement checks remain enabled.
- Successful closure rejects explicitly unverified requirements. Out-of-scope caveats do not waive mandatory executable checks. These are consistency/provenance checks, not universal semantic proof.
- Updated format/data/image guidance: derive conventions from actual evidence, check the highest-risk assumption independently and distinguish file persistence from answer correctness.
- Pre-round validation: 261 Rust tests passed, 3 optional tests ignored, 4 gateway tests passed, server build passed.

## Measured results

| # | Task | Result | Checks | Agent time | Quota wait | Recorded USD cost |
|---|---|---|---|---|---|---|
| 27 | `hf-model-inference` | Passed | 4/4 | 202.6 s | 20.9 s | $0.02287 |
| 28 | `largest-eigenval` | Passed | 3/3 | 72.9 s | 0.0 s | $0.00901 |
| 29 | `make-doom-for-mips` | Failed | 0/3 | 900.3 s | 521.2 s | $0.08657* |
| 30 | `merge-diff-arc-agi-task` | Passed | 5/5 | 355.1 s | 46.6 s | $0.02590 |
| 31 | `model-extraction-relu-logits` | Failed | 0/1 | 471.7 s | 59.9 s | $0.03860 |

Estimated cost from available usage receipts: **$0.18295**. *One interrupted Doom call has no final usage receipt, so this is not a complete cost total. Total jobs including setup/verification/cleanup: 44.3 min. Agent time already includes quota wait; do not add the two columns.

Recorded tokens: 5,726,195 input (4,725,644 cache reads and 769,223 cache writes, included in input), 32,822 output. 167 calls with HTTP responses; the interrupted call limits these totals.

## Per-task audit

### 27. hf-model-inference

Downloaded the model and wrote a Flask endpoint; observed actual response-shape failure, removed an incorrect nested-list assumption, restarted its persistent service and verified positive/negative cases and invalid input. All four official checks passed. Extra calls came from repeated tool discovery, readiness/status polling and testing before restart.

Follow-up: Reinforce the existing service skill: dependency readiness, intentional restart after code changes, then bounded endpoint readiness and the behavioral check; reuse visible tool schemas. No blanket restart automation or new reviewer.

### 28. largest-eigenval

Initially tried complex power iteration, which failed the supplied evaluator. Replaced it with scipy.linalg.eig(check_finite=False), selected the eigenvalue of largest magnitude, and reran the evaluator successfully. The official eigen-pair, dominance and speed checks all passed. No owner question or unsupported closure was attempted.

Follow-up: Prefer a proven numerical primitive before implementing a custom optimization unless measurements justify it; check algorithm applicability to nonsymmetric/complex cases. Preserve the successful recovery flow rather than adding a numerical-specific runtime gate.

### 29. make-doom-for-mips

Built a MIPS ELF but never produced the required frame. An unsupported early failed closure was rejected and work continued. Repeated glibc/ABI variants and edits to the supplied VM did not establish a compatible minimal runtime. Several diagnostic shells printed that the frame was absent yet returned zero. Harbor stopped the attempt at 900 seconds; 521.185 seconds were gateway quota waits. The final API stream was interrupted with no usage receipt, so its cost is incomplete.

Follow-up: Keep the tail of long failure diagnostics in durable recovery state; test a tiny target executable and required I/O before porting a full program. Preserve the acceptance runner and avoid unvalidated no-op emulation. Use assertions/nonzero exit status for missing required output. Reinforce existing skills, not a new cross-compilation subsystem.

### 30. merge-diff-arc-agi-task

Passed all five official Git and hidden mapping checks. The revised question policy rejected an optional request for additional merge criteria; the agent continued from the supplied examples. An interactive tzdata package dialog caused two unchanged waits and an unsupported failed close; it then cancelled the stalled installer, recovered dpkg configuration/PATH and completed the merge. It recovered from trying to fetch/reset the checked-out branch by using a remote ref and detached checkout. The final mapping passed all supplied examples and the hidden case.

Follow-up: Expose the recent redacted log tail on a quiet-process advisory without moving incremental cursors or answering automatically; reinforce unattended setup and observed-prompt handling in the existing environment skill. Preserve effective optional-question rejection and Git recovery instead of adding a new Git workflow manager.

### 31. model-extraction-relu-logits

Wrote a query-based finite-difference/kink extractor, but read forward.py internals and fixed the unknown hidden width to the visible 20. It repeatedly used the actual A1 for local validation and tuned offsets, density and clustering; several unsupported failed closes were rejected. One edit had identical before/after hashes yet counted as recovery. Its eventual local cosine check passed, but the official verifier substituted a network with 30 neurons and a different scale/seed: 20 of 30 reference directions failed the required tolerance. The fixed count and incomplete generality violated the unknown-shape requirement; privileged local validation was not evidence of a black-box solution.

Follow-up: Use existing native before/after hashes to avoid counting unchanged edits as recovery or invalidating fresh checks. Strengthen existing guidance for query-only information boundaries, unknown dimensions inferred from permitted evidence, independent input variation and real failing assertions. Do not hard-code the hidden network or add a task-specific extractor to the harness. These changes still require a future controlled benchmark run to establish effectiveness.

## Changes after the round

- Durable diagnostics preserve both head and tail. Distinct terminal errors following long shared warning prefixes no longer collapse into the same stagnation signature solely for that reason; complete receipts remain recoverable.
- Quiet-process advisories replay at most 1,200 recent redacted log bytes, explicitly labeled as replay. Reader cursors, task ownership, cancellation and deadlines remain intact; the engine never answers prompts or kills processes based on silence.
- An edit with equal native before/after hashes is not a repair and does not invalidate current checks. Missing equality evidence keeps conservative behavior.
- Existing service, environment, build, performance, testing and reverse-engineering skills were updated: restart edited services, unattended setup, minimal ABI smoke before large ports, proven algorithms, real assertions and information boundaries. Versioned hashes preserve customized copies.
- Post-round validation: **265 Rust tests passed, 3 optional tests ignored**, server build passed. All four new regression tests failed before the correction and pass afterwards. No new models, reviewers or subsystems.
- These final changes were not remeasured on Terminal-Bench. The table describes the earlier binary frozen across all five tasks. No deployment or GitHub push.

## Method and limits

- Selection fixed before outcomes: next alphabetically unattempted text-input CPU tasks with official timeout ≤900 s. One attempt, official resources: 1 CPU and 2 GiB except ARC with 4 GiB.
- Fresh containers, no manual task intervention. Unmodified verifiers were audited only after completion. Skill learning disabled; other auxiliary calls included in usage.
- Host disk capacity required building the HF image from its unmodified official Dockerfile; other tasks used published images. Instructions, resources and verifiers were unchanged.
- Doom suffered substantial quota waiting; no control establishes that more active time would solve it. ReLU consulted internals, so it does not demonstrate clean black-box reconstruction regardless of official score.
- Comparing this subset with the previous five different tasks cannot establish a causal percentage improvement. Receipt checks and guidance do not guarantee semantic correctness.
- Recorded per-million tariff: input $0.10, cache read $0.01, cache write $0.125, output $0.50. Usage estimate, not invoice; excludes infrastructure and taxes.

Dataset: `7131e4375048a0e408a8fb404b5f499d726b695b`.
Evaluated binary SHA-256: `452d4439a01eaac559de9c8e108e5ad2f4f9151fc47c34f90230f2e6c94e06e2`.
Post-round binary SHA-256: `0b9fbcd3a0183de4d76396e3638f99719ea01351334fda6a0bc146161360ced0`.
Git HEAD: `7a0b28e6697abf3d71af001463da744ea5aee2e6`; `main`, with snapshotted local changes.

[Machine-readable metrics](fenix-tasks-27-31-luna-low-2026-09-26.json). Raw traces and credentials stay outside the repository.
