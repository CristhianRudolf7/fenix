# Fénix

[Español](README.md)

**Current version: 0.2.0**

A local personal AI assistant for Linux. A Rust service provides a web interface
and optional Telegram with **one continuous conversation**, local tools, explicit
memory, manual skills, reminders and scheduled work. No Node server is needed at
runtime.

## Run locally

Install Rust using [rustup](https://rust-lang.org/tools/install/), then run from
this repository:

```sh
cargo run --locked -p fenix-server --bin fenix
```

Open **http://127.0.0.1:4173**. The welcome screen requires a provider API key and
verifies a real inference before enabling chat. Telegram is optional. The current
default is **OpenAI `gpt-5.6-luna`, low reasoning**. Any of the 17 integrated
providers can be configured; choose a model supporting tool calls.

For a desktop launcher and a background user service on Linux with systemd:

```sh
bash scripts/install-local.sh
```

This installs under your home directory without `sudo`. Open **Fénix** from your
application menu afterward. Closing a browser tab does not stop the agent. Your
computer and service must stay on for Telegram and scheduled work.

## Available features

- Bilingual web with chat, projects, provider setup, Telegram, memory, skills, schedules and
  task traces. White light mode is the default; an explicit dark-mode choice persists.
- One active master task and a FIFO conversation queue, with up to four background
  subagents. Delegated work has its own transcript and returns a report to the
  master. The master can serve other requests while children work or await approval.
- Master tools cover local work, memory, projects and delegation. Children have
  a restricted catalog without conversation history or delegation, plus a required
  finish report; they can inspect only their own task.
  Commands may use any OS-accessible directory; dedicated file tools stay within the configured workspace.
- Backend-enforced, single-action textual approvals for commands and file changes
  by default. Local settings can explicitly allow those actions.
- SQLite history, task/event IDs, source references, token usage, bounded outputs
  and recoverable output pages. Basic context summaries preserve original history
  and complete tool-call/result groups.
- Projects with names, aliases, briefs and optional workspaces within the same
  chat. Routing precedes memory retrieval; general conversation has its own scope
  and executions retain their project bindings when the foreground topic changes.
- General and project memory CRUD, scoped exact duplicate checks and Unicode
  lexical search with FTS5. No automatic extraction or embeddings.
- Four versioned manual skills: `memory-and-reminders`, `local-file-work`,
  `conversation-recap`, `local-troubleshooting`.
- One-time, interval and cron schedules in `America/Lima`. Reminders deliver text;
  scheduled agent tasks join the same queue. Identical active schedules and
  overlapping executions of one schedule are checked.
- Optional Telegram pairing for one owner, private text messages only.
- Static and project-scoped credentials for 17 providers and five protocols.

Runtime data lives in ignored `.fenix/` by default. Credentials are stored in a
private local SQLite database, without a separate encryption master key. Known
keys are redacted from agent output and never returned by settings endpoints.
The service binds to loopback and rejects foreign browser origins and hosts.

Web search, browsers, heartbeat, voice, advanced memory, action
restoration and automatic task recovery remain in [ROADMAP.md](ROADMAP.md).
Interrupted tasks are recorded, not replayed. The web previews OpenAI Responses text as it arrives and shows live tool activity.
Other provider adapters retain complete responses; Telegram delivers complete messages.
Reconnecting the browser restores the current preview without repeating inference.

## Background work and shared skills

Every subagent receives a precise purpose and expected output, with optional
context and constraints; there is no mandatory “information needed” field. The
engine enforces one delegation level, four concurrent children, at most three
total attempts for retryable failures, and 32 steps / ten minutes per attempt.
Retries preserve the child transcript and completed tool results rather than
replaying tool actions. Blocked, cancelled and non-retryable work can stop early.
Restarting interrupts unfinished children; it does not automatically resume them.

The **Tasks** view shows purpose, expected output, state, IDs, attempts and results.
Answer in the conversation with `aprobar <request_id>`, `rechazar <request_id>` or
`responder <request_id> <answer>`; cancel with `cancelar <task_id>`. Each pending
request has its own ID, so a waiting child does not block the master. Model and
permission settings cannot change while a task is active. Child history remains
outside the main conversation; explicit memory is still available through tools.
The runtime serializes file/memory/schedule mutations and managed command lifetimes
within its process. Commands retain Linux user access and are not sandboxed.

Skills live in **`~/.fenix/skills/<skill-name>/SKILL.md`**, shared across runtime
data directories. `--skills-dir PATH` overrides this location; `--data-dir` only
changes runtime data. Master and children see the complete compact metadata
catalog, including enabled state; only `skill_load` loads enabled instructions.
Disabled skills stay visible but cannot be loaded. The four defaults are templates,
not a limit on how many skills can be installed.

On startup, legacy `<data-dir>/skills` resources are copied into the selected
library before defaults are installed. Existing destination files are never
replaced, symbolic links are skipped and the originals remain intact. Automatic
migration is limited to 32 MiB per file, 512 MiB total, 10,000 entries and 16 levels;
if exceeded, select the old directory with `--skills-dir` or copy resources manually.

## Repository layout

```text
apps/server/             Rust executable and loopback HTTP API
apps/web/                Browser UI embedded in the Rust executable
crates/fenix-core/       Provider-independent conversation contracts
crates/fenix-credentials/ Secret handling and exact project credential resolution
crates/fenix-providers/  Provider catalog, HTTP clients and protocol codecs
crates/fenix-storage/    SQLite migrations, conversation, journal and lexical memory
crates/fenix-agent/      Agent loop, context, permissions, tools and channel adapters
resources/              Runtime instructions and default skill templates
packaging/linux/        Desktop launcher and systemd user unit
scripts/                Local installer and opt-in live verification
docs/en/                Primary documentation
docs/es/                Spanish documentation
config/                 Provider-library configuration examples
```

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked -p fenix-server
npm --prefix apps/web ci
cd apps/web
npx playwright install chromium
npm test
```

Default tests use local HTTP endpoints and fake credentials. Browser tests start
the real Rust service with isolated data. Node and Playwright are development
dependencies only. For an opt-in billable Luna check, with `OPENAI_API_KEY` already
set in the environment, run `python3 scripts/smoke-live.py` from the repository
root. It uses temporary data and writes a secret-free report under ignored
`target/`. Mock coverage of all 17 providers does not establish live access to all
17 accounts; see the verification record.

- [Projects in one conversation](docs/en/projects.md)
- [Architecture](docs/en/architecture.md)
- [Local installation and operation](docs/en/local-operation.md)
- [Verification record](docs/en/verification.md)
- [Provider library usage](docs/en/development.md)
- [Provider endpoints and authentication](docs/en/provider-reference.md)
- [Current requirements](REQUIREMENTS.md)
- [Future work](ROADMAP.md)
