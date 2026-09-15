# Ten simultaneous searches — implementation and live measurement

[Español](../es/web-search-concurrency.md)

Fénix now admits up to **10 concurrent searches or direct reads**, with a separate
pool of at most **10 HTTP downloads** shared by every agent and the crawler.
An eleventh operation waits for capacity. The previous global HTTP mutex has
been replaced by per-origin coordination, so one site's pauses do not block other
sites. The configuration field is `max_concurrent_searches`, default 10, range
1–10; existing files without this field automatically use 10.

Each search reads up to three sources concurrently. Simultaneous requests for
the same URL share one refresh, including with article caching disabled. Robots
loads are combined by origin. Cancellation releases capacity; waiting has a
30-second limit separate from the 35-second reading budget. The crawler yields
to foreground work before starting another URL. The agent executor dispatches
web-only tool batches concurrently and keeps model-facing results in call order.
See the [full contract](web-search.md#concurrent-searches-and-downloads).

## Live results on 2026-09-14

The same ten topics and eleven phases as the [earlier benchmark](web-search-benchmark.md)
were repeated: 110 search calls in one isolated process, without an LLM. The
default crawler ran for 60 seconds, indexing 10 pages from 10 origins and finding
883 URLs with 28 HTTP attempts. Five topics initially had hits. Explicitly reading
ten known source URLs added seven pages; three Wikipedia articles again exceeded
512 KiB. Query measurements used 17 pages from 11 origins, with the crawler stopped.
Seven topics had hits; these counts do not establish relevance or general coverage.

The dev build used Rust 1.98.1, no debug symbols, incremental compilation disabled
and two Tokio runtime threads. The 15-minute article cache was disabled only for
the `fresh_reads_*` phases. Validators, robots and site delays remained active;
HTTP 304 revalidation is included in those phases.

| Condition, ten queries | Total seconds | Peak RSS, MiB | CPU seconds | HTTP attempts | Failed source reads |
| --- | ---: | ---: | ---: | ---: | ---: |
| `empty_index` | 0.0701 | 12.15 | 0.01 | 0 | 0 |
| `default_corpus` | 0.0332 | 22.93 | 0.03 | 0 | 0 |
| `index_sequential` | 0.0303 | 25.21 | 0.02 | 0 | 0 |
| `index_parallel_4` | 0.0315 | 25.21 | 0.02 | 0 | 0 |
| `index_parallel_10` | 0.0301 | 25.93 | 0.03 | 0 | 0 |
| `cached_reads_sequential` | 0.0534 | 26.43 | 0.03 | 0 | 0 |
| `cached_reads_parallel_4` | 0.0381 | 26.46 | 0.04 | 0 | 0 |
| `cached_reads_parallel_10` | 0.0306 | 26.68 | 0.03 | 0 | 0 |
| `fresh_reads_sequential` | 26.3262 | 30.68 | 1.28 | 15 | 0 |
| `fresh_reads_parallel_4` | 15.8827 | 32.77 | 0.82 | 12 | 0 |
| `fresh_reads_parallel_10` | 17.8689 | 36.74 | 0.90 | 12 | 0 |

**The ten simultaneous queries with HTTP completed together in 17.87 seconds**,
compared with 35.01 seconds and one failed source read before the change. All 15
requested source reads completed in this run: two returned new bodies, ten were
conditionally revalidated and three shared another call's refresh. Only 12 HTTP
attempts were needed. The ten cached queries took 30.64 ms together, compared with
18.85 ms in the earlier run; this change does not establish faster local queries.

Four concurrent calls, measured in batches of 4+4+2, took 15.88 seconds and were
faster than ten in this sample. Increasing concurrency does not remove origin
delays or guarantee lower latency. The slowest query was climate at 17.87 seconds;
AI research took 14.99 seconds and health 0.17 seconds. Empty topics did no source
reading. The independent fixture tests verify actual overlapping downloads and
the cap; a single live before/after run cannot isolate the effect from network
variability, remote content or request ordering.

Process peak VmHWM was **37,624 KiB (36.74 MiB)**; this is the benchmark process,
excluding the model and the complete Fénix service. Index, WAL and configuration
occupied **2,014,026 bytes (1.92 MiB)**, with 1,105 known URLs. There were 79 HTTP
attempts overall. RSS was sampled every 20 ms and VmHWM every 100 ms; CPU comes
from `/proc/self/stat` at 10 ms resolution. Allocator retention and phase order
affect RSS comparisons. Transferred bytes and large-corpus behavior were not measured.

## Validation and reproduction

Deterministic HTTP fixtures check ten overlapping downloads, an eleventh queued
call, cancellation of active and queued calls, shared refreshes, takeover after
leader cancellation, per-origin spacing, an unrelated origin making progress,
parallel source reads under a smaller cap, and foreground priority over new
crawls. Mocked-model integration checks ten-call batches, ordered outputs,
single result journaling, argument validation and prior denials.

```sh
cargo test --locked -p fenix-web
cargo test --locked -p fenix-agent -p fenix-server --test web_search
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo build --locked -p fenix-web --example benchmark
target/debug/examples/benchmark target/verification/web-benchmark-parallel-10.json
```

The last command uses live public HTTP in a temporary corpus that is removed at
exit. [Complete data](../benchmarks/web-search-parallel-10-2026-09-14.json) preserves
every query's timing, hits, read status, explicit preparation URLs and metrics.
This measures concurrency and resource use; expanding useful search coverage
remains a separate concern.
