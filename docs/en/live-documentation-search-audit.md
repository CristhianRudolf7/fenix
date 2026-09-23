# Live documentation search audit — 2026-09-20

## Result

The current engine can fetch public documentation at a known URL, but it does **not** implement reliable, always-live documentation discovery without a local corpus. This is an audit of the current source code, not an update of the running Fénix installation.

Ten live searches used the same `discover: true` default selected by agent dispatch, with fresh isolated data directories and the normal discovery seeds, byte limits, pacing and 15-minute cache. No background crawler or language model was started. Three cases ran concurrently; known URLs were supplied manually only to the assisted controls. This measures tool behavior, not a model's ability to choose useful domains, versions or browser fallback. A separate eleventh search disabled cache for comparison. All temporary corpora were deleted. The saved report contains metadata and content-presence flags, not documentation bodies.

[Machine-readable evidence](../benchmarks/live-documentation-search-2026-09-20.json) and [reproducible audit](../../crates/fenix-web/examples/live_documentation_audit.rs).

## Observed searches

| Input | Elapsed | Observed outcome |
| --- | ---: | --- |
| React/useEffect question, no domain or URL | 8.14 s | One unrelated result: gob.pe homepage |
| Next.js 16 proxy/middleware, no domain or URL | 8.10 s | One unrelated result: NASA homepage |
| Tokio timeout, no domain or URL | 8.05 s | One unrelated result: NASA homepage |
| useEffect/cleanup restricted to react.dev | 2.96 s | Homepage read, zero matching results |
| Proxy/middleware restricted to nextjs.org | 6.79 s | Relevant Next.js 16 release article plus conference page; not the requested API reference |
| API v2 context restricted to context7.com | 6.33 s | Generic guides and contact pages, neither containing `/v2/context` in the returned passage |
| Exact React useEffect URL | 2.24 s | Rejected by the download byte limit; zero results |
| Exact Next.js proxy API-reference URL | 7.55 s | Requested page rejected by byte limit; discovery found a smaller getting-started proxy page |
| Exact Tokio timeout URL | 9.54 s | Requested function documentation and related timeout_at documentation returned |
| Exact Context7 API guide URL | 6.30 s | Requested API guide returned, including `/v2/context`; another result concerned OpenAPI ingestion |

The source URLs are [React useEffect](https://react.dev/reference/react/useEffect), [Next.js proxy reference](https://nextjs.org/docs/app/api-reference/file-conventions/proxy), [Tokio timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) and [Context7 API guide](https://context7.com/docs/api-guide). They are controlled inputs, not URLs discovered by the unaided engine.

The full run, including repeated reads and the zero-cache control, took 41.77 seconds after compilation. Peak RSS was 35.07 MiB for the audit process (three independent Web instances concurrently), excluding compilation and any browser/model. This small sample is not a production-load benchmark. Low memory use did not establish search quality.

## Confirmed limitations

1. **Discovery starts from a small seed set.** With no domains or source URLs, it chooses at most four configured seeds using terms in their URL strings. There is no general library-name-to-official-documentation resolver. The live cases reached Wikipedia, gob.pe and NASA. Sparse exact lexical matches broaden to any meaningful term; one returned hit can therefore be unrelated. Domain selection helped Next.js but did not reliably discover React or Context7 API documentation. Following at most six links, two per origin, further limits traversal.
2. **A new tool call does not necessarily contact the source.** Two immediate reads each of the successful Tokio and Context7 sources returned `source: cache` with **zero new HTTP requests**. A controlled test changed the origin from `legacyMethod` to `currentMethod`; the next read still returned the old method without a request. Expiring the cached record allowed the new method to be read. This is intentional cache behavior, not an HTTP transport failure.
3. **Disabling cache is only a partial solution.** With `cache_minutes: 0`, each repeated Tokio read made one request and returned `revalidated`. It still created a local SQLite corpus (about 0.39 MiB including associated files in this control), and it did not improve source discovery. No per-call freshness or no-storage switch exists in the current search/read tool schemas.
4. **The default 512 KiB download cap excludes real documentation.** Both the exact React useEffect and Next.js proxy reference pages exceeded it. Direct retries failed the same way. A bounded text/Markdown alternative or smarter document handling is needed; merely retrying cannot fix this.
5. **HTML code formatting is lost.** A controlled `<pre><code>` Python example lost its newlines and indentation because extraction splits all text on whitespace. Returned snippets are not guaranteed to preserve executable examples. This test checks HTML extraction, not plain Markdown responses.
6. **Publication recency is different from documentation freshness.** `days: 30` removed all results for the successfully fetched Tokio docs because publication metadata was absent. A controlled newly fetched documentation page showed the same exclusion. This behavior correctly avoids inventing publication dates, but it cannot answer “give me currently maintained docs.”
7. **Version matching is not implemented as a documentation feature.** Search accepts query words and URLs, without a structured library/version input or project dependency resolution. A version-specific URL can constrain reading, but the tool itself does not verify that the returned API matches the project's installed version.
8. **Rendering and content selection remain limited.** The HTTP tool does not execute JavaScript or automatically invoke Chromium; the skill instructs the agent to choose the browser when needed. Text is capped at 24,000 characters per page and passages at 2,400 for search or 6,000 for direct reads. Passage selection uses the first matching query term, not section-aware documentation retrieval. Some returned pages were explicitly marked truncated.

## Recommended next change

Add an explicitly live documentation path that resolves official sources and relevant versions, retrieves/revalidates on every request, processes content transiently without populating the persistent corpus, preserves headings/code blocks and reports source URL, checked time, version and truncation. Prefer bounded official Markdown/llms.txt/sitemap paths where available, using browser rendering only for a concrete missing-text problem. Reject weakly related results rather than filling the result list with any matching word. These changes were **not** made during this audit.

## Reproduction

```sh
cargo run -p fenix-web --example live_documentation_audit -- docs/benchmarks/live-documentation-search-2026-09-20.json
cargo test -p fenix-web --lib live_docs_audit -- --nocapture
```

The two controlled characterization tests passed, confirming the cache, indentation and date-filter limitations above. Their passing is evidence of current behavior, not evidence that the requested always-live feature already exists. Network results can change as sites update or restrict access.
