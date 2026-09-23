# Recent and community research

Our own workflow, informed by a review of Last 30 Days. No upstream scripts, provider credentials, hosted search endpoints or private cookies are needed.

## Plan a small search

Separate the entity, the requested question and the time window. “What changed in X?” needs dated primary announcements; “What problems do users have with X?” also needs original discussions. Avoid feeding the whole conversational request into lexical search. Keep distinct product/version names, and include a comparison query when relevant.

Choose sources because they can answer the question: official documentation/releases for supported behavior; a relevant public community for experiences; a repository's original issue or discussion for a reported defect. Hacker News is useful for technical discussions, but not every topic belongs there. Consult actual discovered threads; do not invent IDs, communities or URLs.

Examples (independent queries may run together):

```json
{"query":"SQLite performance", "domains":["sqlite.org", "news.ycombinator.com"], "days":30, "discover":true, "read_pages":3}
```

```json
{"query":"ownership borrowing", "source_urls":["https://doc.rust-lang.org/book/"], "discover":true, "read_pages":2}
```

A successful call covers only selected roots and relevant observed links, not every historical thread. Feeds and link labels help find originals but are not claim evidence. If a relevant original listing/feed is known, use it as a starting point. If the platform denies access, report that limitation instead of interpreting it as no discussion.

## Dates and evidence

Use `days` for declared publication dates, independently of keyword queries. HN permalink dates refer to the submission/comment identified by that permalink; a newer comment does not make an old submission recent. Publication date, update date, event date and retrieval time are different. A recent discussion of an old event is evidence of recent discussion only. Unknown dates remain unknown.

Read the passage supporting each claim and retain the URL and date. Separate primary announcements from user anecdotes, repeated claims from independent experiences, and verified observations from inference. Do not aggregate votes across platforms or invent engagement scores. Report comment coverage and truncation honestly; a title is not a conversation.

Stop when the remaining uncertainty cannot be resolved by another concrete source or query. A useful answer can be partial if its limits are explicit; more repeated calls do not establish completeness.

## Español

Divide la pregunta en pocas consultas precisas; conserva nombres y versiones. Elige fuentes pertinentes y abre los originales. Usa fechas de publicación confirmadas para el período solicitado; no rejuvenezcas un artículo por comentarios recientes. Distingue anuncios, experiencias, inferencias y duplicados. Termina cuando no haya una siguiente consulta concreta que resuelva una duda material.
