# Scheduled workflows and native connections

[Español](../es/workflows-and-connectors.md)

Fénix supports ordinary scheduled prompts/reminders and versioned workflow definitions. The **Scheduled tasks** page shows both. Describe a procedure in the conversation; the agent saves its graph with `workflow_manage`. The diagram opens in the shared workspace panel. It supports zoom, step inspection, execution history and conversational changes. The standalone flow view collapses navigation; small screens stack the inspector below the diagram.

## Execution model

Definitions, version snapshots and run checkpoints are durable SQLite records in the authenticated user's Runtime. Each run gets a task ID, pinned project/workspace, immutable workflow snapshot and per-step outputs/status/attempts. A separate worker executes flows while the conversation stays available. Closing the web page does not cancel a run. Workflows must be acyclic; JSON data references may only read dependency outputs or earlier actions inside the same browser block. Conditions can select branches. Steps within one run execute serially in dependency order; independent runs may proceed concurrently (up to 16 active workflows).

Supported steps:

| Kind | Arguments and behavior |
| --- | --- |
| `tool` | A permitted existing tool and its exact arguments. Existing project, filesystem and approval policies apply. |
| `connector` | Connection ID, operation and operation arguments. Call `connector_call action=describe` for the input contract. |
| `browser` | Ordered existing DOM tools, including observe, act, upload and checks. Intermediate completed action results are saved. |
| `script` | Ordinary `command_run` arguments; completion requires exit code zero. |
| `prompt` | `prompt` instructions queued to the existing agent. The flow waits for its actual task result. |
| `condition` | `reference`, `operator`, `value`; outputs `matches`. |
| `wait` | `question`; asks in the conversation and waits for the normal textual response. |
| `notify` | `text`; publishes to the conversation and the originating task's configured delivery channel. |

Download connector steps can omit `destination` to use `automations/<workflow-id>/<run-id>/<step>.pdf` (or `.bin`). String arguments support only the safe identifiers `{{run_id}}` and `{{workflow_id}}` for unique paths; arbitrary expression/code interpolation is not supported.

Bindings such as `{"$ref":"search/files/0/id"}` preserve JSON types; missing outputs fail explicitly. A step's `when` uses the same condition shape. Ordinary dependencies require successful predecessors; skipped branches propagate to their descendants. This release does not offer arbitrary loop nodes or in-place changes to active definitions. A newly scheduled run uses the latest saved definition; an existing run keeps its pinned version.

Use `test` to run recognized read-only steps and pause before an effectful or unknown step. This is a partial preview, not a simulated successful upload or end-to-end validation. Each recognized read-only step may use one to three attempts. Writes are never automatically retried. Failure and restart preserve completed checkpoints. Explicit resume asks for a textual owner decision before repeating an effect with an uncertain outcome. Browser checks returning `passed=false` fail the step. This is not an exactly-once guarantee across third-party APIs: inspect a submission receipt before retrying an ambiguous upload.

Scheduled executions retain America/Lima and the existing once/interval/cron behavior. Missed times while the service is offline are not replayed. A flow with a queued/running/waiting execution rejects overlap instead of creating duplicates. Pausing a schedule affects future runs; cancel a run separately to stop current work.

## Connections

The catalog follows `/home/Rudy/proyectos/repo/Conexiones.md`. **Integrations** includes:

| Capability | Native support |
| --- | --- |
| Google Workspace | OAuth code/PKCE, refresh; Drive search/download/export; Docs, Sheets, Slides, Gmail and Calendar through their scoped APIs. |
| Microsoft 365 | OAuth code/PKCE, refresh; OneDrive/SharePoint, Outlook, Calendar and Excel through Microsoft Graph. |
| Slack | OAuth or token; channels, history, messages and native API requests. |
| GitHub | Personal access token; repositories, issues and native REST requests. |
| Email | IMAP over verified TLS on 993; SMTP over verified TLS on 465 or required STARTTLS on 587; list/read/send. Gmail and Outlook also support their OAuth connectors. |
| PostgreSQL | Verified TLS, prepared statements, scalar parameters, read-only transaction unless write access is enabled. |
| SQLite | Existing workspace database, parameterized statements, no attach/detach/pragma escape or symlink paths. |
| Telegram | Reuses the existing owner-specific bot/channel settings. |
| WhatsApp | Reuses the existing WhatsApp Web pairing and Fénix browser session. |
| Web search | Reuses Fénix's own search/index and browser tools. |
| PDF / office | Reuses native document tools for PDF, Word, spreadsheets and presentations. No Microsoft subscription is needed for local document work. |

`connector_call list` exposes account names and available operations, never credentials. `describe` returns operation input fields. Raw requests accept only a provider/service-relative path, supported methods, JSON body and query fields; hosts are fixed and redirects do not forward tokens. Graph downloads may follow a verified Microsoft download host without forwarding authorization. Downloads create new workspace files and return SHA-256 evidence; existing files are not overwritten. The default artifact limit is 25 MiB and JSON responses are bounded to 3 MiB. IMAP reads at most 20 recent headers or 128 KiB of an explicitly selected message and preserves unread state.

Each Runtime has its own encrypted connector vault (`connectors/accounts.enc`, AES-256-GCM) and local key, both protected by filesystem access controls. The key is kept in the same private runtime directory; this protects stored credential contents, not a compromised process or host administrator. The execution broker remains available independently; native connectors do not replace its host-defined operations. Never include secrets in a flow's scripts, arguments or prompts.

## OAuth setup

An administrator registers Fénix with each provider and registers the exact redirect URI:

```text
https://your-fenix-host/api/connectors/oauth/callback
```

Configure `FENIX_GOOGLE_CLIENT_ID` / `FENIX_GOOGLE_CLIENT_SECRET`, `FENIX_MICROSOFT_CLIENT_ID` / `FENIX_MICROSOFT_CLIENT_SECRET`, or `FENIX_SLACK_CLIENT_ID` / `FENIX_SLACK_CLIENT_SECRET` in the relevant runtime service. An owner may instead enter their own app credentials in the connection dialog. OAuth state is account-local, expires after ten minutes and is consumed before code exchange. Google/Microsoft use PKCE. Request only needed scopes; defaults distinguish read and write connections. Provider API enablement, app verification and organizational admin consent may still be required. These cannot be created or granted solely by adding the connector code.

Personal tokens do not gain refresh capability automatically. Gmail/Outlook accounts that disable password authentication should use OAuth, not IMAP passwords. Account connection checks perform authentication/read probes, never send a test message. A successful check confirms the tested connection, not every operation or resource permission.

The callback GET serves a public, static landing page without accessing any account. It removes the authorization query from browser history and submits a same-origin JSON POST through the authenticated account gateway. Only that POST exchanges the one-use state/code. This preserves the existing `SameSite=Strict` session cookie and its expiration; a cross-site redirect cannot perform an authenticated token exchange on its own. The distinction follows the browser's [SameSite cookie behavior](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie).

## Validation

Offline tests cover graph validation, conditions, checkpoints, optimistic versions, previews, overlap rejection, tenant separation, vault encryption, OAuth state/PKCE, API request contracts using local mock servers, SQLite queries and escapes, HTTP routes, and desktop/mobile flows. Real OAuth consent, provider scope policies, real SMTP delivery and university submissions require the owner's accounts; mock tests are not evidence of live provider authorization.

```sh
CARGO_INCREMENTAL=0 cargo test -p fenix-connectors
CARGO_INCREMENTAL=0 cargo test -p fenix-agent --test workflows
CARGO_INCREMENTAL=0 cargo test -p fenix-server --test workflows --test integrations
pnpm --dir apps/web build
pnpm --dir apps/web exec playwright test tests/workflows.spec.js tests/integrations.spec.js tests/oauth.spec.js
```

See [native Google Workspace improvements](google-workspace.md) for service selection, per-service checks, schemas, pagination, Gmail composition and uploads.
