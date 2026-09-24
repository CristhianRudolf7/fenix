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
