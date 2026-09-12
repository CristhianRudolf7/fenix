# Local operation

[Español](../es/local-operation.md)

## Start and configure

From the clone, run `cargo run --locked -p fenix-server --bin fenix` and open
http://127.0.0.1:4173. The default data directory is `.fenix/` relative to the
working directory. `--data-dir`, `--skills-dir` and `--port` provide explicit
overrides; `--help` lists them. Listening is always on IPv4 loopback.

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
and project scope. A project literally named `default` is distinct from the
unscoped default key. Endpoints must support text and function calling for agent work.

## Normal use

Chat is continuous. New requests queue in arrival order. Replies to a pending
approval use `aprobar <request_id>` / `rechazar <request_id>` or `approve` / `reject`.
A clear yes/no works for the sole pending approval. Clarifications use
`responder <request_id> <answer>` or `reply ...`. Use `cancelar` or
`cancelar <task_id>` to stop active or queued work. These controls are textual;
cancellation does not reverse completed effects.

Memory, skills, schedules, activity and instructions have web panels. Memory only
stores explicitly requested facts. To add a manual skill, create
`.fenix/skills/<name>/SKILL.md`, with `description:` and `version:` metadata in its
opening Markdown frontmatter. Refresh the Skills panel to discover it. Default
templates are never copied over existing files.

Schedule dates without an offset are Peru local time. Cron accepts five fields
(minute, hour, day, month, weekday) or six/seven fields including seconds and an
optional year. For example, `0 9 * * 1-5` means weekdays at 09:00 in Peru. A reminder
posts text; a scheduled task invokes the agent. A paused schedule resumes only
future executions. Ask in chat to cancel a schedule by name or ID.

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

This is a separate data directory from a development clone's `.fenix/`; it does not
copy keys automatically. The installer enables the user service. The launcher
starts it and opens the web. No administrator privileges are needed. The service
follows the user's login session and is not configured to run while logged out.
Updating through the installer stops an existing Fénix user service before replacing
the binary; unfinished tasks will be marked interrupted when it restarts.

For diagnostics: `systemctl --user status fenix` and
`journalctl --user -u fenix`. Stop it with `systemctl --user stop fenix`.
Uninstall the executable/launcher/unit after `systemctl --user disable --now fenix`,
then run `systemctl --user daemon-reload`. Personal data is deliberately retained;
remove it separately only if you intend to erase the installation's history and keys.

## Data and shutdown

The data directory is private (`0700`) and SQLite/secret files are `0600` on Linux.
Keys are stored locally without master-key encryption. Back up the whole data
directory while Fénix is stopped; do not copy only a live SQLite main file while
WAL writes are pending. Runtime data, personal skills and `apuntes.md` are ignored
by Git. No credential is included in the binary or desktop package.

Closing a browser tab does not stop Fénix. Shutdown cancels managed processes.
On restart, previous running/waiting/queued tasks are marked interrupted. Original
history and schedules persist, but missed schedule runs and task effects are not
replayed. There is no automatic rollback or process restoration in this version.
