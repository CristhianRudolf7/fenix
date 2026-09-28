# Harness comparison: Terminal-Bench tasks 10–12

[Español](three-harness-tasks-10-12-luna-low-2026-09-26.es.md) · [Measured data](three-harness-tasks-10-12-luna-low-2026-09-26.json)

GPT-6 Luna low, standard service, no Flex. One attempt per combination. Results: fenix 2/3, codex 1/3, opencode 1/3.

Evaluated changes: independent differential fixtures; review of nonessential clarification blockers; fresh receipts for declared executable paths; preserved missing-dependency diagnostics. Local validation: 251 tests passed, three optional tests skipped, successful server build.

| Task | Harness | Checks | Time (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| 10. polyglot-rust-c | fenix | 1/1 | 161.79 | 0.01780715 | 376,236 | 7,568 |
| 10. polyglot-rust-c | codex | 0/1 | 25.85 | 0.00354923 | 19,079 | 2,329 |
| 10. polyglot-rust-c | opencode | 0/1 | 97.88 | 0.00731708 | 135,373 | 6,705 |
| 11. git-multibranch | fenix | 1/1 | 163.68 | 0.02848899 | 485,799 | 4,536 |
| 11. git-multibranch | codex | 1/1 | 62.27 | 0.00907058 | 202,864 | 5,452 |
| 11. git-multibranch | opencode | 1/1 | 96.90 | 0.00778408 | 220,814 | 4,592 |
| 12. configure-git-webserver | fenix | 0/1 | 34.02 | 0.00783213 | 99,699 | 777 |
| 12. configure-git-webserver | codex | 0/1 | 27.20 | 0.00397572 | 39,688 | 2,409 |
| 12. configure-git-webserver | opencode | 0/1 | 54.30 | 0.00481098 | 76,775 | 3,550 |

≥ indicates recorded lower bounds when a request lacks usage. Token totals in those cases can also be partial. Quota pacing and transport errors remain in JSON and are not automatically attributed to harness work.

Time measures agent execution, excluding container preparation/download, harness installation and the external verifier. Provider pacing is included. Input tokens accumulate across calls, not in one context window. A stopped task cost is not a cost per solved task.

[Official pricing](https://developers.openai.com/api/docs/models/gpt-6-luna): per million tokens, input $0.10; cache reads $0.01; cache writes $0.125; output $0.50. Ordinary input subtracts both cache categories. Usage estimate, not an invoice; infrastructure and taxes excluded.

## 10. One Rust/C++ Fibonacci source

The named dataset task is polyglot-rust-c, but the instruction uses rustc and g++ (C++). Both must print Fibonacci with f(0)=f(1)=1.

- **Fenix:** repaired successive comment/raw-string errors. An early failed finish was rejected by the existing recovery gate; work continued. Later, the new invocation gate rejected a completed finish because Rust receipts predated the final edit. It reran Rust, then completed. Official result: 1/1. Quota pacing accounted for 28.535 seconds. Repeated searches included already-available finish/contract tools, showing residual discovery overhead.
- **Codex:** inspected the workspace and ended without writing the artifact, stating that it could not produce a reliable polyglot. Official result: 0/1 (missing source), with no quota error or deadline forcing the stop.
- **OpenCode:** made repeated delimiter patches before its combined compile check; Rust still failed and it reported failure. Official result: 0/1.
- **Remaining Fenix improvement:** tie the executable being tested to the compiler output. Its final rustc command used cwd=/app without -o, then invoked /app/polyglot/main left by an earlier explicit-output compile. The official verifier rebuilt the final source successfully, so the score is valid, but command success/fresh invocation alone does not prove binary provenance. Validate small candidate syntax early and avoid redundant discovery. The rejection trace demonstrates that the new freshness gate fired; this is not a controlled estimate of its effect on success rate.

## 11. SSH Git hosting with separate main/dev HTTPS deployments

Serve a password-authenticated repository and deploy both branches through post-receive hooks within three seconds. All three received the official 1/1 result.

- **Fenix:** installed/configured services, recovered from administrative commands absent from PATH by locating absolute paths, and repaired bare-repository ownership after an actual push failure. Changed the web layout to sibling main/dev directories, avoiding deletion of one deployment by publishing the other. Verified authenticated cloning/pushing, distinct HTTPS responses and a dev push under three seconds. No unnecessary owner question stopped the work. Some repeated finish-tool discovery remained despite the schema being available.
- **Codex:** built the server and exercised end-to-end pushes. The first push exposed a hook that deleted its staging directory before moving it; then a permissions issue prevented serving the deployment. Corrected both and verified both HTTPS responses, reporting an approximately 0.56-second push.
- **OpenCode:** implemented the service, hook and certificate; tested pushes and cloning, correcting its password/host-key test setup. Both endpoints returned the requested branch data.
- **Verifier caveat:** the official test reported success despite non-fast-forward push rejection after agent self-tests had already published the expected content. Its shell/Expect workflow checks matching endpoint content without reliably requiring each test push to succeed. We preserve official scores but do not treat them as independent proof of a new deployment; the agents' successful push receipts provide complementary evidence.
- **Remaining Fenix improvement:** resolve executable locations before reinstalling packages, reduce repeated discovery and use isolated/clean deployment fixtures with unique new content. Assert push success and resulting commit/content identity, not only cached HTTP text. These improvements follow the same meaningful-state-change principle as task 7.

## 12. Publish Git pushes through a web server on port 8080

Create /git/server and publish master so that a subsequent clone/commit/push exposes hello.html over HTTP. The request excludes login setup. All three failed the official 0/1 check: Git was still missing and the HTTP endpoint did not respond (HTTP 000). The image intentionally starts with Ubuntu and curl, so missing Git was a dependency to install, not evidence of a broken verifier.

- **Fenix:** loaded two relevant skills and inspected the environment. The inspection returned an empty runtime output with exit 0, providing little useful evidence. It asked whether /app was the intended server and requested the hostname. After the first question was refused, it supplied a requirement blocking_reason and closed blocked with owner_input_unavailable, without configuring anything. The new policy requires an explanation but does not establish that the explanation is sound: this remaining failure demonstrates its semantic limitation. No rate limit or time budget forced the stop.
- **Codex:** created a setup script and README for a presumed external server. It did not install Git or execute the setup; the final response delegated execution to the user. Its script assumes systemd without verifying the actual environment.
- **OpenCode:** also produced a setup script and README rather than configuring the running environment. It checked shell syntax and executable permissions, but those checks do not prove a working deployment. Git was missing; it did not install it or start the service.
- **Remaining Fenix improvement:** make the evaluation environment/destination explicit in the benchmark adapter for all harnesses, without supplying hidden test details or weakening production requirements. Before treating destination ambiguity as an owner blocker, inspect available local evidence and continue independent in-scope preparation where appropriate. Require evidence of an actually running service and successful push-to-HTTP flow for an execution request; a script is only sufficient when the requested deliverable is a script. Investigate the empty environment probe output rather than assuming an empty workspace identifies an unavailable remote host. Do not remove legitimate initial clarification or authorization checks in ordinary user tasks.

The three failures share an execution-versus-instructions mismatch, but Fenix stopped earlier through its clarification policy. These are remaining findings, not additional fixes applied during this measurement.

## Protocol and limitations

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `580f0588c8757a90304a327c9f3914a1d923adb860211351cc5d6054d3a17b32`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Fresh official container per attempt, 1 CPU, 2 GiB, 900-second agent deadline. Rotating order: Fenix/Codex/OpenCode; Codex/OpenCode/Fenix; OpenCode/Fenix/Codex. Frozen binary, adapters and configuration. No behavioral changes, extra task attempts or hidden-test exposure during measurement. The same gateway respects provider quotas. JSON retains outcomes, timings, usage, pacing and private trace locations.

No deployment or runtime modification in this round. Three tasks cannot establish broad superiority; this is not a paired rerun measuring improvement on tasks 7–9.
