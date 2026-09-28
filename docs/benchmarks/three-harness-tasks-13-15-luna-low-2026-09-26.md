# Harness comparison: Terminal-Bench tasks 13–15

[Español](three-harness-tasks-13-15-luna-low-2026-09-26.es.md) · [Measured data](three-harness-tasks-13-15-luna-low-2026-09-26.json)

GPT-6 Luna low, standard service, no Flex. One attempt per combination. Results: fenix 3/3, codex 3/3, opencode 3/3.

Evaluated changes: linked compiler output/execution checks; concrete environment probes; reduced redundant discovery; fresh deployment evidence and execution-oriented skills. All three adapters received identical task-independent execution-target context. Local validation: 252 tests passed, three optional tests skipped, successful server build.

| Task | Harness | Checks | Time (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| 13. nginx-request-logging | fenix | 8/8 | 64.66 | 0.00994297 | 201,128 | 1,585 |
| 13. nginx-request-logging | codex | 8/8 | 49.03 | 0.00515203 | 85,830 | 1,795 |
| 13. nginx-request-logging | opencode | 8/8 | 58.96 | 0.00448368 | 103,985 | 1,618 |
| 14. openssl-selfsigned-cert | fenix | 6/6 | 35.57 | 0.00513034 | 87,690 | 1,209 |
| 14. openssl-selfsigned-cert | codex | 6/6 | 13.15 | 0.00318499 | 30,812 | 915 |
| 14. openssl-selfsigned-cert | opencode | 6/6 | 35.91 | 0.00320738 | 58,000 | 1,591 |
| 15. sqlite-db-truncate | fenix | 1/1 | 44.18 | ≥0.00658276 | 108,895 | 2,170 |
| 15. sqlite-db-truncate | codex | 1/1 | 26.24 | 0.00481085 | 69,088 | 1,957 |
| 15. sqlite-db-truncate | opencode | 1/1 | 59.91 | 0.00525387 | 101,696 | 3,916 |

≥ indicates recorded lower bounds when a request lacks usage. Token totals in those cases can also be partial. Quota pacing and transport errors remain in JSON and are not automatically attributed to harness work.

Time measures agent execution, excluding container preparation/download, harness installation and the external verifier. Provider pacing is included. Input tokens accumulate across calls, not in one context window. A stopped task cost is not a cost per solved task.

[Official pricing](https://developers.openai.com/api/docs/models/gpt-6-luna): per million tokens, input $0.10; cache reads $0.01; cache writes $0.125; output $0.50. Ordinary input subtracts both cache categories. Usage estimate, not an invoice; infrastructure and taxes excluded.

## 13. Nginx with custom logging and rate limiting

Install and run Nginx on port 8080, serve the requested index/custom 404 pages, configure request/error logs and a 10-request/s per-IP rate limit. All three passed 8/8 official checks.

- **Fenix:** installed Nginx, changed its actual configuration, recovered from nginx missing on PATH by using /usr/sbin/nginx, and started a persistent supervised foreground server. Verified successful configuration parsing, HTTP 200/404 content and log entries. An invalid zero process timeout and an extra unexpected task_finish field were rejected, then corrected. No owner clarification blocked execution. It called task_finish directly without repeated discovery.
- **Codex:** installed and configured the real service; a configuration-writing attempt required repair, then it validated syntax, HTTP responses, listener and logs. Started Nginx directly after package service startup was blocked by container policy.
- **OpenCode:** used a checklist and file patches, started Nginx, checked responses/processes and repeated a request with a distinct user agent to inspect logging.
- **Remaining Fenix opportunity:** reduce invalid arguments and avoid the first PATH failure by refreshing/using executable evidence after installation. The successful service outcome is demonstrated here, but this task explicitly names localhost and does not prove that all previously observed target-ambiguity blockers are fixed. The official rate-limit check is configuration evidence, not a full load/throughput experiment.

## 14. Self-signed development TLS certificate

Generate a 2048-bit RSA key, a one-year self-signed certificate with the requested subject, a combined PEM, a verification report and a Python checker. All three passed the official verifier.

- **Fenix:** inspected runtime capabilities, generated all artifacts, set the key and combined PEM to mode 600, ran the Python checker and OpenSSL checks, inspected validity/key length, and compared printed public-key digests. The expanded environment probe provided concrete tool availability rather than an empty result. Finished directly with tool evidence, without a clarification blocker.
- **Codex:** completed the work with three model calls, executed a checker that loads the certificate/key pair, and inspected permissions and subject/dates. However, it explicitly set the combined PEM to 644 even though that file also contains the private key. The official pass does not cover that exposure.
- **OpenCode:** inspected tools, generated the artifacts with both private-key-containing files at 600, wrote and ran the checker, checked the key and report, and updated its checklist.
- **Remaining Fenix opportunity:** preserve this targeted verification while trimming unnecessary setup/discovery. Where equality matters, prefer a direct assertion over merely printing two hashes for the model to compare. All three rely on Python's private ssl._ssl._test_decode_cert helper for decoding; it worked in the task environment but is a portability consideration, not an official failure. Protect copies/combined artifacts containing secrets as well as the original private-key file.

## 15. Recover rows from a truncated SQLite file

Recover available word/value records into JSON while preserving the damaged input. Each harness reported ten recovered rows and passed the official 1/1 check.

- **Fenix:** tried read-only SQLite inspection, inspected raw bytes and cell pointers after the database header was unavailable, decoded the remaining B-tree records, corrected the treatment of floating-point serial values before writing the result, and reloaded the JSON to assert row count and key structure. It completed without a clarification or stopping loop and without modifying the input database.
- **Codex:** recovered from unavailable diagnostic commands, inspected bytes with Python, decoded the cells, checked record length consistency and wrote JSON with nonfinite numbers disallowed. Completed with six model calls.
- **OpenCode:** inspected cells and values, repaired an incorrect record-header boundary during decoding, wrote ten rows and checked the resulting JSON. Completed with fifteen model calls.
- **Coverage limitation:** the official check requires more than six matches among eight reference rows; it does not independently certify every one of the ten recovered records or reject all extraneous rows. These fixture-specific decoders are not validated general SQLite recovery utilities.
- **Remaining Fenix opportunity:** use explicit payload-length/bounds and numeric-type assertions when recovering damaged records; consolidate related byte inspections while retaining provenance. A background skill-learning request started after successful work and ended with a gateway broken-pipe error before usage was recorded. The functional result is valid, but its cost/token figures are lower bounds. Evaluation teardown should drain or explicitly disable auxiliary learning and record its usage separately; do not silently charge or compare only foreground calls.

## Protocol and limitations

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `f7086c387fc10d270084440a33728fb264f17ca1e0727cae0799c97da75700d4`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Fresh official container per attempt, 1 CPU, 2 GiB, 900-second agent deadline. Rotating order: Fenix/Codex/OpenCode; Codex/OpenCode/Fenix; OpenCode/Fenix/Codex. Frozen binary, adapters and configuration. No behavioral changes, extra task attempts or hidden-test exposure during measurement. Identical task-independent environment context was added for all three harnesses; this protocol change precludes a direct before/after causal comparison. The same gateway respects provider quotas. JSON retains outcomes, timings, usage, pacing and private trace locations.

No deployment or runtime modification in this round. Three tasks cannot establish broad superiority; this is not a paired rerun measuring improvement on tasks 10–12.
