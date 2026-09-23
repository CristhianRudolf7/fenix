# Community evidence with independent search

Use this workflow for experiences, recent discussions, trends and comparisons. It is guidance inspired by community research, not an integration with upstream search services.

- Search the local corpus with `web_search`, optionally restricting `domains` such as `["news.ycombinator.com"]`. These are filters on our index, not remote platform queries. `days` excludes undated, future and older articles based on explicit publication metadata. Read known public source URLs with `web_read`; use our local browser for rendered pages when needed. Never route queries through external search engines or hosted search APIs.
- Select relevant communities rather than querying every platform. Hacker News stories and public repository discussions are possible original sources, not guaranteed searchable collections.
- Open original sources. Record publication, update and retrieval dates separately. An indexed date is not a publication date. For a time window, separate confirmed in-window evidence, older context and undated sources.
- Trace reposts and duplicate claims back to their origin. Popularity is not truth; votes from different platforms are not comparable. Never imply comments were read when only a story title or excerpt was retrieved.
- Keep disagreement visible and cite actual passages. Change a query only to resolve a material gap; stop when coverage is insufficient. Report blocked or missing sources without bypassing access restrictions.

## Español

Usa el buscador y navegador propios. Escoge fuentes pertinentes, conserva fechas y procedencia, distingue duplicados de corroboración independiente y comunica la cobertura real. Ninguna consulta se delega a proveedores externos.

## Examples

```json
{"query":"SQLite performance", "domains":["sqlite.org", "news.ycombinator.com"], "read_pages":0}
```

```json
{"query":"ownership", "source_urls":["https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"], "read_pages":1}
```

A source URL is a known starting point, not automatically discovered by the query. RSS/Atom feeds add original links to the crawl frontier; a feed is not a verified article. Recent discussions without usable publication metadata may not appear under `days`; explain that gap.
