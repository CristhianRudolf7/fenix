# Local browser


[Español](../es/browser.md)

Fénix controls a locally installed Chromium process through its own Rust CDP
transport (`fenix-browser`). There is no hosted browser service, Playwright runtime,
Selenium server, or browser-provider API. Chromium supplies HTML, CSS and JavaScript;
Fénix does not implement a new rendering engine. Keep the installed Chromium updated
through your operating system. Its sandbox remains enabled.

## Start and keep logins

Open **[Browser](/browser)** to enter your browser directly. Fénix selects an
existing personal session or creates one automatically; it never navigates an
already open tab just because you opened the view. Use the address bar, tabs,
back/forward and reload controls, then click, type, paste or scroll directly in the
page. Mouse dragging, context clicks, double clicks, Unicode/IME input and touch
scrolling are supported. The viewport follows the available space automatically
while you have control; watching the agent does not resize its browser.

**Let Fenix continue** authorizes the agent to use this personal session.
**Take control**, navigation and manual input revoke that access for subsequent
commands; an operation already running can finish. Other agent tasks are unaffected.
This reuses the same account-isolated Chromium process and persistent profile,
not another browser. Existing sessions, downloads, site-dialog controls and the
password vault are under **Saved sessions and options**.

Direct input uses authenticated owner endpoints, without an LLM call or a chat/tool
journal entry. Complete passwords and external MFA yourself when necessary. The
agent can still perform an authorized login with credentials you explicitly supply.
Some sites, including Google, may reject automated browsers; this view does not
bypass their restrictions or guarantee sign-in.

The view requests compressed JPEG frames sequentially (up to approximately five
per second during interaction, one per second at rest). Metadata refreshes less
frequently. This is a remote page view, not a full desktop/video stream. Hidden
browser tabs and leaving this view stop polling and abort in-flight reads. They
do not cancel agent tasks or close an in-use browser. The existing idle/checkpoint
lifecycle saves eligible inactive browsers to disk; viewing alone does not create
another Chromium instance. Frames are served with `Cache-Control: private, no-store`.

Close the profile to release its processes. Cookies, local storage, IndexedDB and
other Chromium profile state stay under `<data-dir>/browser/profiles/<name>`.
A subsequent opening reuses that state. Website expiration, session-only cookies,
logout, device unlinking and revocation still apply; an account is not guaranteed to
remain logged in forever. No actual university or WhatsApp account is bundled or
automatically logged in. Personal profiles are separate from task-specific test
profiles. Avoid opening a Fénix-managed profile with another Chromium process.

The password panel stores credentials separately, encrypted with AES-256-GCM and
an Argon2id-derived key. Supply a master password of at least 12 characters to save,
fill or delete an entry for the current profile and exact origin. Filling never
submits the form and no password is returned to the model or stored in its journal.
The master password is not saved; forgetting it prevents decrypting the vault.
Native Chromium password saving is disabled to avoid Linux's plaintext fallback.
The **whole Chromium profile is not encrypted by this vault**: its private directory
has mode 0700, and the vault has mode 0600. The local debugging endpoint is unauthenticated: other processes on this machine
and OS administrators are outside this boundary. Use a trusted, single-owner host.

The agent can create a new named personal profile through `browser_open` for
owner-authorized recurring work. That profile is enabled for the agent and keeps
its login across browser closures. Existing profiles taken over by the owner are
not silently reauthorized. Input observations still omit values. The browser also
supports right/middle clicks and native range controls. Page permissions, disabled
or read-only controls and external MFA/CAPTCHA requirements still apply.

## Agent workflow and testing

`browser-navigation` is a bundled skill. The model discovers real Rust tools using
`tool_search`: `browser_open`, `browser_observe`, `browser_act`, `browser_resize`,
`browser_inspect`, `browser_wait`, `browser_tabs`, `browser_check`, `browser_close`.
The skill describes when and how to use these tools; it does not execute actions.

Omit `profile` for a temporary profile owned by the task. A task cannot access
another task's temporary profile. Close it after testing to delete its browser
state. New named profiles created for authorized work are enabled automatically; existing profiles still respect the owner's access switch. `browser_tabs` with `action=profiles`
lists enabled names without launching Chromium. Individual commands are serialized. Concurrent multi-step workflows must use
separate profiles to avoid sharing the active page. Up to two profiles remain open by default. Background report and plan
review workers cannot use browser effects.

Observation returns bounded page text, geometry and references to interactive
DOM elements, including open shadow roots. Actions include clicks, text, keys,
scroll, hover, drag, select, checkbox, upload from an authorized workspace, dialogs,
and history navigation. Frames can be selected by ID; detached or inaccessible
frame contexts return an error. There is no arbitrary model-provided JavaScript
execution or cookie extraction tool. Browser content is untrusted evidence.

Use actual CSS viewport sizes (for example 390×844, 768×1024 and 1280×800), optional
device scale and touch emulation. Viewport dimensions apply across a profile's tabs.
Check concrete visible/text/URL states and horizontal overflow, inspect bounded
network/JavaScript failure metadata, and capture screenshots. `screenshot_matches`
compares exact PNG bytes against an explicitly created baseline. Dynamic content,
fonts, animations and different environments can change that comparison.

Screenshots are visible to the owner; the current model interface receives text
and geometry, not image input. DOM checks are not a complete visual audit. Chromium
emulation does not replace real devices or testing in Firefox/WebKit. Complex
cross-process frames, closed shadow roots, canvas-only controls, anti-bot systems
and browser-native dialogs can require manual intervention. There is no promise
that every flight or university site can be automated without intervention.

## Resource controls

Edit `<data-dir>/browser.json` while Fénix is stopped, then restart:

```json
{
  "executable": "/usr/bin/chromium",
  "max_sessions": 2,
  "max_tabs": 4,
  "idle_seconds": 600,
  "hibernate_after_seconds": 60,
  "memory_budget_mib": 2048
}
```

No browser starts until needed. Finished tasks release their browsers; waiting
tasks may use encrypted process checkpoints on a qualified gVisor/Docker host.
Global admission considers available RAM and quotas. Unsupported suspension keeps
the original page open. See [browser lifecycle](browser-lifecycle.md) for ownership,
configuration, recovery limits and the completed real-checkpoint qualification.
Looking at the live screen counts as activity. Chromium's disk cache is capped at 64 MiB, screenshots
at 16 megapixels and 100 artifact entries per profile; CDP messages are bounded.
RAM depends on websites and profile count, not just the controller's language.

The server remains loopback-only with origin/host checks. Browser commands cannot
open the Fénix control URL; profile files are inside its protected runtime data.
Do not expose the control server or Chromium debugging port to the public network.

## Verification

See the [browser regression audit](browser-regression-audit.md) for the adversarial
matrix, React fixture, responsive checks and reproduction commands.

`cargo test -p fenix-browser` covers encryption and authorization without starting
Chromium. Run the real-browser fixture explicitly on a sandbox-capable machine:

```sh
node apps/web/tests/fixtures/build-browser-controlled.mjs
cargo test -p fenix-browser -- --include-ignored --nocapture --test-threads=1
```

HTTP tests cover same-origin access and credential error privacy. The frontend suite
also exercises the local browser controls when `/usr/bin/chromium` is available.


### Local verification record (2026-09-15)

The workspace suite passed 211 tests (four explicitly ignored platform/opt-in tests).
The browser fixture was also run explicitly against installed Chromium 152 with its
sandbox enabled: form input/clicks, a same-origin iframe, three viewport sizes,
screenshots, password-vault fill without secret observation, two tabs, closure and
reopening with persisted cookies/localStorage/IndexedDB all passed. The reopening
page does not recreate the saved state. Summed process RSS at the two-tab observation
was 1165 MiB in one run; this is neither peak nor private memory, and not a WhatsApp
or university benchmark. No real account was used. The complete web suite passed
12 tests, including the Browser panel. Later targeted checks cover owner downloads
as streamed attachments and the final browser controls.

## Search and rendered browsing

The `/browser` address bar opens an explicit HTTP(S) URL. Request research through
the conversation: the agent uses `web_search` / `web_read` and can open a result in
an authorized browser profile when interaction is required. The local search API
remains `POST /api/web/search`; no external search provider was added.
See [independent search](web-search.md) for its bounded coverage contract.

## Long downloads

`browser_inspect` includes `downloads.transfers` (ID, state and byte progress) and
`downloads.files` (completed files and owner links). A `browser_wait` on a download
can return `passed: false, timed_out: true` while transfer bytes continue to grow.
Wait on the same ID; do not click again. Canceled transfers return a terminal
failure state immediately. Completion requires a local finished file. Transfers
are tracked independently of the 100-event diagnostic history, and completed
files can be verified after Chromium restarts. The ledger retains at most 128
transfers; old completed files remain on disk.

See [automation review](browser-automation-review.md) and
[authenticated workflow audit](browser-workflow-audit.md) for the tests and limits.
