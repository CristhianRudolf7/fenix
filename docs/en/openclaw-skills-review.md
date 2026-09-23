# OpenClaw catalog review and Fénix improvements

> Historical 164-skill review. The current catalog has 167 entries after the [ten-repository review](ten-repository-skills-review.md); the evidence below belongs to this earlier iteration.

[Español](../es/openclaw-skills-review.md) · [Candidate decisions](../skills/openclaw-review-matrix.md) · [Evidence](../skills/openclaw-review.json) · [Catalog inventory](../skills/openclaw-catalog-inventory.json)

## Outcome

Fénix now has **164 skills**, up from 156. Eight distinct workflows were added, six existing entries improved, and four original executable helpers plus six references supplied. The changes address specific gaps found by comparing OpenClaw candidates against the actual local library. They do not install OpenClaw, copy its runtime, or create new provider connections.

The strongest demonstrated improvements are local feed comparison, calendar ambiguity detection, exact statement arithmetic and subtitle timing checks. The broader instructions improve available workflow coverage; their effect on live-model task success and cost has not been measured. More skills alone do not establish a better assistant.

## What was examined

The [VoltAgent catalog snapshot](https://github.com/VoltAgent/awesome-openclaw-skills/tree/4367aac4da2a6202b56cf6dfdbab91076bbb8a1d) was retrieved on 2026-09-19. Its 30 category files contain **5,196 parseable listing rows, representing 5,193 distinct URLs**, plus one malformed bullet. These are listing counts, not counts of independent capabilities. Category labels are imperfect: accounting appears under speech, and many gaming entries are agent social platforms.

All category listings were inventoried. Candidates were selected for a missing procedure, a concrete weakness in an existing entry, or a proposed mechanism already covered by Fénix. **Twenty-seven complete entrypoint bodies were read; one additional selected download returned 404.** The report does not claim to have read all 5,193 skill bodies or audited their linked executables. The [matrix](../skills/openclaw-review-matrix.md) records every selected candidate and the category-level triage separately.

Bodies came from the catalog's linked `clawskills.sh/skills-markdown/…` downloads. The evidence records retrieval time, URL, byte count and SHA-256. This is a mirror snapshot, not a verified upstream Git commit. A lookup of the historical `openclaw/skills` GitHub archive returned 404 in this environment; it was not used as source provenance. The catalog is MIT-licensed, but that does not establish the license or safety of every linked package. No upstream body, helper, installer or asset was vendored or executed.

## New workflows and their boundaries

| New skill | Concrete value | Available implementation and prerequisites |
| --- | --- | --- |
| [feed-monitoring](../../resources/skills/feed-monitoring/SKILL.md) | Detect late entries and edits by identity/content; separate unread, selected and delivered state | Original local RSS 2.0/Atom comparison helper. Fetching and recurring scheduling use existing tools; no new subscription daemon. |
| [web-data-extraction](../../resources/skills/web-data-extraction/SKILL.md) | Schema-first records, per-row sources, bounded pagination, missing values and repeated-page detection | Procedure for native web/browser tools or supplied exports. No hosted scraping/search replacement or universal scraper. |
| [desktop-automation](../../resources/skills/desktop-automation/SKILL.md) | Fresh window/focus evidence, accessibility targets and saved-artifact verification | Requires an existing OS-compatible GUI adapter and graphical session; no desktop driver installed. |
| [smart-home-control](../../resources/skills/smart-home-control/SKILL.md) | Exact device identity, capability units and read-back after a command | Requires an authorized controller. No live device was operated. |
| [audio-production](../../resources/skills/audio-production/SKILL.md) | Available TTS syntax, measured duration, chunk joins, explicit streams and final loudness checks | Uses available synthesis/processing tools. FFmpeg processing was exercised; speech synthesis and listening quality were not. |
| [weather-briefing](../../resources/skills/weather-briefing/SKILL.md) | Resolved location/timezone, observation versus forecast, units and official-alert validity | Requires current weather sources. No provider account or aviation briefing system added. |
| [game-development](../../resources/skills/game-development/SKILL.md) | Simulation timing, input/collision handling, pause/restart ownership and a playable-loop check | Uses the requested engine. No engine, game template suite or publishing integration installed. |
| [wearable-data-analysis](../../resources/skills/wearable-data-analysis/SKILL.md) | Session/date identity, unit conversion, duplicate-device handling and coverage denominators | Uses supplied exports or an existing connection. No diagnosis, credential archive import or continuous health sync. |

These outcomes differ from existing generic skills: extracting a dataset needs coverage and record provenance; a research answer needs claim support. Feed monitoring needs persistent item identity and delivery semantics. Desktop actions need focus and OS/session evidence that browser actions do not provide. Physical device commands need actual capability and state checks. This supports separate discovery entries without a vendor-specific skill for every CLI.

## Improvements to existing skills

- **calendar-scheduling 1.1.0:** handles repeated/nonexistent local times, floating/all-day/recurring calendar semantics and CalDAV synchronization scope. `local_time_candidates.py` returns all valid UTC interpretations without choosing a fold. Sync can push pending writes; reading availability is not a reason to publish unrelated changes.
- **learning-and-tutoring 1.1.0:** adds source-grounded flashcards and retrieval practice, note-versus-card identity, duplicate handling and preservation of existing Anki scheduling history. An import file is distinguished from a completed live import.
- **audio-transcription 1.1.0:** adds long-recording manifests, global timestamp offsets, chunk-boundary review and an SRT checker. Chunking does not guarantee no out-of-memory errors or perfectly seamless text.
- **invoice-and-expense-records 1.1.0:** adds normalized statement reconciliation, pending versus posted rows, partial/grouped payments and transfer handling. A Decimal-based helper checks account/currency balances without categorizing or paying.
- **skill-source-review 1.1.0:** distinguishes an index or scanner badge from source evidence, tracks mirror/revision provenance and makes missing helper review explicit. Its Spanish discovery description was improved after a missed query.
- **scheduled-workflows 1.1.0:** adds natural recurring-task and overlap aliases after the paired test exposed a Spanish retrieval miss. Its underlying scheduling procedure was already appropriate.

Six exact pre-change entrypoints are retained as fixtures and registered by hash. The native installation tests verify upgrades of known originals, preservation of personalized files and disabled state, exact resource loading and denial of resources for disabled skills. Existing user references are not overwritten automatically.

## Specific upstream decisions

The [feed-digest entrypoint](https://clawskills.sh/skills-markdown/odysseus0/feed-digest.md) marks triaged entries read. Our version keeps selection, reading and confirmed delivery distinct, preventing a failed digest delivery from consuming pending entries. The RSS helper also handles a late publication whose timestamp is older than the previous newest post; a time-only cursor would miss it.

The [CalDAV entrypoint](https://clawskills.sh/skills-markdown/asleep123/caldav-calendar.md) prescribes synchronization before queries and offers a remote-wins conflict configuration. Our guide first checks pending changes and cache freshness. Its local-time helper uses actual IANA transitions, including half-hour folds and skipped civil days, rather than assuming every transition is one hour. [Python zoneinfo](https://docs.python.org/3/library/zoneinfo.html) provides the timezone data and fold semantics.

The [Linux desktop entry](https://clawskills.sh/skills-markdown/ouyangabel/linux-desktop.md) relies on X11 utilities and delays; the [desktop guardian](https://clawskills.sh/skills-markdown/s3rous/desktop-guardian.md) adds persistent monitoring and automatic cleanup. Fénix's workflow verifies adapter/session compatibility and fresh target state, without adding an unsolicited background guardian or closing unrelated applications.

Smart-home instructions retain the useful capability and target checks from the candidates. An authoritative distinction is made explicit: Home Assistant's state endpoint updates a representation, whereas actual control uses services. A successful HTTP response is not physical verification. [Home Assistant REST API](https://developers.home-assistant.io/docs/api/rest/).

The [audio generator](https://clawskills.sh/skills-markdown/udiedrichsen/audio-gen.md) assumes a configured service, a personal script path and fixed speech pacing. Our version measures an actual sample and final file, checks the selected engine's control syntax, and respects the owner's existing authorization. The [Whisper wrapper entry](https://clawskills.sh/skills-markdown/neal-collab/auto-whisper-safe.md) makes broad RAM and seamless-merge promises that were not adopted.

The [Rust/Godot game entry](https://clawskills.sh/skills-markdown/kjaylee/game-dev-rust-godot.md) embeds another owner's framework restrictions, private asset locations and deployment target. Our version preserves the actual user's engine and scope. Playability, frame timing and restart behavior are useful checks; a mandatory asset collection or fixed screenshot count is not necessary for every game task.

Memory and self-improvement candidates were also read. Fénix already has scoped, evidence-backed memory, persisted task history, plans and agent-workflow diagnosis. A second WAL, a parallel learning database, compulsory writes before every reply, arbitrary promotion thresholds or a monitor that reruns effects would duplicate or conflict with that infrastructure. This review did not introduce those mechanisms or an automatic policy-rewriting loop.

## Evidence and limits

The [54 development scenarios](../skills/openclaw-expansion-cases.json) were recorded before entrypoint edits: 32 for new workflows, ten for initially revised existing skills and twelve neighboring tasks. Baseline and candidate use the same current runtime; baseline disables the eight new skills and restores the six previous parents. This is not a historical full-application binary comparison or a held-out model evaluation.

| Group | Baseline top 1 / top 3 | Final candidate top 1 / top 3 |
| --- | ---: | ---: |
| 32 new-workflow queries | Unavailable by construction | 31 / 32 |
| 10 revised-skill queries | 3 / 4 | 8 / 10 |
| 12 neighboring queries | 11 / 11 | 12 / 12 |

All **54/54** expected skills appear in the top three; **51/54** appear first. None of the eight new entries wins a neighboring query. The first run missed a source-review query; it also exposed an existing recurring-task miss. Natural descriptions/aliases were improved without changing the query corpus, then the suite was rerun. These are tuned development results, not estimates of real-world task accuracy. The earlier four-repository regression retains its separately documented parser-repair baseline miss.

Validation completed:

- **23 native Rust skill tests**, including previous regressions, the paired corpus, six-parent migration and resource activation checks.
- **23 Python helper tests:** feed edits/late arrivals/duplicate IDs/XML boundaries; DST gaps/folds/skipped days; exact monetary arithmetic/pending rows/currency mismatches; SRT indices, bounds and nested overlaps; CLI exit status and unchanged input files.
- A real FFmpeg smoke check joining one- and two-second generated tones, encoding MP3 and decoding it. Probe duration was 3.024 seconds including encoding overhead, mono at 48 kHz. This is processing evidence, not TTS, intelligibility or listening evidence.
- Format, native tool names, resource limits, bilingual catalog membership, migration hashes and local documentation links; Rust formatting and Clippy with warnings denied.

Reproduce native checks with `cargo test -p fenix-agent --lib skills:: --locked -- --nocapture` and `cargo clippy -p fenix-agent --lib --tests --locked -- -D warnings`. Run helpers with `PYTHONDONTWRITEBYTECODE=1 python3 tests/skills/test_openclaw_helpers.py`. The helpers need Python 3.9+ and IANA timezone data; they add no network dependencies. Exact versions, artifact hashes and paired results are in the evidence manifest.

The new entrypoints total **24,054 UTF-8 bytes**. Revised entrypoints grew from **10,632 to 12,600 bytes**; supporting resources total **23,472 bytes**. They load on demand. These sizes are not per-turn token measurements or proof of reduced cost.

## Activation and remaining work

Changes are integrated into Fénix's source and embedded skill catalog. They become available when an updated build starts; no running service was deployed or restarted in this task. The separate Codex skill installation was not changed.

No live GUI application, home device, wearable account, calendar connector, hosted scraper, TTS service or game engine was connected or exercised. Those prerequisites remain explicit. The review therefore adds tested local utilities and usable guidance without claiming unavailable integrations.

Further breadth should follow actual task demand and failures. The next evidence needed for general excellence is an independent set of end-to-end tasks scored on delivered artifacts, correct actions, completion rate and measured cost/latency. The catalog inventory provides a searchable starting point for future targeted reviews; it is not a recommendation to install thousands of entries.
