# Community search audit — 2026-09-19

[Español](../es/community-search-audit.md)

**The adapted skill is installed and enabled; full Reddit/Hacker News search is not implemented.** This audit exercises the existing native engine directly, without model calls or changes to the agent, production configuration or deployed binary.

## Deployed state

`web-research` version 1.2.0 is enabled and matches the repository. Its independent-research reference is installed. It adapts source selection, dates, provenance and corroboration guidance from `mvanhorn/last30days-skill`; upstream connectors, scripts and search-provider services are not installed.

The live index contained zero Hacker News pages, zero Reddit pages and two GitHub pages. The deployed configuration contained no community seed URLs: the newer defaults did not overwrite the existing seed configuration. Read-only topic queries confirmed zero HN/Reddit hits and two GitHub hits for Rust. See the [production baseline](../benchmarks/community-search-production-2026-09-19.json).

## Isolated source and retrieval checks

Tests used a temporary empty index with background crawling off. Explicit public source URLs seed the test; thread URLs were discovered from the actual HN page by Fénix, not invented or taken from a search provider. These test pages do not populate the production corpus.

| Check | Observed result |
| --- | --- |
| HN RSS | Read successfully; 30 original links discovered. Feed excerpts are not indexed as articles. |
| HN front page | First sequential run timed out at 30.008 s. Paced run read/indexed it in 362 ms, excluding the deliberate wait. |
| Three discovered HN threads | All read/indexed and all 3/3 retrieved by title-based positive-control queries. This is not a general search accuracy score. |
| Last-30-days filter on those threads | 0/3 returned; all had unknown publication metadata in the current extractor. |
| HN topic coverage after ingestion | SQLite: 0; Rust: 1; Show HN: 2. Sparse corpus coverage, not full-site search. |
| Reddit r/rust and r/all RSS | Both denied by the site's robots policy; no posts indexed and no results. No alternate identities or access bypass attempted. |
| GitHub rust-lang/rust | Repository page read/indexed; Rust query recovered it. Not full GitHub search. |

HN's retrieved robots policy specifies `Crawl-delay: 30`. Fénix's current 30-second fetch timeout includes origin pacing, leaving little or no time for a request queued immediately after another HN fetch. A controlled repeat with 31 seconds between HN reads succeeded. The paced audit took 130.545 s including those waits. All six index-only topic queries made zero additional HTTP requests.

Sources and results: [first run](../benchmarks/community-search-2026-09-19.json), [paced run](../benchmarks/community-search-paced-2026-09-19.json). Records retain identifiers, queries, hashes, statuses and timing; page/comment excerpts are omitted.

## Remaining work identified, not implemented in this audit

- Populate and maintain actual community coverage in existing installations, without overwriting owner configuration.
- Separate polite origin waiting from the network fetch timeout, or defer reads explicitly.
- Extract publication dates from supported community page structures and retain their provenance.
- Keep Reddit marked unavailable to this public crawler unless a permitted access method becomes available. A skill cannot grant access a website denies.

Reproduce the paced source checks:

```sh
cargo run --locked -p fenix-web --example community_audit -- target/verification/community-search.json --paced
```
