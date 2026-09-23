# Authenticated browser workflow audit

[Español](../es/browser-workflow-audit.md)

Verified on 2026-09-20 with Chromium 153.0.8010.47. The final authenticated
workflow passed with **46 tool calls, 2,457,600 downloaded bytes and one download
request in 125.24 seconds**. Active context compaction occurred during the task.
Build time is excluded from the workflow duration. The model decisions were scripted.
See the [implementation review](browser-automation-review.md) for the follow-up changes.

## Scope

The opt-in `agent_authenticated_multistep_workflow_survives_wait_timeouts` test
uses the actual agent worker, tool discovery, browser authorization, local Chromium,
persistent profiles and download files. Model responses are scripted; the website,
credentials and report are synthetic. This verifies execution mechanics, not a
live model's ability to plan an unfamiliar university website.

The workflow checks:

- Agent password entry is rejected. The owner completes a separate login, and
  Chromium is closed before the agent resumes using its authorized profile.
- Authenticated navigation survives reopening with the saved persistent cookie.
- An asynchronous search receives the correct Unicode input and returns results.
- Eight linked course pages are read and their individual content is verified.
- The task compacts its active context mid-workflow and continues without losing
  the browser session or skipping/replaying an action.
- An authenticated file streams for 99 seconds. One short wait and three 30-second
  waits return pending with increasing byte counts; a final wait verifies completion
  without clicking download again.
- The downloaded bytes match the expected report, only one download request occurs,
  and a download link remains available after closing the browser.
- The owner's password and authentication cookie do not enter agent history.

## Defects found and corrected

The initial run failed with `Recovery budget exhausted for no_progress` while
advancing through different course pages. The loop detector counted identical
`click #next` acknowledgments without recognizing subsequent page observations.

The agent now tracks observed page evidence separately for each profile, tab and
frame. Changed page evidence advances its progress signature. Fresh element refs
and screenshot filenames do not count as progress, and a successful click alone
still does not prove that a page changed. Unit tests cover advancing pages,
unchanged observations with fresh refs, failures and switching unchanged tabs.

`browser_inspect` now also returns completed file IDs, byte counts and owner
download URLs. The scripted model uses that returned URL in its answer instead
of inventing an application route from the download GUID. Transfer state is now
independent of diagnostic event eviction; pending waits preserve byte progress,
canceled transfers return immediately, and actual byte changes advance the loop
guard. Compact receipts preserve bounded download metadata.

## Follow-up validation

The focused validation set contains **23 passing cases**:

| Area | Cases |
| --- | ---: |
| Agent execution state / loop detection | 6 |
| Compact tool receipts | 5 |
| Browser transfer ledger and Chromium regressions | 5 |
| Authenticated agent workflow | 1 |
| Owner HTTP routes / privacy / streamed files | 1 |
| Production interface and account gateway | 5 |

The current implementation also passed six execution-state tests, five compact
receipt tests, two bounded-ledger unit tests, the real-Chromium download eviction /
cancellation / restart test, explicit-tab vault isolation, the persistent-login /
responsive-browser regression, and the owner HTTP route / privacy / streamed-file
test. The production web build passes both TypeScript configurations.

The browser interface tests use the production bundle with mocked owner APIs;
they verify Unicode and composition, keys, tab targeting, resizing, vault request
routing and field clearing, profile handoff, download links, polling cleanup, the
displayed page URL, and a pending handoff invalidated by a tab switch. The account
suite checks the gateway `/browser` route, login/logout and phone layout.
They do not substitute for a live account or live-model website evaluation.

## Operational limits

The inspected instance uses 128 model steps and a four-hour active task timeout.
Browser defaults are two open profiles, four tabs per profile, ten-minute idle
closure and a soft 2048 MiB memory budget. These are limits, not a four-hour soak
test or a guarantee of uninterrupted website access.

The agent cannot enter a password supplied in chat. The owner Browser view has now
been restored at `/browser`, with manual login, profile authorization and encrypted
vault controls. The inspected running instance had no personal profiles; rebuilding
and deploying the application is separate from these source-level tests.

This audit does not use real university accounts, test MFA/CAPTCHA handling, certify
arbitrary websites or deploy a rebuilt application into the running instance.

## Reproduce

```sh
cargo test -p fenix-agent --test browser_workflows -- --ignored --nocapture
cargo test -p fenix-agent --lib execution_state::tests
cargo test -p fenix-agent --lib tool_receipt::tests
cargo test -p fenix-browser --lib download -- --include-ignored
cargo test -p fenix-browser --lib owner_vault_fills_the_explicit_tab -- --ignored
cargo test -p fenix-browser --lib real_chromium_navigation_responsive_and_persistent_login -- --ignored
cargo test -p fenix-agent --test runtime long_active_task_compacts_and_continues_without_replaying_tools
cargo test -p fenix-agent --test runtime repeated_identical_tools_stop_without_erasing_the_audit_trail
cargo test -p fenix-server --test browser
pnpm --dir apps/web build
pnpm --dir apps/web exec playwright test tests/browser.spec.js tests/accounts.spec.js
```

Run on a machine with sandbox-capable Chromium. The test uses temporary runtime
data and synthetic credentials; it does not touch the owner's browser profiles.
