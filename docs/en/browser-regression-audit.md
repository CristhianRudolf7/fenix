# Browser regression audit

[Español](../es/browser-regression-audit.md)

For agent-level authenticated navigation, task continuity and slow downloads, see
the [authenticated browser workflow audit](browser-workflow-audit.md).

This suite exercises Fénix's Rust browser controller against real, sandboxed
Chromium. Tests inspect the resulting DOM, application state, downloaded bytes
and PNG dimensions; a successful tool acknowledgment alone does not pass a case.
All local browser data lives in temporary directories. Personal profiles and real
accounts are not used.

Verified on 2026-09-19 with Chromium 153.0.8010.47 on Debian 13: **59/59 matrix
scenarios, 6/6 Rust tests and 8/8 focused UI tests passed**. The persistent-profile
test additionally verifies an actual IndexedDB token value after restarting,
not just the existence of the database. Rust Clippy passed with warnings denied.
The [machine-readable matrix report](../benchmarks/browser-regression-2026-09-19.json)
records individual outcomes and elapsed times. The rebuilt application also passed
the live browser-controls workflow and its welcome/setup test (2/2), including
manual screen input, resizing, owner handoff, closure and profile persistence.

## Coverage

The adversarial matrix contains 59 independent scenarios:

- Unicode, emoji, combining characters, multiline text, replacement, clearing,
  caret insertion, Enter, literal `+`, and keys directed at a particular input.
- Text, email, number, date, time, datetime, month, week, color, textarea and
  contenteditable controls. Invalid values, maxlength, read-only, disabled,
  inert and noneditable targets must fail rather than claim a successful fill.
- A real React controlled form, including component state after submission.
- Checkboxes, radio buttons, prevented changes, disabled select options and
  optgroups, and a hidden file input with file name and size verification.
- Partial overlays, complete overlays, clipped targets, hover, double-click,
  native HTML drag-and-drop, duplicate selectors and DOM replacement on focus.
- Open Shadow DOM, scoped text checks and delayed controls.
- Nested frames, cross-site frames in separate Chromium processes, nesting
  within those frames, CSS scaling and a parent overlay covering a frame.
- JavaScript alerts and prompts, history navigation and SPA history.

A second local workflow checks element references and their invalidation,
observation privacy, redirects, downloads, popups, tab selection/closure,
responsive layouts, screenshots, new-tab emulation inheritance, cancellation
and temporary-profile cleanup. Viewports include 390×844 at DPR 3, 768×1024 at
DPR 2, 1280×800 and 1920×1080. It checks media-query layout, touch emulation,
actual PNG pixel dimensions and clicks after resizing.

The existing persistent-profile test also covers vault encryption/fill,
cookie/localStorage/IndexedDB persistence after closing and reopening Chromium,
and closing a tab outside the controller. A separate opt-in network smoke test
fills and submits synthetic values to [Selenium's public test form](https://www.selenium.dev/selenium/web/web-form.html),
then checks the submitted values and receipt page.

Browser UI tests cover keyboard shortcuts, paste, dead-key input, composition
events without duplicated text, clearing the local keyboard buffer and wheel
units. Composition events are simulated; this does not certify every physical
keyboard, operating-system IME or mobile device.

## Repairs exercised by the tests

The browser now validates editable targets and the value actually accepted by a
fill. Date-like controls and colors use their native value setter and input/change
events; ordinary text uses native Chromium input. Target resolution, waits and
observations share visibility, disability and Shadow DOM semantics. Click points
respect clipping and overlays.

Frame handling attaches to the correct Chromium process, preserves site isolation,
transforms coordinates and checks the embedding page at each process boundary.
Dialogs interrupt actions promptly so a subsequent `dialog` action can handle
them. `text` supplies a prompt reply. An interrupted action must not be blindly
repeated: the triggering click may already have happened. Already-cancelled
operations cannot close a live profile.

The manual screen uses an editable keyboard proxy for composition and native
text input, forwards the text once and clears the proxy. Wheel line/page units
are converted to pixels using the remote viewport. Pointer coordinates are captured
without allowing keyboard focus to scroll a tall screenshot away from the click.

## Reproduce

Install the repository's Rust and frontend dependencies and a sandbox-capable
local Chromium. Keep the sandbox enabled. From the repository root:

```sh
node apps/web/tests/fixtures/build-browser-controlled.mjs
cargo test -p fenix-browser
cargo test -p fenix-browser adversarial_browser_matrix -- --ignored --nocapture
cargo test -p fenix-browser browser_navigation_resources_and_responsive_layout -- --ignored --nocapture
cargo test -p fenix-browser real_chromium_navigation_responsive_and_persistent_login -- --ignored --nocapture
# Optional: requires external network access.
cargo test -p fenix-browser public_selenium_form_smoke -- --ignored --nocapture
pnpm --dir apps/web exec playwright test browser-search.spec.js
# End-to-end controls in a fresh temporary Fénix instance:
pnpm --dir apps/web build
cargo build -p fenix-server
FENIX_TEST_APP_PORT=4199 pnpm --dir apps/web exec playwright test application.spec.js --grep 'welcome validates|local browser controls'
```

For one serial run of all Rust browser tests, after building the React fixture:

```sh
cargo test -p fenix-browser -- --include-ignored --nocapture --test-threads=1
```

The matrix writes `target/browser-adversarial-report.json`, with each scenario's
outcome and elapsed time. `FENIX_BROWSER_CASE=frame` filters the matrix to frame
cases and overwrites that report with only the selected cases. Summed process RSS
printed by workflow tests is an observation, not a memory guarantee: shared pages
can be counted more than once and measurements are machine/load dependent.

## Limits

Passing this suite does not guarantee every website. It does not certify real
university or WhatsApp sessions, payment/booking flows, CAPTCHA, closed Shadow DOM,
canvas-only widgets, physical devices or other rendering engines. Chromium remains
the rendering engine; the controller and fixes are local Rust/JavaScript code with
no hosted automation provider. General Fénix planning logic is outside this audit.
