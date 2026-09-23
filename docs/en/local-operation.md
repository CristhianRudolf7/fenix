# Local operation

[Interactive execution, updated limits and on-demand tools](execution-harness.md)

[Español](../es/local-operation.md)

## Start and configure

First install Node 22.12+ and pnpm 10.34.5, then build the frontend with
`pnpm --dir apps/web install --frozen-lockfile` and `pnpm --dir apps/web build`.
See [browser voice and Live](live.md) for development and voice configuration.

From the clone, run `cargo run --locked -p fenix-server --bin fenix` and open
http://127.0.0.1:4173. The default data directory is `.fenix/` relative to the
working directory. `--data-dir`, `--skills-dir` and `--port` provide explicit
overrides; `--help` lists them. Skills default to `~/.fenix/skills`, independently
of `--data-dir`. Listening is always on IPv4 loopback.

The welcome screen selects a provider, model, reasoning effort where supported
and an API key. Advanced options contain the endpoint, account/project headers,
credential scope, file workspace and action policies. OpenAI Luna with low
reasoning is preselected, but OpenAI is not mandatory. Even local Ollama requires
a nonempty key value in this initial onboarding flow; its native endpoint may
ignore that value. Bedrock and Azure require an explicit endpoint.

Setup sends a small real inference, which the provider can charge for. An invalid
key/model/endpoint does not complete setup. Selecting another provider or model
does not erase the conversation. Configuration changes wait until active and
queued tasks finish. Blank key fields retain the saved key for that exact provider
and project scope. This includes background children: model or permission changes
are rejected while work is active. A project literally named `default` is distinct from the
unscoped default key. Endpoints must support text and function calling for agent work.

## Normal use

Chat is continuous. New requests queue in arrival order. Replies to a pending
approval use `aprobar <request_id>` / `rechazar <request_id>` or `approve` / `reject`.
A clear yes/no works for the sole pending approval. Clarifications use
`responder <request_id> <answer>` or `reply ...`. Use `cancelar` or
`cancelar <task_id>` to stop active or queued work. These controls are textual;
cancellation does not reverse completed effects. Several subagents may request
approval or clarification simultaneously; use the exact request ID. A waiting
child leaves the master available for other requests.

Projects, memory, skills, schedules, tasks and instructions have web panels. Memory only
stores explicitly requested facts. To add a manual skill, create
`~/.fenix/skills/<name>/SKILL.md`, with `description:` and `version:` metadata in its
opening Markdown frontmatter. Refresh the Skills panel to discover it. Default
templates are never copied over existing files. The complete compact catalog is
visible to master and children, including disabled entries, but only enabled skill
instructions can be loaded on demand. Five default skills are included; they are
not a catalog-size limit. To keep an installation-specific library, supply an
explicit `--skills-dir`.

Existing `<data-dir>/skills` resources are copied to the selected global directory
on startup without overwriting destinations, following symbolic links or deleting
originals. Migration limits are 32 MiB per file, 512 MiB total, 10,000 entries and
16 levels. A limit failure leaves originals intact; use `--skills-dir` for the old
library or copy the remaining resources manually.

Tool catalogs vary by role and scope. The master delegates with a precise purpose
and expected output; context and constraints are optional. Children cannot read
the main conversation or delegate again, and inspect their own task plus explicitly authorized plan evidence. Their transcripts
remain separate; final reports return automatically to the master. The **Tasks**
panel shows purpose, expected output, IDs, state, attempts, results and pending
textual requests, refreshed while the view is open.

The engine enforces one delegation level and four concurrent child inference requests. Each child
has at most three total attempts for retryable failures, with configurable defaults
of 128 steps per task and one hour per attempt. Retries preserve context and completed tool results, without
replaying actions. Blockage, cancellation and non-retryable failure can end earlier.
File, memory and schedule changes and foreground command lifetimes are serialized
inside the runtime. Explicit persistent sessions can overlap later work; external
processes and other Fénix instances are not covered.
Commands can use the Linux user's accessible directories and are not sandboxed.

Schedule dates without an offset are Peru local time. Cron accepts five fields
(minute, hour, day, month, weekday) or six/seven fields including seconds and an
optional year. For example, `0 9 * * 1-5` means weekdays at 09:00 in Peru. A reminder
posts text; a scheduled task invokes the agent. A paused schedule resumes only
future executions. Ask in chat to cancel a schedule by name or ID.

## Projects without separate chats

Open **Projects** to create or edit a name, unique aliases, description, working
brief and optional existing folder. Mention the name or alias in the same chat.
General discussion remains available; “general conversation” explicitly returns
to that scope. Reply to an earlier message with the web **Reply** action or Telegram's
reply feature to provide a task reference. That reference is not an approval.

Fénix resolves the destination before loading project memories or using execution
tools. Clear references can use rules; other requests may need a routing call to
the configured model and can require a textual clarification. A compound request
can ask the master to consult A and delegate substantial work in B. Running work
retains its original project and workspace when you change subjects.

The **Memory** scope field selects general or one project's records. Existing
memories cannot change scope through an ordinary edit. Projects' progress dialogs
show recorded task state; this is not a fresh check of outside resources.

Project form saves are explicit owner replacements and increment the version;
they do not use optimistic locking. Reload a project's form before saving if other
work has changed its brief. Agent brief updates require an expected version and
reject stale writes. Polling updates the project list without replacing form edits.
Legacy history/memory is preserved as general; migration does not assign old text
to projects automatically. See [the project guide](projects.md) for details.

## Telegram

Create a bot using Telegram's `@BotFather`, paste its token in Settings and enable
Telegram. Fénix verifies `getMe` and shows a `/start <random-code>` pairing message.
Send that exact text privately to the bot. Only the paired owner can enqueue work.
Web and Telegram share the same history and queue. Groups, files, images and voice
are not processed. No public server or webhook is required.

Do not run another `getUpdates` poller or an active webhook for the same bot. The
web displays Telegram configuration and the latest connection error. Delivered
responses and reminders remain in the web conversation even if Telegram delivery
fails. Outgoing network retries may duplicate a message after an ambiguous result.

## Desktop installation

On Linux with a systemd user session, run `bash scripts/install-local.sh`. It builds
the release binary and installs:

| Item | Path |
| --- | --- |
| Executable | `~/.local/bin/fenix` |
| User service | `~/.config/systemd/user/fenix.service` |
| Application launcher | `~/.local/share/applications/fenix.desktop` |
| Installation / data | `~/.local/share/fenix/.fenix/` |
| Shared manual skills | `~/.fenix/skills/` |

This is a separate data directory from a development clone's `.fenix/`; it does not
copy keys automatically. The default skill library is shared with development
instances; skill enablement remains local to each runtime database. The installer enables the user service. The launcher
starts it and opens the web. No administrator privileges are needed. The service
follows the user's login session and is not configured to run while logged out.
Updating through the installer stops an existing Fénix user service before replacing
the binary; safe task boundaries can resume, while uncertain work remains interrupted.

For diagnostics: `systemctl --user status fenix` and
`journalctl --user -u fenix`. Stop it with `systemctl --user stop fenix`.
Uninstall the executable/launcher/unit after `systemctl --user disable --now fenix`,
then run `systemctl --user daemon-reload`. Personal data is deliberately retained;
remove it separately only if you intend to erase the installation's history and keys.

## Data and shutdown

The data directory is private (`0700`) and SQLite/secret files are `0600` on Linux.
Keys are stored locally without master-key encryption. Back up the shared skills
directory separately when it is outside the data directory. Back up the whole data
directory while Fénix is stopped; do not copy only a live SQLite main file while
WAL writes are pending. Runtime data, personal skills and `apuntes.md` are ignored
by Git. No credential is included in the binary or desktop package.

Closing a browser tab does not stop Fénix. Shutdown cancels managed processes.
On restart, safe task boundaries may resume; uncertain work remains interrupted.
History and schedules persist, but missed schedule runs and saved tool effects are
not replayed. Managed file history supports explicit restoration; process state is
not restored. See [continuity and restoration](harness-continuity.md).

[Local browser / Navegador local](browser.md)
