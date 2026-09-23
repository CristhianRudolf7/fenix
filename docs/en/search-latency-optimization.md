# Search latency optimization — 20 September 2026

[Español](../es/search-latency-optimization.md)

Repeated searches now locate relevant indexed pages and revalidate them directly.
They no longer repeat an entire discovery traversal before reading a known source.
The four measured warm individual calls fell from **8.40–13.32 s** to
**0.23–2.13 s**. Ten warm concurrent calls fell from **28.49 s to 4.77 s total**.
These are observed timings, not a universal latency guarantee.

## Measured comparison

The same `latency_audit` example ran before and after the changes. Each process
started with an empty temporary persistent corpus, ran ten cold concurrent queries,
four warm individual queries, ten warm concurrent queries, one new related query,
and an explicitly offline lookup. Live queries used `fresh: true`. No model,
Chromium or search provider was used. The existing background worker was not
started in this controlled comparison, so queued follow-up work did not consume
benchmark HTTP capacity.

| Repeated query | Before | After | HTTP before → after |
| --- | ---: | ---: | ---: |
| React useEffect cleanup | 10.040 s | 0.281 s | 15 → 1 |
| Python asyncio TaskGroup | 13.316 s | 1.353 s | 12 → 1 |
| SQLite transactions isolation | 8.398 s | 0.227 s | 5 → 1 |
| WHO dengue symptoms | 9.448 s | 2.134 s | 9 → 2 |

| Measurement | Before | After |
| --- | ---: | ---: |
| Median of four warm individual queries | 9.744 s | 0.817 s |
| Ten cold concurrent calls, total | 23.731 s | 18.898 s |
| Cold group HTTP attempts | 88 | 87 |
| Ten warm concurrent calls, total | 28.491 s | 4.770 s |
| Warm group HTTP attempts | 91 | 17 |
| New related query, Python pathlib | 10.120 s | 7.916 s |
| Explicitly offline lookup | 159 ms | 21 ms |
| Process peak RSS | 69.3 MiB | 59.0 MiB |

The four warm calls used 41 HTTP attempts before and five afterward. Every returned
live passage in the optimized audit was network-fetched, conditionally revalidated
or shared from an in-flight fetch: none was labeled `cache`. The ten cold cases
returned the **same ordered source URL lists** before and after. The offline
control is not counted as a live search speed.

The separate 24-query regression also retained exactly the same source URL sets
as the previous implementation: 22 cases with relevant sources, one partial
Context7 README result, and one empty recipe query. The two cold ten-call groups
there took 44.43 s and 38.22 s, with peak process RSS of 68.3 and 77.7 MiB.
These cold groups must not be confused with the 4.77-second warm group above.

This is a small, curated, technical/scientific query sample and one run per build.
Internet latency varies, and compilation also ran on the shared host. Cold searches
still require discovery and can be much slower for missing or blocked topics.
RSS includes the debug harness, retained results and engine, not the full Fénix
server, browser or model. No isolated CPU or steady-state filled-index benchmark
was performed.

## Implementation

- The index supplies locators. Up to three strong candidates are checked directly;
  changed/failed evidence falls back to discovery. Explicit recency wording and
  publication windows keep fresh listing discovery.
- Navigation can use its bounded TTL cache, but `fresh: true` still revalidates
  selected evidence. Failed reads are not duplicated within the same call.
- Up to 16 pending observed URLs per call are prioritized for the existing crawler,
  with a 256-entry queue cap. Queueing does not mean the sources were read.
  Foreground reads satisfy pending crawl work and retain retry/refresh pacing.
- HTTP clients reuse connections, with at most 32 origin clients and one idle
  connection each. DNS is checked before every request; changed addresses replace
  the pinned client, and private addresses remain forbidden.
- ASCII lexical comparisons avoid temporary allocations. Link ingestion reuses
  prepared SQL statements and counts each origin once per transaction.
- A same-origin tie breaker no longer admits unrelated links. Search concurrency
  remains capped at ten and site-specific spacing is unchanged.

See [the current tool contract](web-search.md#index-first-latency-optimization).
The bundled research skill is version 1.5.0; known bundled versions can upgrade,
while owner customizations remain protected. No additional paid service, model
call or external search provider is required.

## Evidence and reproduction

- [Before](../benchmarks/search-latency-before-2026-09-20.json)
- [After](../benchmarks/search-latency-after-2026-09-20.json)
- [Random-topic regression](../benchmarks/retrieval-optimized-random-2026-09-20.json)
- [Complementary-topic regression](../benchmarks/retrieval-optimized-extra-2026-09-20.json)

The latency artifacts omit full passages and retain source URLs, read provenance,
content hashes, passage lengths, timings and HTTP counts. During concurrent calls,
per-call HTTP counter deltas overlap; use the aggregate batch totals.

```sh
cargo run -p fenix-web --example latency_audit -- /tmp/fenix-latency.json
cargo test -p fenix-web --lib
cargo test -p fenix-agent --test web_search
cargo clippy -p fenix-web --all-targets -- -D warnings
cargo check -p fenix-server
```

Validation passed: **71 engine tests**, **4 agent integration tests**, strict
Clippy, package formatting, skill validation and the server compilation check.
New cases cover warm current reads, changed/failed evidence, cached navigation,
news discovery, background priority/backoff, duplicate crawl prevention and bounded
HTTP-client/priority queues. Agent integration uses a mock model; the live audits
test the real search implementation directly.

Deployment is separate from these measurements: the already-running installation
was not replaced or restarted. It must load a build containing these changes to
use the optimized implementation.
