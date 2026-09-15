# Initial application verification

[Español](../es/verification.md)

Verified on Linux during the initial implementation, 2026-09-12. No credentials
or personal runtime data are included in this document or the repository.

## Automated coverage

The final Rust run passed 75 unit/integration tests and two documentation examples;
all four Chromium scenarios passed. Rust formatting, Clippy with warnings denied,
frontend formatting/syntax, shell syntax and desktop-entry validation also passed.

The workspace suite covers the existing 17 provider adapters, default and exact
project credentials, five native/compatible protocol round trips, reasoning
metadata, error classification and HTTP limits. New application tests exercise:

- FIFO activation and separate context order, including a new request arriving
  between an earlier task's tool call and result.
- Mandatory approval, changed-argument rejection, denied effects, cancellation
  while waiting and clarification replies bypassing queued unrelated messages.
- Private configuration output, invalid credential setup, loop prevention and
  retention of original evidence after failure.
- Memory deduplication, accented lexical search, corrections, deletion and output
  recovery by pages.
- File path escapes, protected paths, non-overwriting creation, exact edits,
  commands outside a workspace, redacted output, process ownership and timeouts.
- Cron in Peru, reminder delivery, duplicate schedules, overlap protection,
  restart interruption and skipped offline executions.
- Manual skill preservation, versions and enabled states; compaction of complete
  tasks while retaining original history and tool pairs.
- Telegram pairing, owner isolation and text-only input; loopback host/origin and
  HTTP content-type checks.

The four Chromium scenarios use the actual Rust service and a local model fixture:
welcome/real-inference validation, chat with textual approval and FIFO, memory/
skills/schedule/activity panels, and persistent themes/mobile layout. They check
that first use is white even when the OS requests dark mode and that saved keys
are not returned to the browser. Screenshots were inspected in light desktop and
dark mobile layouts. Browser fixtures do not establish live Telegram connectivity.

## Real Luna requests

The authorized OpenAI key was located in a sibling project, read privately and
used for `gpt-5.6-luna` with `reasoning.effort: low`. A direct minimal Responses call
returned HTTP 200. The application's own setup then verified an inference, and
`scripts/smoke-live.py` completed these five scenarios against OpenAI:

| Scenario | Model calls | Executed tools | Result |
| --- | --- | --- | --- |
| Save explicit memory | 5 | `skill_load`, `memory_search`, `memory_manage` | Stored, deduplicated and read back with source metadata. |
| Retrieve memory | 2 | `memory_search` | Retrieved the expected fictitious preference. |
| Create and verify a file | 5 | `skill_load`, `files_list`, `file_write`, `file_read` | One textual approval; exact final file bytes verified outside the model. |
| Command outside the file workspace | 3 | `command_run`, `command_status` | One textual approval; expected output and exit code 0 verified. |
| Create a recurring reminder | 3 | `schedule_manage` | The persisted reminder, interval and next execution were checked. |

No tool errors occurred in those five runs. The report is generated at
`target/verification/live-smoke.json` and is intentionally ignored by Git. The
test's temporary database, key and files were removed after it finished. The
script remains opt-in; CI does not send paid model requests.

Live account access was checked **only for OpenAI Luna**. Other providers have
local protocol/authentication tests; their account permissions, credits and model
availability must be validated during setup with the owner's own credentials.
No Telegram bot token was supplied, so Telegram's remote connection and delivery
were not exercised. The Linux desktop installer is supplied with shell/static
checks; the implementation session did not install or enable a user service.

This is an initial functional application. It is not a benchmark score, a claim of
days-long autonomous reliability, an exhaustive security audit, or proof that
every model will make correct tool decisions.

## Streaming update

The final workspace run passed 83 Rust tests and documentation examples; the two
opt-in live tests are excluded from that default count. Formatting and Clippy
with warnings denied also passed.

The update adds an SSE parser test for fragmented UTF-8, CRLF, comments and
multiline events; HTTP tests preserve terminal tool arguments, usage and opaque
reasoning while excluding those fields from previews. Missing terminal events,
provider failure events, invalid JSON and size limits reject the inference.
A runtime scenario confirms that an incomplete preview is discarded without
retrying inference or adding it to conversation context.

Six browser scenarios passed, including reload during an active stream without
another inference or duplicate history, split-key redaction and textual
cancellation that removes the preview. These use the real Rust server and a
controlled local provider. An opt-in live OpenAI check with `gpt-5.6-luna` and
low reasoning also passed: visible streamed text matched the terminal response.
No live streaming checks were made against the other provider adapters; those
retain complete-response transport. Telegram keeps complete-message delivery.

The live streaming check is opt-in and billable:

```sh
cargo test -p fenix-providers --test streaming live_openai_preview_matches_the_completed_response --locked -- --ignored
```

It reads `OPENAI_API_KEY` from the environment, makes one inference request and
does not modify the assistant's conversation or print the key.

## Background subagents — 0.2.0

Verified on 2026-09-14: **102 Rust unit/integration tests and two documentation
examples passed**. Formatting, workspace Clippy with warnings denied, and the
server build passed. This update used local provider fixtures; it did not make
billable model requests or exercise live Telegram delivery.

All seven Chromium scenarios passed, including background status updates,
simultaneous textual requests, escaped report content and mobile layout.
Desktop light and mobile dark screenshots were visually reviewed. Frontend
syntax and formatting checks passed.

The new coverage checks that a master answers another request while a child is
still running or awaiting approval; isolated child context exposes skill metadata
and loads bodies on demand. Engine and storage checks reject recursive delegation,
preserve completed effects across inference retries, stop after three total
attempts, and prevent report turns from resetting that limit. Further cases cover
four-worker concurrency, queued/running cancellation, independent approval IDs,
hidden reports, migration from the existing database and restart interruption.

Cancellation tests verify process cleanup and effect-lock release, and wait for
filesystem work that outlives a cancelled async caller. Skill migration tests cover
personal-file preservation, symlink avoidance, resource limits and an explicitly
configured destination nested inside the legacy directory.

A separate temporary runtime started and shut down successfully with the default
global skill directory, installing the four templates in `~/.fenix/skills`.
Its temporary database was removed; existing installation data was not migrated
as part of this check. These checks validate orchestration behavior, not general
model task accuracy or long-duration autonomous performance.

## Projects and general conversation

Verified on 2026-09-14: **137 Rust unit/integration tests, two documentation
examples, and eight Chromium scenarios passed**. Workspace Clippy passed with
warnings denied; JavaScript syntax, web formatting and diff whitespace checks
passed. The two opt-in live-provider tests remained ignored.

The project update adds coverage for deterministic routing, model-routed compound
requests and fixed execution scopes. Storage cases exercise unique aliases,
arrival-time reply/focus hints, immutable bindings, version conflicts, scoped exact
IDs and lexical search, memory provenance, and isolation of neighboring history
messages. Migration tests retain existing records as general without guessing
historical project ownership.

The runtime scenarios use a controlled provider to ask for A's state and then work
on B, continue general conversation while a child runs, and resolve a late approval
without changing that child's project. Additional cases cover project schedules,
reports that preserve focus, deduplication by destination, and workspace fallback
captured from the parent. They verify orchestration and tool boundaries rather than
assuming the model will interpret every real request correctly.

A native Telegram reply fixture resolves the original approval by its bot/chat/
message reference and verifies that replying to an unrelated task does not approve
the only other pending action. Scoped reminder records retain the same references
without creating runnable work or appearing in another project's lexical history.

HTTP coverage checks project creation/update, duplicate aliases, workspace
validation, task bindings in the single visible history, and memory search/edit/
delete operations with matching scopes. Chromium exercises project administration,
separate general/Atlas/Beacon memories, project badges, reply references sent with
the next chat message, retained earlier conversation and mobile layout.

These additions use temporary databases, fake credentials and local model fixtures.
They introduce no live-model benchmark or live Telegram delivery claim. A separate
evaluation with real models is needed to measure routing accuracy, unnecessary
clarifications, wrong-scope reads/effects, task success, total tokens and latency.


## Editable planning and pub/sub — 2026-09-14

The planning implementation passed 153 Rust tests (including two documentation
tests), `cargo clippy --workspace --all-targets -- -D warnings`, JavaScript syntax
and formatting checks, and 10 Playwright browser tests. Providers were mocked;
these are functional checks, not live-model benchmarks.

Coverage includes chat during initial inference, natural-language steering using
scoped active-task references, persisted minor questions, plan revisions while a
worker is active, superseding an older editing question, dependency/generation
fences, retained effects, scoped predecessor evidence, pinned delivery destinations,
subscriber replay, SSE cursor reconnection, concurrent context ordering and the
mobile plan/revision view.

The final Rust run used an isolated snapshot of the planning changes because a
separate `fenix-web` crate was being introduced concurrently and its workspace
manifest temporarily had no target. It does not validate that independent change.
Browser tests used the real local HTTP service with a mocked provider; the plan
display test supplies a controlled plan fixture.

## Independent web search integration — 2026-09-14

The combined workspace with planning and `fenix-web` passed 168 Rust unit/HTTP
integration tests and two documentation tests. The web addition accounts for 14
module tests, two agent integration tests and one localhost status-route test.
All 10 Playwright scenarios passed with the real local Rust service and mocked
model transport. Browser fixtures disable background crawling explicitly.

The new tests cover the actual background worker discovering other domains and
sitemaps, lexical retrieval, second-phase HTTP reading, persistent cache and FTS
updates, robots/opt-out rules, public-address validation, redirected access,
decompressed byte limits, daily budgets, cancellation and honest empty results.
They also check that both agent roles receive the tools and a query-only call
returns coverage through the normal model/tool round trip. Formatting, JavaScript
syntax and Clippy with warnings denied passed.

An explicit live run of `fenix-web`'s `read_search` example fetched
`https://doc.rust-lang.org/book/ch01-01-installation.html`, indexed one article,
discovered 11 URLs, found `installation` and read its cached source. It made two
HTTP requests (robots and article), took about 2.19 seconds and reported a peak
RSS of 16,292 KiB (about 15.9 MiB) via Linux child-process resource usage. This was
an isolated unoptimized example on Rust 1.98.1, with one page and no model calls;
it is not a measurement of full Fénix, a populated corpus or sustained crawling.
These checks establish the local pipeline and one public HTTP connection, not
worldwide search coverage or general retrieval quality.

The initial debug link encountered a full filesystem. Only generated development
build artifacts were cleaned; verification then used `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0` and `CARGO_INCREMENTAL=0`. Runtime data was preserved.
Configuration and known limits are documented in [the web search guide](web-search.md).

## Ten simultaneous web searches — 2026-09-14

After adding bounded concurrent downloads and web-only model tool batches, the
workspace passed **178 Rust unit/HTTP integration tests**, **two documentation
tests** and **all 10 Playwright scenarios**. Formatting, frontend syntax, Clippy
with warnings denied and the local server build passed. This includes 22 web
module tests, four agent web integration tests and one web status-route test.

The added checks cover ten overlapping downloads, an eleventh operation waiting,
active/queued cancellation, per-origin spacing without blocking unrelated sites,
shared refreshes with caching disabled, cancellation takeover, smaller configured
caps, foreground priority and compatibility with older configurations. Agent
batches preserve output order, journal each result once, validate arguments and
honor prior denials while allowing independent calls to complete.

A second live run repeated 110 search calls across ten topics. The ten concurrent
queries with HTTP took 17.87 seconds together with no failed source reads, using
12 HTTP attempts for 15 reads through in-flight sharing. Peak benchmark process
RSS was 36.74 MiB. The corpus remained small and manually enriched; these numbers
exclude the model and full service. See [method, full data and comparison](web-search-concurrency.md).

## Interactive execution and on-demand tools

Verified on 2026-09-15: **122 tests in `fenix-agent` and `fenix-storage` passed**;
the opt-in external language-server check remained ignored. Clippy passed with
warnings denied for these two crates, with dependency linting excluded. Rust
formatting, JavaScript syntax, formatting of the changed web files and diff
whitespace checks passed. **All 11 existing Playwright scenarios passed**, including
persistence of execution limits through the real local server. Browser-module
development proceeding concurrently is outside this verification claim.

New checks cover real PTY/pipe input, EOF, terminal resize, process timeout and
cleanup, persistent sessions across turns, split-secret redaction, large-output
head/tail previews, cursor replay, saved log recovery and access from later
authorized master turns. A 41-response local provider fixture completes beyond
the previous 32-step limit. SQLite and runtime scenarios verify durable deduplicated
follow-ups, correction of stale actions, reuse of completed worker context and
preservation of retry counters. Discovery tests verify smaller initial schemas,
bounded cache refresh, Spanish queries and rejection of nested delegation.

These checks use local model fixtures and shell processes. They do not measure
live model accuracy, token-cost savings or multi-day service reliability. See
[execution behavior and limits](execution-harness.md).


## Local browser integration (2026-09-15)

The workspace suite passed 211 tests, with four opt-in/platform tests ignored.
The Chromium fixture was explicitly run with the sandbox enabled and passed
navigation, forms, same-origin iframe actions, responsive viewports, screenshots,
vault fill, tab control and on-disk session restoration in a new browser process.
HTTP checks passed for profile access, credential privacy and streamed downloads.
All 12 frontend tests passed; a final targeted run also verified manual typing and
clicking through the Browser view after the last UI changes. Formatting, skill
validation and workspace Clippy checks passed. These use local fixtures and mocked
model responses, not real university or WhatsApp accounts. See [browser.md](browser.md).

## Release 0.3.0 — 2026-09-15

The release passed 211 workspace unit/integration tests, two documentation
examples and all 12 web scenarios. Four platform-specific or opt-in tests remained
ignored. Workspace Clippy with warnings denied, Rust formatting, web syntax and
formatting, and the server build passed. Model requests used local fixtures.

The long active-context scenario now allows sufficient time for its multiple
large reads and summary requests on shared hosts. The browser UI scenario waits
for and checks the actual open response before checking the first screenshot,
respecting Chromium's existing cold-start allowance. Assertions for preserved
tool results, viewport resizing and owner interaction remain in place.
