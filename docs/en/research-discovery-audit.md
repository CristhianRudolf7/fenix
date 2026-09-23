# Search and Last 30 Days adaptation audit

Date: 2026-09-19. Scope: native public search, agent dispatch, research skill and evidence delivery. No browser interaction logic was changed for this audit.

## Finding and implementation

The existing `web-research` 1.2.0 already contained a partial methodological adaptation, but a query-only tool call principally searched the local corpus. It did not discover sources for the query. Date-filtered Hacker News searches missed submissions because their publication timestamps were not extracted. A 30-second fetch deadline also included origin pacing, causing valid reads to expire while waiting.

The agent now defaults `web_search` to bounded discovery. A caller can still request an offline lookup explicitly; Rust library and HTTP endpoint defaults remain compatible. Selected domains bootstrap public roots even with an older seed configuration. The engine reads at most four entry pages (or up to five explicit sources), follows at most six observed links over two waves, with at most two follow-up URLs per origin, and then searches SQLite FTS5. Link labels and feed titles rank leads on disk; feed excerpts never become article evidence. Observed documentation/contents pages can bridge to original articles, including a same-origin no-JavaScript contents frame. Scripts are not downloaded or executed.

Publication extraction associates an HN age timestamp with the requested same-origin permalink. Other comments cannot date the submission. Unknown publication dates remain excluded from `days`. The 100-second total budget accommodates origin pacing separately from each HTTP request's 12-second timeout. Admission still has its own 30-second limit and all operations share the maximum of ten; cancellation drops pending work.

Search reports distinguish attempted sources, failures, candidate/follow counts and unvisited requested domains. HTTP 429 is an access/rate restriction, not a successful search with zero matches. When model context limits truncate a search result, a structured receipt preserves result/failure counts and instructs the agent to retrieve the full saved evidence before citing it.

SQLite migration v3 adds bounded frontier labels without rebuilding the corpus. Bundled skill 1.3.0 upgrades recognized unmodified versions using legacy hashes and installs a new reference filename, preserving owner-customized files. System instructions require actual search calls for internet-search requests and explain live discovery, dated evidence, source restrictions and receipts.

## Skill review

Reviewed the [official Last 30 Days skill](https://github.com/mvanhorn/last30days-skill/blob/beb7ed1868f034f198842174bfe2694e44b78363/skills/last30days/SKILL.md), commit `beb7ed1868f034f198842174bfe2694e44b78363`; downloaded SKILL.md SHA-256 `bdde67d5df7fd7f73dbfff4ee7f9fea919babedd5a5b17690e6d216fc1769bc6`.

Retained as our own written methodology: concise query decomposition, entity disambiguation, pertinent communities, separate time windows, original passages, deduplication, disagreement and explicit access/coverage limits. Did not import its Python runtime, Algolia/hosted-search adapters, provider credentials, private cookies, installation wizard or forced output formatting. Our implementation remains Rust and directly reads original public sources. The local skill is `resources/skills/web-research/SKILL.md`, with `references/recent-research.md`.

## Live evidence

Reproducible command (network opt-in):

```sh
cargo run --locked -p fenix-web --example research_discovery_audit -- docs/benchmarks/research-discovery-2026-09-19.json
```

The example uses a temporary empty corpus, no background crawler, no browser, no model and no external search service. Four queries run simultaneously; the known Rust book URL is a starting listing, not a preselected answer page. A second phase performs explicit offline lookups. Source passages are omitted from the saved artifact while URLs, dates, statuses and passage lengths remain.

| Query / source | Observed result | Time |
| --- | --- | --- |
| Ownership/borrowing, Rust book | Discovered the static contents page and then the original references/borrowing chapter; one matching result | 6.18 s |
| Transactions, SQLite | Discovered the documentation overview; one matching overview, not a full transaction tutorial | 4.24 s |
| Show HN, Hacker News, last 30 days | Front page readable; Show listing returned HTTP 429; no verified recent thread in this run | 32.28 s |
| Rust, Reddit, last 30 days | Denied by robots policy; no thread read | 0.075 s |

Wall time: **32.28 seconds for the four concurrent queries**, 11 HTTP attempts including robots. Isolated search-process RSS increased from 11,328 KiB to 19,448 KiB (peak about **19 MiB**). This is not total Fénix/browser memory, an upper bound, or a worldwide search benchmark. Offline repeats took 0.80–1.84 ms each and made zero HTTP requests. No active operations/downloads remained afterward.

The [final report](../benchmarks/research-discovery-2026-09-19.json) records these results. The [initial report](../benchmarks/research-discovery-initial-2026-09-19.json) is retained: all four queries initially returned no matching evidence. That run exposed missing navigation traversal and short-acronym URL false positives, which were fixed and regression-tested. Do not interpret this small sample as search recall or accuracy across arbitrary topics.

A separate [HN positive control](../benchmarks/hacker-news-control-2026-09-19.json) searched `open source`, a topic observed on the live front page, from another empty corpus. It followed the actual submission link, read the original thread and returned it under `days:30` with publication `2026-09-19T10:46:58Z`. Duration was 32.54 seconds including the site's pacing, three HTTP attempts (robots, listing, thread), and zero HTTP attempts for the offline repeat. The returned passage was truncated to 2,400 characters, so this does not establish full comment coverage or topic-wide recall. It confirms that the HN traversal and publication filter work when that endpoint permits reading; the earlier Show-listing 429 remains a separate recorded failure.

To run a focused audit, pass a second argument containing a JSON array of 1–10 `SearchRequest` values; the same URL, robots and resource policies apply.

## Verification and limits

Passing targeted suites: 43 native web tests, four agent search integration tests, 14 skill tests, four tool-receipt tests and two HTTP search integration tests. The migration test was rerun after adding an old-frontier schema check. Skill YAML validation, web/agent Clippy with warnings denied, and the `fenix` executable build passed.

```sh
cargo test --locked -p fenix-web --lib
cargo test --locked -p fenix-agent --test web_search
cargo test --locked -p fenix-agent --lib skills::
cargo test --locked -p fenix-agent --lib tool_receipt::tests
cargo test --locked -p fenix-server --test web_search
cargo clippy --locked -p fenix-web --all-targets -- -D warnings
cargo clippy --locked -p fenix-agent --lib -- -D warnings
cargo build --locked -p fenix-server --bin fenix
```

Automated checks cover cold-corpus discovery; feed-title leads; source/domain/date filters; persisted link labels; nofollow/private-target rejection; static contents traversal; HN dates and recent-comment exclusion; blocked and unvisited sources; bounded traversal; ten concurrent searches sharing downloads; cancellation; and an actual wait longer than the previous 30-second deadline. Agent checks cover tool discovery, query-only live defaults, explicit offline overrides, ten-call batches, validation/denials, skill retrieval and compact evidence receipts. Mock model responses validate dispatch/protocol behavior, not the quality of a live model's research decisions.

This remains bounded crawling and lexical retrieval. Source roots, outgoing links, body/link limits, publication metadata, robots policies and site rate limits constrain coverage. A query-only call is not a universal search engine; the agent must select pertinent domains or known listings and report unresolved gaps. Reddit/HN access is not guaranteed, and no denied source is bypassed. Persistent personal browser sessions remain separate from the public corpus.

The updated executable is `target/debug/fenix`. Restart the application using that
build and the same `--data-dir` / `--skills-dir` options as your existing instance.
An already-running process does not load new compiled instructions automatically.
