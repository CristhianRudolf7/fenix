# Independent web search and source reading

[Español](../es/web-search.md)

Fénix provides `web_search` and `web_read` as agent tools. The owner asks in the
existing conversation; there is no separate search screen, search subscription,
search API key, or model call inside the search implementation. The ordinary
conversation still uses the configured model provider.

Search operates on **this installation's own, gradually growing public corpus**.
It accepts any topic, but cannot promise an answer about every topic. A fresh
installation has no indexed pages. Neither an empty result nor a small result
set describes the whole internet. This is a bounded crawler and search engine,
not an already populated alternative to a worldwide search index.

## How it works

1. The executable starts a background Rust worker, independently of conversation
   and planning workers. It loads a persistent URL frontier and fetches public
   pages, following permitted links to other domains and same-origin XML sitemaps.
2. HTML extraction retains article/main text where available, drops common
   navigation, scripts and hidden text, and writes bounded text and metadata into
   a separate SQLite FTS5 index. Exact content hashes deduplicate search results.
3. `web_search` runs a local Unicode lexical query, ranks with BM25 and a title
   boost, and returns URLs, titles, excerpts, indexing times and corpus coverage.
   It tries all query terms first and fills sparse results with individual-term
   matches. It does not provide semantic translation, embeddings or synonym expansion.
4. By default, the tool also reads up to three results. Recent cached text is
   reused; older pages are fetched, or conditionally revalidated with ETag or
   Last-Modified. `web_read` can fetch a known URL even before it has been indexed.

Initial addresses include Wikipedia in Spanish and English, gob.pe, NASA, the UN,
WHO, World Bank, Smithsonian, BBC Mundo, arXiv, MDN and the Rust book. They are
**discovery seeds, not an allowlist**. Ordinary permitted hyperlinks expand the
frontier to other public domains. These seeds bias early coverage; a site's crawl
policy, download size or availability can prevent its inclusion. There is no
background query to Brave, Google, Bing, DuckDuckGo, Tavily or a metasearch service.

## Agent contract

The only required search argument is the query:

```json
{"query": "photosynthesis in plants"}
```

Optional `limit` defaults to 5 and accepts 1–10; `read_pages` defaults to 3 and
accepts 0–3. Setting `read_pages: 0` searches the disk index without HTTP requests
from that call. The independent crawler may still be active. Queries accept
1–400 characters and must contain letters or numbers.

For a known source:

```json
{"url": "https://doc.rust-lang.org/book/ch01-01-installation.html", "query": "installation"}
```

`query` is optional for `web_read` and focuses the excerpt. URLs are required to be
absolute public HTTP(S) addresses on ports 80 or 443, without embedded credentials.

Search results contain `backend: "local_fts5"`, `coverage`, `results` and `warnings`.
Each hit has `url`, `title`, a snippet of at most 350 characters, `indexed_at` and
`read_status`. Successful second-phase reads add `page` with a passage of at most
2,400 characters, URL, title, `retrieved_at`, SHA-256 `content_hash`, language,
truncation flag and `source` (`network`, `cache`, `revalidated` or `shared`).
`shared` means this call reused another in-flight read of the same URL. Direct reads
return up to 6,000 characters. Failed second-phase reads expose `read_error` and
retain the older indexed snippet with an explicit warning. They are not reported
as successful current verification. `not_requested` means no read was attempted
for that hit.

The agent must cite source URLs and distinguish stored evidence from fresh
retrieval. Fetch/check times are **not publication dates**. Public text is
untrusted evidence: page instructions cannot authorize actions, request secrets,
change project scope or override the conversation. Empty results include a
coverage warning; the agent should explain it without inventing sources or
polling continuously.

## Concurrent searches and downloads

`max_concurrent_searches` defaults to **10**, with a validated range of 1–10.
Searches and direct `web_read` calls share that foreground admission limit across
all agents. An eleventh call waits for capacity, with its own 30-second admission
timeout. `queue_wait_ms` reports that wait; it does not consume the subsequent
35-second source-reading budget. Cancelling a queued or active call releases its
slots. Old configuration files without this field automatically use ten.

The HTTP pool has the same global cap, including robots, redirects and the
crawler. Different origins download in parallel. Each origin has one active
request and a persisted minimum interval; waiting on an origin does not hold a
global download permit. The crawler begins new work only when no foreground
operations are active or queued; an already-started crawl may finish.

A search starts up to three result reads concurrently, preserving result order.
Concurrent readers of the same URL share its full refresh and result, even when
`cache_minutes=0`. Robots loads are also combined by origin. Completed shared
results are released when their callers leave; this does not create an extra
persistent cache. Cancelling the leader lets a surviving waiter take over; no
network worker is detached. Bounded HTML parsing may finish after cancellation.

If one model response contains only `web_search`/`web_read` calls, the agent engine
runs those calls together under the shared limit. Each completed result is
redacted and journaled immediately; model-facing results retain call order.
Responses mixing web tools with other tools keep their existing execution order.
The web limit does not change how many model subagents the planner may spawn.

See the [live concurrency and memory measurement](web-search-concurrency.md) for
the ten-topic comparison and reproduction commands.

## Local operation and configuration

Start Fénix normally:

```sh
cargo run --locked -p fenix-server --bin fenix
```

The service must remain running for crawling to progress. The crawler also runs
before model onboarding, and makes no model calls. Shutdown cancels its in-flight
HTTP operation. A restart resumes URL discovery from the index; this is unrelated
to resuming agent tasks or planning effects.

Runtime files under `--data-dir` (default `.fenix/`) are:

- `web-search.json`: generated configuration; edit while stopped, then restart.
- `web/index.sqlite3` and its SQLite WAL files: public text, FTS index, pending
  URLs, cached robots policies, retry times and persistent request counters.

This database does not read conversation history, private memory, project files,
provider credentials or Telegram data. URLs requested by agents enter the public
corpus; URL values themselves can disclose information to the destination site.
Known credential query parameters are rejected, but this does not classify every
possible sensitive value in arbitrary URLs. Tools send no login cookies,
authorization headers or referers and do not inherit environment HTTP proxies.

Default budgets are conservative starting points, not measured RAM guarantees:

| Setting | Default | Effect |
| --- | --- | --- |
| `enabled` | `true` | Enable crawler and both web tools. |
| `max_concurrent_searches` | 10 | Shared foreground-call and global-download caps; accepts 1–10. |
| `max_pages` | 5,000 | Stored article cap; existing pages can refresh at capacity. |
| `max_frontier` | 20,000 | Maximum known/pending URLs retained. |
| `max_urls_per_origin` | 200 | Limit one origin's share of the frontier. |
| `max_body_bytes` | 524,288 | Maximum decompressed page download (512 KiB). |
| `max_text_chars` | 24,000 | Maximum stored text per page. |
| `request_interval_ms` | 2,000 | Minimum delay per origin; also the active crawl-loop pause. |
| `requests_per_day` | 2,000 | Shared HTTP attempt budget per UTC day, including robots and redirects. |
| `refresh_hours` | 72 | Next background refresh after a successful crawl. |
| `cache_minutes` | 15 | Reuse cached text before checking the source again; 0 forces checking. |

Up to ten HTTP requests can be active, with one HTML parser shared by master,
children and crawler. Downloads are bounded individually; multiple bounded
responses can wait for parsing. The index stays on disk, with a SQLite page cache target of
2 MiB and memory mapping disabled. There is no browser, local inference model or
vector index in this module. Total process RSS also includes TLS, extraction
structures, the Rust allocator and all other Fénix features; it is not capped at
2 MiB. Input, text, links, result counts and structural complexity are bounded.

Configuration validates practical ranges: 1–100,000 pages, 1–200,000 frontier URLs,
1–10,000 URLs/origin, 1 KiB–2 MiB downloads, 256–64,000 text characters,
100–60,000 ms intervals, 1–100,000 daily attempts, 1–720 refresh hours and
0–1,440 cache minutes. At most 100 seed URLs are accepted. Lowering a cap prevents
further growth; it does not automatically delete an existing larger corpus.

To disable all web activity, set `"enabled": false` and restart. To customize
initial coverage, edit `seeds`; existing URLs remain in the index and frontier.
An empty seed array allows an initially empty corpus to grow from explicit
`web_read` calls and their discovered links. Increasing limits can improve
coverage at the expense of disk use, crawl time and traffic; it does not guarantee
web-scale completeness. There is currently no automatic eviction/replacement of
stored pages or frontier URLs at capacity.

`GET /api/web/status` returns crawler state, last crawl error, indexed page/origin
counts, known/due URL counts, latest indexing time, today's attempts and the page
cap. It is protected by the existing localhost Host/Origin checks. It is a status
endpoint, not a public search or arbitrary HTTP proxy. It also exposes
`max_concurrent_searches`, `active_operations`, `queued_operations` and
`active_downloads`; direct reads count as foreground operations.

Library callers control network startup: `Web::open` and `Runtime::open` perform
no network I/O. `Runtime::start()` starts conversation/planning workers;
`Runtime::start_web_search()` explicitly starts the crawler. The executable calls
both and joins them at shutdown. This keeps existing offline runtime fixtures
offline.

## Fetch behavior and limits

- Before requests, validate the URL and all resolved addresses and pin the
  approved addresses to that request's HTTP client. Reject loopback, private,
  reserved, link-local and disallowed IPv6 transition ranges. Recheck every page
  and robots redirect; the caller cannot pass a DNS override.
- Honor robots.txt with the `FenixBot` user agent. Cache policies for 24 hours;
  network/server failures use a conservative temporary denial for five minutes.
  401, 403 and 429 deny crawling; other unavailable 4xx policies permit crawling.
  Honor the largest parsed crawl-delay conservatively. Long delays may exceed the
  read budget. A missing robots file does not bypass page authorization.
- Respect HTML robots metadata and X-Robots-Tag noindex/noarchive/nofollow
  directives conservatively. Opt-out pages are not returned as article text or
  stored as searchable content. Sitemaps provide URLs, not search evidence.
- Limit redirects to five follows, connect time to five seconds and each request
  to 12 seconds. A full fetch, including queue wait and robots, has 30 seconds;
  second-phase search reading shares a 35-second budget, including origin waits.
  Direct reads also have a 35-second budget. Foreground admission can separately
  wait up to 30 seconds before either reading budget starts. Task cancellation drops
  pending network work. A running bounded parser/SQLite blocking task may finish.
- Persist retry backoff and daily budgets across restarts. Reaching the traffic
  budget leaves index-only search and fresh cached reads available. A leased URL
  interrupted during crawling becomes eligible again after one minute.

HTTP extraction supports HTML/XHTML, plain text and XML sitemaps. It does not
execute JavaScript, compute external CSS visibility, solve CAPTCHAs, sign in,
extract PDFs, submit forms, make purchases, reserve flights, inspect screenshots,
test rendered sites or change viewport size. Interactive browsing and responsive
testing remain separate future work.

## Verification

See the [ten-topic live benchmark](web-search-benchmark.md) for measured latency,
RSS, coverage and shared-instance concurrency, including observed failed reads.

```sh
cargo test --locked -p fenix-web
cargo test --locked -p fenix-agent -p fenix-server --test web_search
```

Local HTTP fixtures exercise link discovery across origins, sitemap discovery,
lexical retrieval, source reading, persistence, deduplication, caps, cache and
304 revalidation, robots denials, redirected private addresses, opt-out content,
download limits, cancellation and honest empty/failed reads. Agent integration
uses a mocked model and requires no provider key or external website.

For an explicit live HTTP smoke check in a temporary index, without an LLM:

```sh
cargo run --locked -p fenix-web --example read_search -- \
  https://doc.rust-lang.org/book/ch01-01-installation.html installation
```

Implementation references: [SQLite FTS5](https://www.sqlite.org/fts5.html),
[robots exclusion protocol, RFC 9309](https://www.rfc-editor.org/rfc/rfc9309.html),
[reqwest client configuration](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html).
