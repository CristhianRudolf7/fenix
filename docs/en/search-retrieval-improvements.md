# Search retrieval improvements — 20 September 2026

[Español](../es/search-retrieval-improvements.md)

The search engine now retrieves actual reference pages for previously failing
React, Next.js and Tokio queries. Fresh source checks are enabled by default in
both agent tools. This is still a bounded independent crawler/index, not a
worldwide search service.

## What failed and what changed

- The baseline returned gob.pe for React and NASA for Next.js/Tokio. Weak OR
  matching and poorly targeted entry pages caused these false positives. Shared
  query normalization, topical/proximity filtering and official source roots
  now focus retrieval. The root catalog is a bootstrap, not an answer database.
- Large reference pages exceeded the old 512 KiB download allowance. New defaults
  allow 2 MiB decompressed HTML and 64,000 extracted characters. Code indentation
  and highlighted line boundaries survive extraction; passages allow 6,000 characters.
- The first 100 links, a full frontier, and loss of fragment anchor labels hid
  useful references. Extraction now retains up to 2,000 links, merges labels and
  stores bounded link provenance. Observed links remain usable when the crawl
  frontier is full. Concurrent searches cannot borrow unrelated discovery links.
- Discovery could stop at an overview. It now prioritizes whole path words,
  normalizes dotted product names and allows two expansion waves before its
  lexical early-stop rule. Maximums remain four waves, 16 links and eight per origin.
- Cached snippets could look current after a failed or outdated read. The agent
  now defaults to `fresh: true`; revalidated content updates passages and snippets,
  and changed irrelevant pages are removed from results. A read already completed
  in the same call is reused, including when the TTL is zero. Read provenance and
  failures remain explicit.
- The research skill checks relevance, versions and source language, follows up
  concrete gaps with a relevant domain, and does not treat nonempty results as an
  answer. No hosted search API, platform search endpoint or browser was added.

## Method and results

The real Rust `Web::search_with` implementation was invoked with only query text,
`discover: true` and `fresh: true`. Each of 24 cold cases had its own empty temporary
corpus; three independent cases ran together. A fixed seed (20260920) selected 12
queries from a curated pool of 24; the remaining 12 formed a complementary batch.
Both batches were used during iteration: the file named `holdout` is **not** a
blinded evaluation. The pool favors technical subjects and known source roots;
these percentages do not estimate whole-web coverage.

**23/24 cold queries returned sources. Manual review classified 22 as having a
relevant source, one as partial and one as empty.** Relevance is not proof that
an agent can answer every factual question from the passage. Some secondary hits
remain broad landing pages or translations in another language.

- Cold-query median: **12.92 s** across 24 cases; range **10.17–42.86 s**.
- Each batch also ran ten calls simultaneously through one shared, initially empty
  engine: **50.87 s total** for batch A, **41.23 s total** for batch B. These are
  batch wall times, not time multiplied by ten. Batch A's nine nonempty queries
  finished within 32.66 s; the empty recipe query finished at 50.86 s.
- Process peak RSS: **62.7 MiB** and **75.9 MiB** respectively. These measurements
  include the debug audit process and retained results, but exclude Fénix's model,
  browser and server. They are observations, not a hard RAM ceiling.
- Shared parallel runs made 103 and 105 HTTP attempts respectively, including
  robots and redirects. Both ended with zero active/queued operations and no
  active downloads. Same-origin pacing remains in force.
- The separate documentation controls used approximately **0.37–1.90 MiB** of
  temporary corpus storage per case. No steady-state full-index disk/RAM test or
  isolated CPU benchmark was performed. Other compilation work ran on the host.

| Query | Seconds | Source review | Selected source |
| --- | ---: | --- | --- |
| IPCC climate change | 10.65 | Relevant | [Page](https://www.ipcc.ch/) |
| Python asyncio TaskGroup | 16.88 | Relevant | [Page](https://docs.python.org/3/library/asyncio-task.html) |
| receta peruana de ají de gallina | 42.86 | Empty | — |
| Docker multi stage builds | 10.30 | Relevant | [Page](https://docs.docker.com/get-started/docker-concepts/building-images/multi-stage-builds/) |
| React useEffect cleanup | 10.58 | Relevant | [Page](https://react.dev/reference/react/useEffect) |
| Git rebase conflicts | 11.78 | Relevant | [Page](https://git-scm.com/docs/git-rebase) |
| Busca documentación actualizada de React useEffect cleanup | 12.89 | Relevant | [Page](https://react.dev/reference/react/useEffect) |
| NASA Mars atmosphere | 12.97 | Relevant | [Page](https://science.nasa.gov/mars/) |
| Tokio timeout | 12.95 | Relevant | [Page](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) |
| NASA solar eclipse | 14.24 | Relevant | [Page](https://science.nasa.gov/science-research/heliophysics/nasa-science-soars-during-august-total-solar-eclipse/) |
| World Bank poverty | 12.65 | Relevant | [Page](https://www.worldbank.org/ext/en/topic/poverty/measuring-poverty) |
| PostgreSQL window functions | 11.64 | Relevant | [Page](https://www.postgresql.org/docs/current/functions-window.html) |
| Next.js proxy middleware | 11.08 | Relevant | [Page](https://nextjs.org/docs/app/getting-started/proxy.md) |
| SQLite transactions isolation | 10.17 | Relevant | [Page](https://www.sqlite.org/isolation.html) |
| Rust ownership borrowing | 27.04 | Relevant | [Page](https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html) |
| Kubernetes deployment rolling update | 17.81 | Relevant | [Page](https://kubernetes.io/docs/tasks/run-application/update-deployment-rolling/) |
| TypeScript generics | 12.87 | Relevant | [Page](https://www.typescriptlang.org/docs/handbook/2/generics.html) |
| CSS grid layout | 14.19 | Relevant | [Page](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Grid_layout) |
| NASA Jupiter moons | 14.45 | Relevant | [Page](https://science.nasa.gov/jupiter/jupiter-moons/) |
| WHO malaria prevention | 10.71 | Relevant | [Page](https://www.who.int/activities/promoting-malaria-chemoprevention) |
| WHO dengue symptoms | 16.32 | Relevant | [Page](https://www.who.int/health-topics/dengue-and-severe-dengue) |
| Context7 API context | 13.31 | Partial | [Page](https://github.com/upstash/context7) |
| Python pathlib Path | 10.33 | Relevant | [Page](https://docs.python.org/3/library/pathlib.html) |
| historia del imperio inca | 22.77 | Relevant | [Page](https://es.wikipedia.org/wiki/Portal:Imperio_incaico) |

The Context7 query-only case reached the project's own README, but not the specific
API reference. The focused `domains: ["context7.com"]` control reached its actual
GET-documentation-context reference in 10.38 s. The recipe case remained empty;
adding unrelated results would conceal insufficient discovery coverage. The
history query included useful Spanish sources but ranked other-language versions
first. Domain/language selection by the agent still matters.

## Documentation controls and evidence

The independent documentation audit repeated three query-only cases, three domain
cases, four exact-URL controls and a zero-TTL control. Query-only React/Next.js/Tokio
now returned their references instead of the baseline's unrelated pages. Exact
source URLs are assisted controls, not successful URL discovery. The legacy Rust
`read()` retains TTL behavior for compatibility; agent `web_read` calls
`read_with_fresh()`. Freshness behavior is covered by changing-source fixtures.

- [Random batch](../benchmarks/retrieval-random-2026-09-20.json)
- [Complementary batch](../benchmarks/retrieval-holdout-2026-09-20.json)
- [Documentation controls](../benchmarks/live-documentation-search-improved-2026-09-20.json)
- [Original baseline](../benchmarks/live-documentation-search-2026-09-20.json)

Reports retain URLs, timing, read/failure metadata and passage hashes/lengths;
full passages were removed after review. Initial/intermediate audit files are
iteration evidence and are not the final results.

## Validation and reproduction

Passed: **60 engine tests**, **4 agent integration tests**, strict Clippy, package
formatting, skill validation and `cargo check -p fenix-server`. Fixtures cover
changed evidence, code extraction, sitemap limits, full frontier behavior, anchor
labels, discovery isolation, cancellation, SSRF/robots and ten-call concurrency.
The agent integration uses a mock model provider; live audits exercised the real
search engine directly, not a live model's final answers.

```sh
cargo test -p fenix-web --lib
cargo test -p fenix-agent --test web_search
cargo clippy -p fenix-web --all-targets -- -D warnings
cargo check -p fenix-server
cargo run -p fenix-web --example retrieval_audit -- /tmp/fenix-random-audit.json
cargo run -p fenix-web --example retrieval_audit -- /tmp/fenix-extra-audit.json docs/benchmarks/retrieval-holdout-queries.json
cargo run -p fenix-web --example live_documentation_audit -- /tmp/fenix-documentation-audit.json
```

## Remaining limits and activation

Arbitrary topics can still be absent from the bounded corpus. JavaScript-only
content, robots restrictions, login walls, download/structure limits, lexical
ambiguity and missing publication metadata remain real limits. The module does
not bypass restrictions or import private browser sessions. `fresh` checks the
current server response; it does not establish factual correctness or completeness.

Changes are implemented and tested in the workspace. The already-running Fénix
installation was not replaced or restarted during this audit. Loading the changes
there requires rebuilding/deploying that installation. The workspace's local
legacy default resource settings were updated; custom limits elsewhere remain
untouched, and bundled skill upgrades preserve user customizations.
