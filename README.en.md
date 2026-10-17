# Fénix

[**Documentation → fenix.crehlabs.com**](https://fenix.crehlabs.com) · [Leer en español](README.md)

**Development package version: 0.3.2.** An AI assistant with a continuous conversation, background execution, and coding, document, and browser tools. This version targets individual local installations; the multi-user execution sandbox is retired. Capabilities depend on configured services and credentials; historical evaluations do not guarantee success on arbitrary tasks.

## Capabilities

- Persistent chat, scoped projects, concurrent tasks, and configurable fast conversational and intelligent execution roles; optional Telegram and separately configured connections.
- Purpose-specific background subagents, one delegation level, follow-up messages, and an available master agent.
- Editable plans, milestones, dependencies, questions, durable asynchronous events, conversational autonomy, and requirement/risk-based verification.
- File tools, interactive terminals, persistent processes, regex/glob search, multi-file patches, configurable LSP, and Git worktrees.
- Scoped memory with provenance, entities, relations, revisions, retrieval, durable context checkpoints, and saved-result search.
- On-demand skill discovery, private explicit-save editing, persisted translations, and selective reviewed learning with recoverable revisions. Publication remains in the backend; the current UI exposes personal skills.
- Chromium browser tools with persistent profiles and idle/capacity management. There is no browser section in the web UI. Full disk hibernation of unfinished forms is not available in the native engine.
- Independent web search using a local index and crawler, plus lightweight HTTP reading; coverage depends on the indexed corpus.
- Private document versions, extraction, chunks, citations, and hybrid search with optional embeddings. Native DOCX/XLSX/PPTX tools and separately provisioned PDF/OCR components.
- Inline design suggestions, modal previews, a right-hand results panel, diagrams, documents, multimedia, and a user-scoped file explorer.
- Cron, reminders, scheduled workflows, connectors, and browser actions.
- Personal profiles, recorded token usage and email/password authentication with exact seven-day sessions.
- Native evaluation and benchmark adapters with recorded outcomes, time, and usage.

## Architecture

Rust/Axum runs the backend. Vite, React, TypeScript, Tailwind, and pnpm power the frontend. Static production deployments use Caddy/systemd independently of the backend, so UI updates do not require rebuilding Rust.

`apps/` contains applications, `crates/` reusable Rust components, `resources/` runtime instructions and assets, `scripts/` operational tooling, `packaging/linux/` deployment templates, and `tests/` cross-component checks.

Root Markdown documentation is Spanish except for this English README. Existing guides under `docs/es/` and `docs/en/` retain their language versions. Runtime skills, prompts, and fixtures retain their operational language.

## Public installation

`scripts/install.sh` installs the personal application on Linux x86_64 with glibc ≥ 2.39 and a systemd user session, without Cargo, Node or pnpm. It preserves personal data and does not replace hosted deployment services.

Release **0.3.1** provides the prebuilt application and frontend. On Ubuntu 24.04 or newer (x86_64) with a systemd user session:

```sh
curl -fsSL https://fenix.crehlabs.com/install.sh | bash
```

Use `FENIX_VERSION=v0.3.1` to select a published version and `FENIX_PORT` to choose another local port. Do not run with sudo. Open `http://127.0.0.1:4173` to configure your provider, model and API key. OpenSSL 3, zlib and zstd runtime libraries are required. Personal data and skills remain under `~/.local/share/fenix-personal/`; `systemctl --user disable --now fenix-personal.service` stops the application without deleting data. Optional browser and document dependencies need additional setup.

## Development and individual operation

Install the Rust toolchain pinned in `rust-toolchain.toml`, a Vite-compatible Node version, and pnpm **10.34.5**:

```sh
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo run --locked -p fenix-server --bin fenix owner you@example.com --password-stdin
cargo run --locked -p fenix-server --bin fenix
```

The owner command reads its password from standard input: enter a password of at least eight bytes, then end input (Ctrl+D in a Unix terminal). Run it before starting the server. Configure both model roles with your own credentials; installation keys and personal data are not distributed through GitHub.

The interface uses local components with Tailwind, Radix and Motion; resolved versions are pinned in the pnpm lockfile.

The individual server defaults to `127.0.0.1:4173`, with new runtime data in the platform application-data directory. An existing local `.fenix/` remains in use. It accepts `--data-dir`, `--skills-dir`, and `--port`. Provider and conversational role configuration belongs to the installation; source defaults do not imply provider access or dictate shared-service configuration.

For frontend development, `pnpm --dir apps/web dev` serves Vite on port 5173 and proxies `/api` to the individual backend. The personal server requires email/password login and no multi-account gateway.

`scripts/install-local.sh` retains an individual systemd installation mode. For separate frontend/backend deployment, follow the Spanish [deployment guide](docs/es/frontend-deployment.md). Older isolation guides are historical and do not describe a supported 0.3.2 deployment.

## Documentation and checks

See [requirements](REQUIREMENTS.md), [roadmap](ROADMAP.md), [changelog](CHANGELOG.md), [architecture](docs/es/architecture.md), [coding](docs/es/coding-harness.md), [memory](docs/es/adaptive-memory.md), [documents](docs/es/document-library.md), [browser lifecycle](docs/es/browser-lifecycle.md), and [native evaluation](docs/es/native-evaluation.md).

```sh
cargo fmt --all --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
pnpm --dir apps/web check
pnpm --dir apps/web test
```

Browser, OCR, infrastructure, and benchmark tests have additional dependencies. Mock-provider tests are not model quality measurements or complete production validation.

**Pending:** validate the firewall on actual deployment infrastructure, including allowed/blocked destinations, DNS restrictions, and execution termination when protection is lost. See [security boundaries](docs/es/security-boundaries.md).

The initial engine context capacity is 200,000 tokens with compaction at 160,000; it does not increase provider capacity or rate limits. File restoration cannot undo arbitrary external effects. Browser and document compatibility have documented limits. Keep credentials, databases, sessions, and personal artifacts out of Git.

## Documentation website

The public documentation is available at **[fenix.crehlabs.com](https://fenix.crehlabs.com)**. The Astro + Starlight + TypeScript site provides local Pagefind search, responsive navigation, syntax highlighting, light/dark themes, current guides, and clearly labeled historical references. Build it with `pnpm --dir apps/docs install --frozen-lockfile` and `pnpm --dir apps/docs build`; preview it with `pnpm --dir apps/docs preview`.

`scripts/publish-docs.sh` atomically publishes built files. Adapt `packaging/linux/Caddyfile.docs` to the host before installing it. Caddy serves documentation at `/` and `/docs/`; the existing remote mobile view remains at `/movil.html`. No model calls or API keys are needed to build or search the documentation. A public one-command installer is still planned.

## Personal execution in 0.3.2

The [native process engine](docs/es/native-process-engine.md) shares command and terminal execution across Linux, macOS and Windows. Full application and installer support for Windows/macOS remains pending; engine portability is not a complete cross-platform release.

Version 0.3.2 in development runs commands locally with the OS account's permissions. The multi-user execution sandbox, deployment scripts and read-only command sandbox are retired. The public installer still distributes 0.3.1 until a 0.3.2 release is published.

Deterministic risk checks request a single-use, exact-action approval in chat, answered with yes or no, even under an allow policy. This does not detect all malicious code and is not an isolation boundary. Existing approval preferences are preserved. PDF/image extraction uses an owner-configured local Python interpreter and optional dependencies. UI content protections and Git worktrees remain.

### Filesystem access in personal installations

The workspace is the default project directory, not an access boundary. Agent file tools accept absolute paths, external relative paths and file links with the permissions of the account running Fénix. Search and LSP can target external locations. Risk approvals, secret redaction and optimistic edit checks remain enabled. Text size limits are tool limits, not OS permissions; large or binary files require suitable tools or commands. External access does not grant administrator privileges or expose external files through the web viewer.

### Native filesystem engine

`fenix-files` provides shared read/create/replace/delete, path resolution and directory listing APIs with Unix and Windows implementations. Chat and workspace listings no longer require `/proc`. Expected-content checks and OS permissions remain enforced. Linux execution and cross-compilation must be distinguished from native Windows/macOS validation; other application modules and installers still need platform work. See the [scope and verification guide](docs/es/native-file-engine.md).

### Portable skills and private storage

`fenix-skills` preserves installation, export, editing and deletion without `/proc` paths. Private storage uses file locks, Unix permissions or protected Windows ACLs, retaining SQLite migrations, state and saved translations. See the [scope and verification guide](docs/es/skills-storage.md). Native Windows/macOS execution and complete application support remain pending.

## Owner access in the development version

Before first startup, configure the sole owner with `fenix owner EMAIL --password-stdin [--data-dir PATH]`. Supply the password through standard input, not arguments. Sessions last exactly seven days, survive restarts and do not renew with activity. There is no public registration or multi-account gateway. This functionality is not yet part of public release 0.3.1. See [personal access](docs/en/remote-access.md).

The [portable core](docs/es/platform-core.md) centralizes shutdown and private storage and removes the remote execution transport. Cross-compiling core modules does not imply complete mobile applications or installers for every platform.

## Study the code

Explore the [interactive engine atlas](https://fenix.crehlabs.com/atlas/): backend processes, modules, function search and source links. The inventory is rebuilt with the documentation; explanations describe the development tree and may differ from the installed release.

Source-based Windows and macOS setup is described in the [desktop guide](docs/es/desktop-setup.md). Added installers and CI do not replace native end-to-end validation; the public installer remains Linux-only.
