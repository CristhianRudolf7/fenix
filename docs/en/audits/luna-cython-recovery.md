# Luna Cython recovery audit — September 23, 2026

[Español](../../es/audits/luna-cython-recovery.md)

The final requested attempt failed: **7/11 official checks**, reward 0, Harbor's
900-second agent timeout. Native runtime measured 871.72 seconds after setup and
provider preflight; full Harbor run took about 15m48s. Testing stopped at the
owner's request. No deployment or push was performed.

## Comparison

| Last measured configuration | Checks | Native seconds | Recorded input/output tokens | command_status calls |
|---|---:|---:|---:|---:|
| Luna low, iteration 5 | 8/11 | 660.16 | 798,942 / 8,315 | 21 |
| Luna max, iteration 9 | 7/11 | 871.72 | 751,059 / 31,636 | 2 |

Neither completed the official task. This is not an isolated reasoning-level A/B:
max used a newer harness, event waits and 128,000 configured context tokens
(compaction 80,000, recent 8,000); low used 32,768 (compaction 22,000). Native token
counters are not an invoice and may exclude auxiliary inference. Provider totals
including auxiliary calls are recorded in the private summary. One repeatedly
tuned task does not establish general model rankings or benchmark accuracy.

## Findings

The low attempt stopped on context compaction before finishing remaining repairs.
The max attempt compiled and installed all three native extensions and retained
NumPy 2.3.0, but its planning/review, repeated inspection and native builds left
insufficient time for compatibility repair and verification. It fixed the removed
Python fractions.gcd import, but NumPy int/float aliases remained. Official failures:
ccomplexity behavior, Python/Cython invariant agreement, README example, and the
repository suite (18 failing tests). The official test runner was not changed.

## Implemented changes

- Rust completion-event waiting: advisory review after 120 seconds or 60 seconds
  without output. No model inference while waiting; silence alone never cancels.
  Interactive/persistent sessions retain short waits and immediate reads remain available.
- Explicit execution outcomes; progress-only final answers cannot mark execution complete.
- Bounded observational contract history, retained source references, corrected
  compaction headroom and consistent token-estimator comparisons.
- Credential-pattern boundaries avoid false matches inside opaque provider ciphertext;
  known credentials remain matched literally.
- Same-scope workers retain original request literals without inheriting other scopes.
- Native evaluation recognizes plans blocked on user input with no active workers.
- Configuration accepts xhigh/max; protocol test verifies max is sent unchanged.
- Generic package-build skill emphasizes the requested environment and unresolved checks.

The configured 128k evaluation context is recorded, not silently applied to all users.
Luna max support was checked against [official model documentation](https://developers.openai.com/api/docs/models/gpt-6-luna)
and confirmed in recorded requests. All inferences used the existing authorized proxy.

## Attempts retained

| Job | Official checks passed | Native seconds | Harbor exception |
|---|---:|---:|---|
| iteration-1 | 2/11 | 230.42 | — |
| iteration-2 | —/11 | — | RuntimeError |
| iteration-2-startup-retry | 3/11 | 303.77 | — |
| iteration-3 | 3/11 | 205.47 | — |
| iteration-4 | 2/11 | 96.45 | — |
| iteration-5 | 8/11 | 660.16 | — |
| iteration-6-max | —/11 | — | RuntimeError |
| iteration-7-max | 1/11 | 75.01 | — |
| iteration-8-max | 1/11 | 881.01 | AgentTimeoutError |
| iteration-9-max | 7/11 | 871.72 | AgentTimeoutError |

Iteration 2 failed before inference because the recorder was unavailable;
iteration 6 failed before inference because configuration rejected max. These are
startup failures, not measured model outcomes. Iteration 7 exhausted the 32k local
context; iteration 8 requested omitted source details and remained blocked until
the official timeout. No hidden tests or reference solution were supplied to the agent.

## Validation and evidence

Targeted passing groups: processes 11, context 8, provider 30, execution contracts 8,
engine 3, coding harness 4, parallel worktree 1. The earlier full agent run had 198
passes, one ignored test, and one worktree timeout; that worktree test subsequently
passed alone. No further tests were started after the owner requested the final round.

Private artifact directory: `/home/Rudy/.local/share/fenix-evaluations/cython-recovery-20260923`. Protocols include binary hashes and source
patches; `all-attempts-summary.json` contains metrics and wait events. Raw provider
requests/responses, native reports and official verifier logs remain local.
The same official build-cython-ext task, dataset revision
7131e4375048a0e408a8fb404b5f499d726b695b and 900-second deadline were used.
