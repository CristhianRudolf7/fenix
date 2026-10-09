# Fénix

[**Documentation → fenix.crehlabs.com**](https://fenix.crehlabs.com) · [Leer en español](README.md)

**Declared package version: 0.3.0.** An AI assistant with a continuous conversation, background execution, and coding, document, and browser tools. The code supports individual operation and a shared deployment with accounts and isolated user environments. Capabilities depend on configured services and credentials; historical evaluations do not guarantee success on arbitrary tasks.

## Capabilities

- Persistent chat, scoped projects, concurrent tasks, and configurable fast conversational and intelligent execution roles; optional Telegram and separately configured connections.
- Purpose-specific background subagents, one delegation level, follow-up messages, and an available master agent.
- Editable plans, milestones, dependencies, questions, durable asynchronous events, conversational autonomy, and requirement/risk-based verification.
- File tools, interactive terminals, persistent processes, regex/glob search, multi-file patches, configurable LSP, and Git worktrees.
- Scoped memory with provenance, entities, relations, revisions, retrieval, durable context checkpoints, and saved-result search.
- On-demand skill discovery, private explicit-save editing, persisted translations, and selective reviewed learning with recoverable revisions. Publication remains in the backend; the current UI exposes personal skills.
- Chromium profiles, sessions, owner control, idle shutdown, capacity admission, and disk hibernation when a compatible execution service is configured.
- Independent web search using a local index and crawler, plus lightweight HTTP reading; coverage depends on the indexed corpus.
- Private document versions, extraction, chunks, citations, and hybrid search with optional embeddings. Native DOCX/XLSX/PPTX tools and separately provisioned PDF/OCR components.
- Inline design suggestions, modal previews, a right-hand results panel, diagrams, documents, multimedia, and a user-scoped file explorer.
- Cron, reminders, scheduled workflows, connectors, and browser actions.
- Account profiles, recorded token usage, seven-day sessions, password authentication, and Google OAuth support requiring configuration and reserved account capacity.
- Native evaluation and benchmark adapters with recorded outcomes, time, and usage.

## Architecture

Rust/Axum runs the backend. Vite, React, TypeScript, Tailwind, and pnpm power the frontend. Static production deployments use Caddy/systemd independently of the backend, so UI updates do not require rebuilding Rust.

`apps/` contains applications, `crates/` reusable Rust components, `resources/` runtime instructions and assets, `scripts/` operational tooling, `packaging/linux/` deployment templates, and `tests/` cross-component checks.

Root Markdown documentation is Spanish except for this English README. Existing guides under `docs/es/` and `docs/en/` retain their language versions. Runtime skills, prompts, and fixtures retain their operational language.

## Development and individual operation

Install the Rust toolchain pinned in `rust-toolchain.toml`, a Vite-compatible Node version, and pnpm **10.34.5**:

```sh
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo run --locked -p fenix-server --bin fenix
```

The individual server defaults to `127.0.0.1:4173`, with runtime data in `.fenix/`. It accepts `--data-dir`, `--skills-dir`, and `--port`. Provider and conversational role configuration belongs to the installation; source defaults do not imply provider access or dictate shared-service configuration.

For frontend development, `pnpm --dir apps/web dev` serves Vite on port 5173 and proxies `/api` to the individual backend. The shared UI requires the account gateway; individual operation is not a complete multi-user deployment.

`scripts/install-local.sh` retains an individual systemd installation mode. For separate frontend/backend deployment and isolated accounts, follow the Spanish [deployment guide](docs/es/frontend-deployment.md) and [isolation guide](docs/es/user-isolation.md).

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

The public documentation is available at **[fenix.crehlabs.com](https://fenix.crehlabs.com)**. The static site provides local search, responsive navigation, light/dark themes, current guides, and clearly labeled historical references. Build it with `pnpm --dir apps/docs install --frozen-lockfile` and `pnpm --dir apps/docs build`; preview it with `pnpm --dir apps/docs preview`.

`scripts/publish-docs.sh` atomically publishes built files. Adapt `packaging/linux/Caddyfile.docs` to the host before installing it. Caddy serves documentation at `/` and `/docs/`; the existing remote mobile view remains at `/movil.html`. No model calls or API keys are needed to build or search the documentation. A public one-command installer is still planned.
