# Optional typed decisions with Jev

[Español](../es/decision-layer.md)

Jev is an optional decision service, not a replacement for the execution model. It can reorder up to eight authorized skill or memory candidates, request a bounded memory-search expansion, classify memory pairs for review, propose a single-project route, and request extra completion review. Rust remains responsible for permissions, project boundaries, persistence and mandatory audits. Memory classification preserves the original facts.

The feature is disabled by default. Authenticated, user-scoped `GET /api/decisions/settings` returns settings and health without exposing credentials. `POST /api/decisions/settings` accepts the following configuration; supply a real key through the existing protected configuration channel, never in a checked-in file:

```json
{
  "enabled": true,
  "shadow": true,
  "endpoint": "https://ai-gateway.vercel.sh/typesafe/v1/systemone",
  "model": "typesafe-ai/jev",
  "minimum_confidence": 0.85,
  "timeout_ms": 1500,
  "input_usd_per_million": 0.042,
  "api_key": "REPLACE_LOCALLY"
}
```

Shadow mode records decisions but does not apply them. Set `shadow` to `false` after validating quality. Confidence is not proof of correctness. Uncertainty, invalid responses, missing credentials, timeouts and provider failures preserve the existing execution path. A short circuit breaker prevents repeated failed requests. Identical requests have a 60-second runtime-local cache; different users do not share this cache. Only bounded, redacted excerpts are sent to the service, but they may still contain private task data.

Routing only handles bounded single-project cases. Multi-project instructions and unresolved references retain the existing router. A completion decision may request additional review, but cannot certify completion or remove an audit. Memory expansion keeps the same project scope and configured result limit.

## Evaluation and costs

Native evaluation receipts track successful decision calls, cache hits, fallbacks, provider-reported tokens and estimated decision cost separately from execution-model usage. The configurable input price is a planning estimate, not an invoice; promotions, failed requests and missing usage prevent interpreting it as billed spend. Extra ranking calls can increase costs. Savings are plausible when a decision replaces a generative routing or memory-review call, but must be measured on the same tasks with and without this feature.

The authorized connectivity check on 2026-09-24 returned HTTP 403, `customer_verification_required`, in approximately 0.70 seconds. Vercel requires a valid card on the account before serving this key. No successful live inference or real cost reduction was measured. Local simulated-provider tests exercise caching, scope isolation, timeouts, malformed answers, confidence, shadow mode and reranking; storage tests verify that additional review does not mark work complete.

References: [TypeSafe API](https://docs.typesafe.ai/introduction/quickstart), [confidence](https://docs.typesafe.ai/confidence), [Vercel compatibility](https://vercel.com/changelog/ai-gateway-now-supports-typesafe-clients-and-http-api-for-jev).

## Live validation after account verification (2026-09-24)

After account verification, 14 directed live checks passed in isolated Rust runtimes, alongside 41 regression tests. Ten classification cases matched their expected labels (mean 329 ms, range 226–468 ms). The multi-project case correctly abstained; one correct skill relevance label fell below the acceptance threshold and was not applied. Integrated checks covered skill ordering, zero-token cache reuse, completion review flags, and a persisted memory assessment without modifying original facts or calling the generative model (843 ms).

The ten classification responses reported $0 through the promotion and $0.00016338 in market cost. Across all 13 non-cached calls, usage was 5,503 input and 707 output tokens; the configured input-price estimate is $0.000231126, not an invoice. Other integrated calls did not retain billed-cost metadata. This is not a paired full-task benchmark, so a percentage cost reduction is not established. Production remains unchanged. See [machine-readable results](../benchmarks/jev-runtime-2026-09-24.json).

Opt-in live tests require `FENIX_JEV_KEY_FILE` pointing to an external credential file, `FENIX_JEV_REPORT` and `FENIX_JEV_MEMORY_REPORT` pointing to private output files. Run `cargo test -p fenix-agent live_jev_ --lib -- --ignored --test-threads=1`. These tests issue external requests; ordinary test runs skip them.

### One coding trial with Luna low (2026-09-25)

The unchanged `build-cython-ext` task and verifier ran once with GPT-6 Luna low, revised diagnosis/action instructions, two verified independent OpenAI organization quotas and Jev enabled. Fenix finished its own execution in 714.31 seconds; the official verifier passed 9/11 checks (reward 0), versus 2/11 and a timeout in the preceding max-reasoning run. Quota waits fell from 379.6 to 13.862 seconds, with no 429 responses. The two credentials served 18 and 17 requests including startup validation.

Jev received one diagnostic request (477 ms), proposed repair at confidence 0.41, and was correctly not applied at the 0.85 threshold. There is no demonstrated contribution from Jev to this improvement. Multiple settings changed, so this is not a controlled attribution study.

The remaining failures were a runtime `np.int` use in a Cython extension and the repository regression suite. The agent had observed missing pytest and still marked the task completed without installing the test dependency or completing the required suite. Global installation, extension imports and the requested example succeeded. No additional benchmark attempt or production deployment followed. See [results](../benchmarks/luna-low-diagnostic-jev-2026-09-25.json).
