# Account profile and recorded usage

[Español](../es/account-profile.md)

The header's **View my profile** opens `/profile`, a React Router page with Overview, Usage and Password sections. The layout reuses the application palette, supports light/dark themes, and keeps mobile navigation in the left drawer. The page is available before a model is configured. Standalone runtimes without an account gateway show that account management is unavailable.

Owners may update their display name; the sign-in email is read-only because there is no email-change verification flow. `GET/PUT /api/auth/profile` exposes only the authenticated account's email and display name. Password changes require the current password and a different replacement of at least eight bytes. They are rate limited, use Argon2, and atomically update the host registry under the same file lock as the provisioning CLI. All previous sessions become invalid; sign in again. The fixed seven-day session policy and the execution credential epoch remain unchanged, so the password change does not revoke running task access. No registration endpoint is introduced.

`GET /api/account/usage?days=7|30|90` aggregates receipts from the owner's runtime database. Daily boundaries use America/Lima (UTC−05:00). It returns input, output, total tokens, recorded receipt count and incomplete-usage count, including empty days. Graph data has an accessible daily table; zero activity and failed loading are distinct states.

The allowlist covers model responses, context summarization/compaction, project routing, skill learning, uncached decision evaluations, non-duplicated memory extractions, library embedding receipts and newly recorded interlocutor/narration usage. Tool results that repeat usage are excluded. Cached tokens remain part of the provider's input count; reasoning tokens are not added again to output. This is **recorded usage, not a complete billing ledger**: historical voice calls, connection probes, failed requests and other unrecorded operations may be missing. No dollar estimate or fictitious activity is shown. There is no model call to generate the chart.

Validation: storage receipt aggregation/deduplication/isolation, gateway owner-only edits and password/session behavior, plus Playwright route, save, range selection, empty/error states, mobile layout and password confirmation. Frontend and backend are published separately using the existing systemd deployment.
