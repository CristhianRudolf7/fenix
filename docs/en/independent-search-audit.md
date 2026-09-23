# Independent search verification — 2026-09-19

[Español](../es/independent-search-audit.md)

Fénix contacted eight original public pages directly. Seven were indexed and
retrieved for their topic; Reddit was not read because its robots policy disallowed
it. No hosted search service, metasearch, browser service or LLM was used.

This is an ingestion/retrieval smoke test using **known source URLs**, not evidence
of query-based worldwide discovery or general answer accuracy. The initial corpus
was empty, background crawling was disabled and source queries ran in groups of
three. Times include source fetching, robots checks and origin pacing.

| Topic | Source ingestion + query (ms) | Hits | Source status |
| --- | ---: | ---: | --- |
| Rust ownership | 2915 | 1 | read |
| Python concurrency | 2850 | 1 | read |
| SQLite durability | 2981 | 1 | read |
| Web layout | 4999 | 1 | read |
| Astronomy | 2999 | 1 | read |
| Public health | 2419 | 1 | read |
| Community front page | 2588 | 1 | read |
| Public Reddit page | 169 | 0 | failed |

Total wall time: 10.60 seconds; 16 HTTP attempts including robots.
Repeating the eight searches with no source URLs and `read_pages: 0` made **zero HTTP
requests**, with observed local lookup times of 1.23–6.59 ms on this small corpus.
These timings are a single-run observation, not a production latency guarantee.

The [raw report](../benchmarks/independent-search-2026-09-19.json) retains URLs,
coverage, warnings, dates, failures and per-query timings. Reproduce with:

```sh
cargo run --locked -p fenix-web --example independent_audit -- target/verification/independent-search.json
```

Automated checks separately cover publication metadata vs fetch time, date/domain
filtering, refreshed-page scope, duplicate bodies, bounded candidate selection,
RSS/Atom discovery, legacy SQLite migration, robots, cancellation and source errors.
The actual Chromium fixture checks navigation, input, viewports and persistent
profile state. Browser UI tests exercise our API contract without a model/profile,
escaped results, explicit source failures and late-response disposal.

Our architecture uses a native Rust crawler/FTS5 index and a Rust-controlled local
Chromium browser. It has no search-provider adapters or credential settings. Known
search-service endpoints are also rejected by URL policy; this list complements
the architecture, not an exhaustive classification of all internet services.

## Final checks

- 32 native web tests passed, including crawl-budget reservation and persistence.
- 6 agent/server web integration tests passed.
- 9 skill migration, metadata and retrieval tests passed.
- 3 local browser tests passed, including the real Chromium fixture.
- 4 focused search UI tests and 3 application regressions passed (welcome, browser, Design).
- Frontend production build, Rust build, formatting and Clippy with warnings denied passed.

These are targeted checks, not a claim that the entire workspace suite was rerun.
The crawler now stops at its configured share of the daily budget; explicit reads
retain the total cap. In production this deployment uses 2,500 requests/day with a
20% foreground reserve (2,000 crawler-start threshold), preserving the existing
request counter and index. Repository defaults remain 2,000 requests/day.
