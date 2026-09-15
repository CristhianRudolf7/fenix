# Local browser

[Español](../es/browser.md)

Fénix controls a locally installed Chromium process through its own Rust CDP
transport (`fenix-browser`). There is no hosted browser service, Playwright runtime,
Selenium server, or browser-provider API. Chromium supplies HTML, CSS and JavaScript;
Fénix does not implement a new rendering engine. Keep the installed Chromium updated
through your operating system. Its sandbox remains enabled.

## Start and keep logins

Start Fénix normally, then open **Browser**. Create a named profile such as `personal`,
enter an HTTP(S) URL and click Open. Click the rendered screen and type normally;
paste, wheel scrolling and navigation keys work too. Complete passwords, QR codes,
MFA or CAPTCHA yourself. Toggle **Allow agent access** when Fénix should use that
profile. **Take control** disables subsequent agent commands for the profile; a command
already being executed can finish. Manual interaction also disables agent access.

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

## Agent workflow and testing

`browser-navigation` is a bundled skill. The model discovers real Rust tools using
`tool_search`: `browser_open`, `browser_observe`, `browser_act`, `browser_resize`,
`browser_inspect`, `browser_wait`, `browser_tabs`, `browser_check`, `browser_close`.
The skill describes when and how to use these tools; it does not execute actions.

Omit `profile` for a temporary profile owned by the task. A task cannot access
another task's temporary profile. Close it after testing to delete its browser
state. Named personal profiles require the owner's switch; `browser_tabs` with `action=profiles`
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
  "memory_budget_mib": 2048
}
```

No browser is launched until an open command. A monitor checks process-tree summed
RSS every five seconds and closes idle or over-budget sessions. Browser commands
can delay monitoring. This is a **soft budget**, not a cgroup-enforced ceiling;
shared pages may be counted more than once, and short spikes can exceed it.
Personal profile files survive these closures; unsaved forms do not. Looking at the
live screen counts as activity. Chromium's disk cache is capped at 64 MiB, screenshots
at 16 megapixels and 100 artifact entries per profile; CDP messages are bounded.
RAM depends on websites and profile count, not just the controller's language.

The server remains loopback-only with origin/host checks. Browser commands cannot
open the Fénix control URL; profile files are inside its protected runtime data.
Do not expose the control server or Chromium debugging port to the public network.

## Verification

`cargo test -p fenix-browser` covers encryption and authorization without starting
Chromium. Run the real-browser fixture explicitly on a sandbox-capable machine:

```sh
cargo test -p fenix-browser -- --include-ignored --nocapture
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
