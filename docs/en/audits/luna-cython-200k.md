# Luna max with a 200k context window

[Español](../../es/audits/luna-cython-200k.md) · [Previous rounds](luna-cython-recovery.md)

The owner requested one additional build-cython-ext attempt on September 23,
2026. Context was set to 200,000 tokens, compaction to 160,000 and recent context
to 8,000. GPT-6 Luna max, the official task/verifier, and the 900-second Harbor
agent deadline remained unchanged. Fresh container and native Fénix runtime.

**Result: 2/11 official checks; reward 0. Native elapsed time: 859.33 seconds
(14m19s).** The run ended awaiting input after worker failure, not from the
context ceiling or the official deadline. Only NumPy version and clone checks passed.

No active-context compactions occurred. The largest recorded input was 90,864
tokens. Native counters: 1,771,277 input, 82,967 output, 51 model responses,
92 tool calls. These are cumulative counts, not the size of a single context.

Two failures remained:

1. The newly added explicit-completion guard rejected the master's progress-only
   messages while a delegated task was running. It exhausted three recoveries for
   completion_without_outcome and marked the master failed. This is an interaction
   between the new completion protocol and background delegation, not missing context.
2. The provider returned three HTTP 429 responses for the account's 200,000
   tokens/minute allowance. The worker exhausted its attempts. It had built native
   extensions and installed dependencies in an auxiliary virtual environment but
   had not completed the required global installation or compatibility repairs.

Transport limitation: the local recording proxy forwarded Content-Type but not
Retry-After. Fénix's provider client reads Retry-After, so the one-/two-second worker
backoffs observed here do not establish production behavior with normal response
headers. No further changes or extra attempt were made after this result.

This single attempt does not isolate context size statistically: model actions
are stochastic. It demonstrates that increasing the window did not resolve this
run's orchestration and rate-limit failures. The previous max/128k run passed 7/11
in 871.72 native seconds; the last low/32k run passed 8/11 in 660.16 seconds.

The code and web configuration defaults are now 200k/160k; explicit saved user
settings are preserved. Build and the context-budget regression test passed.
Changes were not deployed or pushed. Frozen protocol, patch, binary hash, raw
provider records and official results remain in the local
`cython-recovery-20260923` evaluation directory, job `iteration-10-max-200k`.
