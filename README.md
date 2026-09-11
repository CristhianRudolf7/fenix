# Fénix

[Español](README.es.md)

Fénix is a Rust foundation for a personal AI assistant. The current implementation
provides model inference and isolated API-key credentials for 17 provider integrations.
The assistant runtime, memory, task scheduler, Telegram, voice, and administration
interface are defined in the requirements and roadmap and will build on these libraries.

## Implemented

- Static keys and exact project-scoped credential selection for OpenAI, Anthropic,
  Gemini, OpenRouter, DeepSeek, xAI, Mistral, Alibaba/Qwen, Moonshot/Kimi, Z.ai,
  MiniMax, Groq, Together AI, Bedrock, Azure, Vertex express mode, and Ollama.
- Text inference, tool definitions, structured tool calls/results, token usage,
  and replay metadata for five native or compatible protocols.
- Shared async HTTP connection pools, timeouts, cancellation by dropping the request
  future, bounded request/response bodies, and classified errors.
- TOML configuration containing environment-variable references rather than keys.

`complete()` sends one non-streaming inference request. The library returns tool
calls to its caller; tool execution belongs to the future assistant runtime.

## Workspace

```text
crates/
  fenix-core/          Conversation types and provider identifiers
  fenix-credentials/   Secret handling and project credential resolution
  fenix-providers/     Provider catalog, configuration, protocols, HTTP, tests
config/               Configuration examples without secrets
docs/
  en/                 Primary documentation
  es/                 Spanish documentation
```

These crates do not depend on a frontend or messaging channel. Future runtimes
can share a `ProviderClient` across tasks, and persistence can store conversation
types without accessing credentials. No per-agent operating-system process is required.

## Development

Install [Rust using rustup](https://rust-lang.org/tools/install/). The workspace pins
its toolchain in `rust-toolchain.toml` and dependencies in `Cargo.lock`.

```sh
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The default tests use local HTTP servers and fake credentials. The live test is
ignored unless explicitly selected; it requires a real key and model and may incur
provider charges. Passing local tests does not establish remote account access.

- [Library usage and verification](docs/en/development.md)
- [Provider endpoints, authentication, and official sources](docs/en/provider-reference.md)
- [Configuration example](config/providers.example.toml)
- [Current requirements](REQUIREMENTS.md)
- [Future roadmap](ROADMAP.md)
