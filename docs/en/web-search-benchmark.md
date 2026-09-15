# Ten-query search and concurrency benchmark — 2026-09-14

[Español](../es/web-search-benchmark.md)

This is the historical measurement **before concurrent downloads were implemented**. It used live public HTTP, no search provider and no model calls. Downloads were serialized, and ten concurrent uncached queries produced one failed source read. See the [new ten-search implementation and measurement](web-search-concurrency.md) for current behavior.

## Method

One isolated process, two Tokio threads, unoptimized dev build without debug symbols, Rust 1.98.1. Ten fixed English queries were repeated across eleven conditions: 110 search calls. The empty index was measured first, followed by 60 seconds of default seed crawling. Crawling was stopped during query measurements.

The crawler indexed 10 pages, discovered 883 URLs and attempted 29 HTTP requests during that minute. Only five topics returned hits. Ten known source URLs were then explicitly opened to exercise reading on a more useful corpus: seven succeeded, while three Wikipedia articles exceeded 512 KiB. **This manual enrichment does not demonstrate autonomous discovery of those topics.** The corpus reached 17 pages from 11 origins; seven topics returned hits.

Sequential calls, concurrent batches of 4+4+2 and ten simultaneous calls share one `Arc<Web>`. Index-only tests use `read_pages=0`; other tests read up to three results. The default cache lasts 15 minutes; the same index was reopened with `cache_minutes=0` to force HTTP. Robots caches, validators, per-origin delays and daily budgets remained active.

## Per-topic results

| Topic / query | Hits | Sequential cache, ms | Sequential HTTP, s | HTTP with 10 concurrent calls, s |
| --- | ---: | ---: | ---: | ---: |
| Astronomy: `solar system` | 5 | 4.88 | 0.574 | 35.008 * |
| Health: `malaria` | 1 | 2.27 | 0.175 | 0.125 |
| Climate: `climate change` | 5 | 5.55 | 19.921 | 25.533 |
| Economics: `poverty` | 2 | 3.03 | 4.017 | 17.272 |
| History: `Roman Empire` | 0 | 0.93 | 0.002 | 0.006 |
| Biology: `photosynthesis` | 0 | 1.07 | 0.003 | 0.019 |
| Programming: `installation Rust` | 3 | 3.05 | 2.234 | 25.438 |
| Web design: `CSS` | 2 | 2.41 | 3.964 | 17.202 |
| AI research: `attention transformer` | 1 | 2.19 | 0.063 | 14.995 |
| Mathematics: `Pythagorean theorem` | 0 | 0.99 | 0.001 | 0.017 |

\* Astronomy completed two source reads and exhausted the 35-second budget for another. The three empty topics performed no source reads; their low latency does not indicate useful retrieval. Hits include partial-term matches: the climate query also returned an AI paper because of lexical broadening. No labeled relevance evaluation was performed.

## Results per ten-query batch

| Condition | Total time, s | Peak RSS, MiB | CPU, s | HTTP attempts | Failed reads |
| --- | ---: | ---: | ---: | ---: | ---: |
| `empty_index` | 0.0080 | 12.57 | 0.00 | 0 | 0 |
| `default_corpus` | 0.0479 | 22.82 | 0.04 | 0 | 0 |
| `index_sequential` | 0.0184 | 27.91 | 0.02 | 0 | 0 |
| `index_parallel_4` | 0.0149 | 27.91 | 0.02 | 0 | 0 |
| `index_parallel_10` | 0.0137 | 28.32 | 0.02 | 0 | 0 |
| `cached_reads_sequential` | 0.0277 | 28.96 | 0.03 | 0 | 0 |
| `cached_reads_parallel_4` | 0.0191 | 29.15 | 0.04 | 0 | 0 |
| `cached_reads_parallel_10` | 0.0189 | 29.29 | 0.03 | 0 | 0 |
| `fresh_reads_sequential` | 30.9567 | 32.86 | 1.10 | 15 | 0 |
| `fresh_reads_parallel_4` | 24.4349 | 36.78 | 0.99 | 15 | 0 |
| `fresh_reads_parallel_10` | 35.0081 | 38.85 | 1.22 | 15 | 1 |

## Interpretation and limits

- Ten cached queries took 18.85 ms together; an individual query waited up to 18.54 ms. Sequential individual queries were faster, but the complete batch took 27.73 ms. Higher concurrency does not imply lower individual latency.
- Without article caching, sequential queries took 30.96 s, four concurrent calls took 24.43 s, and ten took 35.01 s with one failed read among 15 requested reads. Calls accept concurrency, but a mutex shares one HTTP lane, SQLite access is serialized and only one HTML parser runs at once. Network/site delays and request ordering affect these measurements; one pass does not establish a stable speedup.
- Process peak RSS was 39,784 KiB (**38.85 MiB**). Index, WAL and configuration occupied 2,026,285 bytes (**1.93 MiB**) with 1,105 known URLs. There were 86 HTTP attempts overall.
- RSS was sampled every 20 ms per phase and process VmHWM every 100 ms. Allocator retention and phase ordering prevent attributing all RAM differences to concurrency. CPU comes from `/proc/self/stat` at 10 ms resolution; zero does not mean no CPU work.
- This excludes the model, full Fénix service, transferred bytes and a corpus of thousands of pages. Compilation is outside timings. The temporary corpus was removed on exit.
- For the implementation measured here, cached reads with at most four concurrent reading queries were a reasonable starting point from this sample. Request coalescing and concurrent downloads have since been implemented; source coverage and relevance remain separate concerns.

## Reproduce

```sh
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo build --locked -p fenix-web --example benchmark
target/debug/examples/benchmark target/verification/web-benchmark.json
```

[Complete data](../benchmarks/web-search-2026-09-14.json) includes explicit preparation URLs, hits, errors and individual timings. This is an opt-in live-network test; future results can differ.
