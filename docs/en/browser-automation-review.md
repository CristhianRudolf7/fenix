# Browser automation review

[Español](../es/browser-automation-review.md)

Reviewed on 2026-09-19–20. Repository popularity is a selection aid, not evidence
that a workflow will succeed. Read source and maintained documentation at pinned
commits; no upstream code or hosted browser dependency was imported.

## Findings and changes in Fénix

Long workflows need durable login state, explicit target identity, observed progress,
bounded waits, and evidence of completion. Fénix already has task journals, active
context compaction, persistent Chromium profiles, isolated test profiles and owner
credential APIs. Replacing that runtime would not address the concrete gaps below.

- **Owner handoff:** restored the `/browser` view in the React interface and server
  routing. Owners can sign in, use the encrypted vault, choose a profile/tab,
  resize, and explicitly enable the agent. Direct input stays outside chat.
  Keyboard input supports Unicode, paste and composition; queued actions carry
  the displayed tab. The vault now honors that explicit tab too.
- **Long downloads:** a bounded 128-entry transfer ledger survives diagnostic-ring
  eviction. Waits expose bytes and pending/completed/canceled state. Completed
  files remain verifiable after browser restart. A canceled transfer returns
  immediately; pending waits never authorize replaying the download click.
- **Idle closure:** measure idle time from the end of a command. Recent increasing
  transfer bytes keep a session active. Repeated events with unchanged byte
  counts do not keep a stalled transfer alive indefinitely. Memory limits still
  apply; this is not a hard RAM reservation or an unlimited task lease.
- **Agent continuity:** actual received bytes advance the loop guard; repeated
  stagnant waits still count as a loop. Compact tool receipts retain up to eight
  files and eight transfer records when their metadata fits 4096 bytes, with the
  full output available through `task_inspect`.
- **Workflow guidance:** the browser skill describes owner waiting with `ask_user`,
  explicit profiles/tabs, observing after interruptions/compaction, and delivering
  only verified file links. Existing policies still govern submissions.

No promise is made of automatic CAPTCHA/MFA solving, cross-engine compatibility,
or successful planning on every website. Named profiles are still shared state:
concurrent tasks should use distinct profiles or explicitly coordinated tabs;
commands are currently serialized by the browser service.

See the [workflow audit](browser-workflow-audit.md) for measured tests and limits.

## Primary sources

- [browser-use/browser-use: browser_use/agent/service.py](https://github.com/browser-use/browser-use/blob/d8110c5ff87ccba887aaa726cdb780f2f84bef8d/browser_use/agent/service.py) (115,329 stars at review; commit `d8110c5ff87ccba887aaa726cdb780f2f84bef8d`). Also reviewed [related source](https://github.com/browser-use/browser-use/blob/d8110c5ff87ccba887aaa726cdb780f2f84bef8d/browser_use/browser/watchdogs/downloads_watchdog.py).
- [microsoft/playwright-mcp: README.md](https://github.com/microsoft/playwright-mcp/blob/f1257a5a67aff872f947fae274759f7d54853862/README.md) (37,354 stars at review; commit `f1257a5a67aff872f947fae274759f7d54853862`).
- [vercel-labs/agent-browser: skill-data/core/references/session-management.md](https://github.com/vercel-labs/agent-browser/blob/44583ac8385d814ab98cbf40feec97620376b50e/skill-data/core/references/session-management.md) (42,881 stars at review; commit `44583ac8385d814ab98cbf40feec97620376b50e`). Also reviewed [related source](https://github.com/vercel-labs/agent-browser/blob/44583ac8385d814ab98cbf40feec97620376b50e/skill-data/core/references/authentication.md).
- [browserbase/stagehand: packages/cli/src/lib/driver/session-manager.ts](https://github.com/browserbase/stagehand/blob/ea2789ca56e8a7ee154850de37657823c98604f8/packages/cli/src/lib/driver/session-manager.ts) (24,544 stars at review; commit `ea2789ca56e8a7ee154850de37657823c98604f8`). Also reviewed [related source](https://github.com/browserbase/stagehand/blob/ea2789ca56e8a7ee154850de37657823c98604f8/packages/extension/understudy/lifecycleWatcher.ts).
