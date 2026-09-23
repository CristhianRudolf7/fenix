# Independent web search and source reading

[Español](../es/web-search.md)

Fénix provides `web_search` and `web_read` as agent tools. The owner asks in the
existing conversation or uses **Browser → Search or enter URL**. Our
`POST /api/web/search` API serves the same index. There is no search subscription,
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
   pages, following permitted links to other domains, same-origin XML sitemaps
   and original article links from RSS/Atom feeds. Feed text is not an article.
2. HTML extraction retains article/main text where available, drops common
   navigation, scripts and hidden text, and writes bounded text and metadata into
   a separate SQLite FTS5 index. Exact content hashes deduplicate search results.
3. `web_search` runs a local Unicode lexical query, ranks with BM25 and a title
   boost, and returns URLs, titles, excerpts, indexing times and corpus coverage.
   It tries all meaningful query terms first. Partial matches must cover at least
   60% of topic terms (all terms for queries of up to three topic terms),
   with matches close together. Source hostname words do not substitute for the topic. It does not provide semantic translation, embeddings or synonym expansion.
4. By default, the tool also reads up to three results. With `fresh: true` (default), sources are fetched or conditionally
   revalidated with ETag/Last-Modified. `fresh: false` permits the TTL cache. `web_read` can fetch a known URL even before it has been indexed.

Initial addresses include Wikipedia in Spanish and English, gob.pe, NASA, the UN,
WHO, World Bank, Smithsonian, BBC Mundo, arXiv, MDN, the Rust book and public
Hacker News/Reddit feeds. They are
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
accepts 0–3. Setting `discover: false`, `read_pages: 0` **without `source_urls`** searches the disk index without
HTTP requests from that call. The independent crawler may still be active. Queries accept
1–400 characters and must contain letters or numbers.

For a known source:

```json
{"url": "https://doc.rust-lang.org/book/ch01-01-installation.html", "query": "installation"}
```

`query` is optional for `web_read` and focuses the excerpt. URLs are required to be
absolute public HTTP(S) addresses on ports 80 or 443, without embedded credentials.

Search results contain `backend: "local_fts5"`, `coverage`, `results` and `warnings`.
Each hit has `url`, `title`, a snippet of at most 350 characters, `indexed_at`,
optional `published_at` from explicit page metadata, and
`read_status`. Successful second-phase reads add `page` with a passage of at most
6,000 characters, URL, title, `retrieved_at`, SHA-256 `content_hash`, language,
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

## Focused source retrieval and our API

| Argument | Behavior |
| --- | --- |
| `domains` | Up to eight domain filters, including genuine subdomains. This filters our corpus; it never queries a platform search API. |
| `days` | 1–365 days of confirmed publication metadata. Excludes unknown, future and older dates; retrieval time is never substituted. |
| `max_per_domain` | 1–10 results per host/domain group, default 2, to diversify evidence. Content hashes also remove exact duplicate bodies. |
| `discover` | Agent default: `true`; Rust library and HTTP endpoint default: `false` for compatibility. First locate and verify strong indexed sources. If insufficient, read up to four entry pages and follow up to 16 observed links (eight per origin, four waves). Date-window and explicit recency queries still discover new URLs. |
| `fresh` | Default `true`: revalidate explicit sources and selected evidence over HTTP. Navigation may use its TTL cache as a locator; that cache is not reported as fresh evidence. A source checked during the same call can be reused. Set `false` to allow cached evidence. Agent `web_read` also defaults to fresh; Rust `read()` retains cached behavior for compatibility. |
| `source_urls` | Up to five **known original public URLs** to fetch/index before searching. Reports success, discovered links or failure in `source_reads`. Does not invent URLs; with discovery enabled, it follows relevant links from these sources. |

For example, an agent can use:

```json
{"query":"ownership", "domains":["doc.rust-lang.org"], "source_urls":["https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"], "read_pages":0}
```

This performs source ingestion over HTTP, followed by our own FTS5/BM25 query.
Later index-only searches require no network. Original page reads, source ingestion
and crawler traffic use the existing robots, DNS, byte and concurrency policies.
No personal browser sessions are ingested. Common English/Spanish filler words
can be removed without translation or model calls; broad term matching is disclosed.
Date-only publication metadata is represented as UTC midnight; no precise local
publication time is inferred. Candidate scanning is bounded to 1,000 ranked rows per pass and reports truncation.
This can omit qualifying pages beyond that sample. Date/domain filters are rechecked
after reading pages in case they redirect or their publication metadata changes.

`POST /api/web/search` exposes these arguments behind the same tenant and origin
boundary as the rest of Fénix. It defaults `read_pages` to **0**; agent tool calls
retain **3**. Browser search needs no running browser profile. Opening a result
uses the selected local Chromium profile. The interface separates publication and
indexing dates and displays actual index coverage.

There are no provider adapters, configurable search-provider endpoints or search
credentials. Known external search-service URLs are rejected before fetching or
adding to the frontier. This guard is supplementary, not an exhaustive registry
of every internet service. Sources themselves remain third-party websites that
Fénix reads directly; neither searching nor rendering is outsourced. The agent
must not bypass this architecture through shell scripts or another tool.

Filters and source reads cannot create missing coverage: a blocked site, absent
publication metadata, exhausted crawl budget or unindexed subject remains a stated
limitation. RSS items enqueue source links; the worker fetches them later under its
normal budget. This does not promise immediate exhaustive community search.

## Concurrent searches and downloads

`max_concurrent_searches` defaults to **10**, with a validated range of 1–10.
Searches and direct `web_read` calls share that foreground admission limit across
all agents. An eleventh call waits for capacity, with its own 30-second admission
timeout. `queue_wait_ms` reports that wait; it does not consume the subsequent
source-reading budget (100 seconds with discovery/source URLs, otherwise 35 seconds). Cancelling a queued or active call releases its
slots. Old configuration files without this field automatically use ten.

The HTTP pool has the same global cap, including robots, redirects and the
crawler. Different origins download in parallel. Each origin has one active
request and a persisted minimum interval; waiting on an origin does not hold a
global download permit. The crawler begins new work only when no foreground
operations are active or queued; an already-started crawl may finish.
The crawler also stops starting work at its share of the daily request budget.
`foreground_reserve_percent` defaults to 20 (0–90), leaving a best-effort share for
explicit user/agent reads. An in-flight crawl can spend extra requests on robots
and redirects, so this is not a strict reservation. Tiny budgets allow at least
one crawl attempt. Foreground operations retain the total daily limit. Status
exposes `crawler_request_budget` and `requests_per_day`.

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
| `max_urls_per_origin` | 2,000 | Limit one origin's share of the frontier. |
| `max_body_bytes` | 2,097,152 | Maximum decompressed page download (2 MiB). |
| `max_text_chars` | 64,000 | Maximum stored text per page. |
| `request_interval_ms` | 2,000 | Minimum delay per origin; also the active crawl-loop pause. |
| `requests_per_day` | 2,000 | Shared HTTP attempt budget per UTC day, including robots and redirects. |
| `foreground_reserve_percent` | 20 | Stop starting background crawls before the total budget is exhausted; accepts 0–90. In-flight requests can reduce the reserved share. |
| `refresh_hours` | 72 | Next background refresh after a successful crawl. |
| `cache_minutes` | 15 | TTL for `fresh: false` and the legacy Rust `read()` method; fresh agent reads bypass it. |

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
  to 12 seconds after origin waiting. A full fetch allows up to 100 seconds
  including robots pacing and redirects. Discovery/source ingestion and result
  reads share a 100-second search budget; index searches without either keep
  their 35-second reading budget. Direct reads allow 100 seconds. Foreground admission can separately
  wait up to 30 seconds before either reading budget starts. Task cancellation drops
  pending network work. A running bounded parser/SQLite blocking task may finish.
- Persist retry backoff and daily budgets across restarts. Reaching the traffic
  budget leaves index-only search and fresh cached reads available. A leased URL
  interrupted during crawling becomes eligible again after one minute.

HTTP extraction supports HTML/XHTML, plain text, XML sitemaps and RSS/Atom link discovery. It does not
execute JavaScript, compute external CSS visibility, solve CAPTCHAs, sign in,
extract PDFs, submit forms, make purchases, reserve flights, inspect screenshots,
test rendered sites or change viewport size. Interactive browsing and responsive
testing use the separate [local browser](browser.md).

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

[Current independent-search verification](independent-search-audit.md).

## Query-directed discovery and recent research

The agent now enables bounded discovery even for a query-only call. Domain filters
bootstrap their public roots even when an older owner configuration lacks community
seeds. Without domains or explicit source URLs, recognized source names select official
roots from a small bootstrap catalog, otherwise up to four configured seeds are
visited. This catalog does not cover all topics and never supplies answer URLs. Original HTML anchor
labels and RSS/Atom titles rank observed links on disk, without indexing feed excerpts
as articles. There are at most four follow-up layers, 16 URLs total and eight per origin. Robots-advertised
sitemaps can be traversed even when their URLs do not contain query terms. Each
document contributes at most 2,000 observed links, within configured frontier limits.
Observed static navigation/contents pages can bridge a listing to an original.
`discovery` reports entrypoints, attempted sources, candidate/follow counts and domains
omitted by the four-entrypoint cap. `failure_kind` distinguishes blocked, timed-out
and unavailable reads; unvisited sources have not been searched. No provider or
platform search API is used.

The bundled `web-research` skill (1.5.0) is our own Last 30 Days adaptation. It plans
concise queries, selects relevant communities, separates publication from retrieval,
reads originals and discloses missing sources. HN timestamps are tied to the requested
permalink so recent comments cannot make old submissions qualify as recent. The
additive SQLite v3 migration preserves the existing corpus and stores bounded link
labels in the frontier. Bounded link-origin edges isolate discovery from
unrelated URLs inserted by concurrent queries. Known bundled skills upgrade; owner customizations are preserved.

This remains a bounded crawler and lexical search engine, not worldwide search.
Reddit access depends on its public policy; a blocked source is not a zero-result search.
See [the implementation audit](research-discovery-audit.md) for measured coverage.

## Retrieval correctness

HTML extraction preserves block boundaries and preformatted code, including
indentation inside highlighted spans. Passage selection favors windows containing
more distinct query terms. Markdown sources contribute observed links. Download,
parser, time, daily-request and ten-operation concurrency limits still apply.
Fresh result reads update snippets as well as passages and discard evidence that
no longer matches the query or requested scope. Failures remain explicit; neither
an HTTP success nor a lexical match proves that a question has been answered.

Defaults apply to newly created configurations; existing custom resource limits
are preserved. Raising a byte limit does not bypass robots or access restrictions.
The retained bounded text is a disk-backed search index, not an ephemeral-only
mode or a full documentation mirror. There is no semantic translation or global
search-provider fallback.

Discovery stops early on a source with strong lexical support or when it runs out
of relevant links/time; `discovery.stop_reason` reports why. This stopping rule
is a resource heuristic, not a confidence score. `source_reads` and
`discovery.sources` include fetch/check timestamps and network/cache/shared
provenance. Reusing a source read inside the same call does not make another
HTTP request, even with a zero-minute TTL. The agent's skill translates concise
query terms when the requested sources use a different language; the Rust engine
itself remains lexical.

Anchor labels pointing at different fragments of the same canonical URL are
merged rather than discarded. Observed link edges remain queryable when the
background frontier is full, without raising its cap; the provenance table is
bounded to four times the frontier cap. Link ranking favors whole path words.
When links and time remain, discovery allows at least two expansion waves before
stopping on strong lexical evidence, so an overview can lead to its reference.
See the [retrieval audit](search-retrieval-improvements.md) for measurements and
observed limitations.

## Index-first latency optimization

For ordinary queries, discovery first looks for substantial relevant pages in the
existing index and checks up to three directly, with a 15-second verification
phase inside the overall budget. If a checked page remains relevant,
`discovery.stop_reason` is `indexed_sources`; no entry-page traversal is needed.
Changed or failed evidence falls back to fresh discovery. Date filters and explicit
recency wording such as `latest`, `news`, `hoy` or `recientes` retain fresh listing
discovery so a known old article cannot prevent finding new URLs.

Other navigation can use the configured TTL cache. A cached navigation page that
becomes selected evidence is revalidated when `fresh: true`. Failed sources are
not requested again inside the same call, and failures never count as successful
current verification. The same-origin ranking preference only breaks ties; it
cannot make an unrelated link a discovery candidate.

At most 16 pending observed links per call can enter an additional **256-URL
priority queue** in the existing frontier. `discovery.background_queued` counts
newly prioritized links; `coverage.priority_urls` reports pending priority entries.
This is deferred work, not completed research. The existing `Web::run()` worker
processes it when foreground traffic and the daily reserve permit; library users
must still start that worker explicitly. No detached per-query crawler is spawned.
Retry delays and frontier/page caps remain in force. Successful foreground reads
also set the next crawl time, preventing immediate duplicate background downloads.

HTTP connections are reused through at most 32 per-origin clients, with one idle
connection per origin, a 30-second idle connection timeout and 60-second client
inactivity expiry checked on pool access. DNS addresses are checked before every
request; changes replace the pinned client and private addresses remain rejected.
Redirect, robots, credential and download policies are unchanged. Lexical ASCII
comparison avoids temporary strings, and bulk link writes reuse SQL statements
and per-origin counts within each transaction.

See the [before/after latency measurements](search-latency-optimization.md). A cold
query still needs discovery; a warm live query still depends on the source's
latency and pacing. An offline index lookup is a separate, explicitly marked case.
