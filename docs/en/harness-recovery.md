# Procedural skills and evidence-led recovery

[Español](../es/harness-recovery.md)

The runtime includes fourteen manually curated skills: the original five plus repository navigation, test-driven repair, environments/dependencies, process/service management, package build/install, SQL analysis, text parsing, Git recovery, and software implementation. These are general procedures, not benchmark answers or automatically learned skills. Installation preserves user-modified files.

`skill_search(query, limit=3)` searches enabled names, descriptions and complete bodies lexically, with higher weights for names and descriptions. Results contain metadata, not bodies. `skill_load(name)` returns the procedure and loads up to eight authorized tools declared in optional comma-separated `required_tools` metadata. Existing custom skills need no new metadata. Skills do not grant permissions. The master and workers can search the entire catalog and load bodies selectively.

A guessed, authorized tool that is not exposed returns `tool_schema_loaded`, its schema and `executed=false`. Only a subsequent valid call may execute. Invalid arguments return required/received fields, the schema and a placeholder example. Examples explain shape; they are not executable task answers.

## Processes and resource conflicts

Ordinary commands keep existing approval and conservative serialization. `command_run` accepts optional `read_paths`: existing absolute inputs or paths relative to cwd. This starts a fresh Rust helper before creating the application runtime. Linux Landlock ABI 3 enforces read-only filesystem access; seccomp restricts syscalls, including network, metadata mutation and process manipulation. Standard libraries/configuration are implicit read inputs. No automatic fallback to unrestricted execution occurs. Kernels without the required support return an error. This deliberately restricted mode may not support arbitrary software, writable caches, temporary files or interactive programs.

Read-only input paths are registered for the lifetime of the process. Independent `file_write` and `file_edit` operations may proceed; overlapping input/output paths wait. Broad directory inputs reduce concurrency. Multi-file patches and unknown effects remain conservative. Inputs are live reads, not snapshots; external processes outside Fénix are not controlled. Read-only commands cannot be persistent or TTY sessions. Task completion still waits for outstanding work. The normal service mode remains available through explicit `persistent=true` and returns its descendant-lifetime policy.

## Context and recovery

Compaction checkpoints the redacted original before changes. It first tries deterministic masking of older successful tool outputs, retaining call/result pairs, exact recent batches, source roles and a runtime contract. Full outputs remain retrievable. This path makes no summary-model call. If it does not fit, the existing summary/fallback path remains; generated summaries carry the assistant role and are explicitly unverified. Exact contract observations override narrative claims. This reduces, but does not guarantee elimination of, semantic loss.

Repeated unchanged observations trigger a bounded diagnostic opportunity with process/tool/checkpoint recovery paths. Existing caps remain three recoveries per reason and eight total. Truncated responses never execute. Large action payloads receive edit-splitting guidance; other output exhaustion receives a concrete probe-and-test strategy. That distinction is a payload heuristic, not a measured reasoning-token classifier.

Planning, revision, background workers and independent plan review reuse existing machinery. The system guidance asks for reviews when local checks leave material uncertainty. It does not create an extra reviewer for every tool call, add model switching, or enable nested delegation.

## Evaluation

Use `scripts/benchmarks/harbor/experiment.py` with `--root`, `--job`, `--tasks` (five comma-separated names), `--dataset`, `--bundle`, `--config` and `--harbor`. It freezes binary/configuration hashes, runs the native evaluator in official containers, and produces `results.json` from official grades plus recorder billing. The recording proxy must already be running. `--summarize --root PATH` refreshes completed results without launching work. Missing charges remain unknown.

Compare fixed main and candidate binaries on the same tasks and settings. Previously inspected tasks are development regressions. Confirmation uses two five-task unseen cohorts with two independent attempts per version. Record failures and infrastructure exclusions; do not treat assistant completion as a passing grade. The research branch is reviewed separately from main.

Background workers snapshot model availability/connections before asynchronous work. They never hold a model read lock while waiting for configuration, which prevents an idle worker from deadlocking a model change. Web view loads similarly discard stale responses after navigation.

Validation on September 15, 2026: 239 Rust tests passed, four environment-dependent tests remained ignored; all 13 browser tests passed, as did Clippy, formatting and documentation tests. Browser validation used an alternate `TMPDIR` because the host `/tmp` volume was full. These are regression checks, not evidence of a Terminal-Bench score improvement.

Task context highlights up to two lexical skill candidates without loading their bodies. Suggestions are optional and may be irrelevant; the complete catalog remains available. Shipped repair and Git procedures update only when their prior content matches exactly, preserving owner edits and disabled state.

Long tool receipts preserve head/tail evidence and execution metadata instead of retaining only a prefix. Complete redacted output remains in the task journal; a preview never proves that the omitted middle is irrelevant. Context estimation counts the larger of normalized content and provider replay, plus an envelope allowance, without duplicating visible text or discarding hidden reasoning. The estimate is not a model-specific tokenizer.
