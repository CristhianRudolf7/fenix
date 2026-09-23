# Library usage and verification

[Español](../es/development.md)

## Boundaries

`fenix-core` owns serializable conversation contracts. `fenix-credentials` owns
secret values and exact project lookup. `fenix-providers` depends on both and owns
TOML configuration, endpoint selection, HTTP, and protocol conversion. Future agent
loops, channels, memory, tools, and schedulers should consume these contracts from
their own modules rather than being embedded inside provider adapters.

No web interface, interactive CLI, tool execution engine, or provider account
provisioning is included. `ProviderClient::complete` performs one non-streaming
inference call. Authentication uses existing API keys; project keys are created
in the provider's console and then referenced by Fénix.

## Static keys

Create a `CredentialStore`, register a provider credential, then share one client
across tasks. Keys can be supplied through an environment variable or injected
as `ApiKey` values by a future secret manager. Secret values are not serializable,
are redacted in debug output, and are zeroized on drop. HTTP headers are marked
sensitive; error types omit response bodies and credentials. This is not encryption
at rest: persistence of secrets remains the responsibility of the hosting application.

```rust
use fenix_core::{ChatRequest, Message, ProviderId};
use fenix_credentials::{Credential, CredentialStore};
use fenix_providers::{ProviderClient, ProviderConfig};

async fn complete(model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut credentials = CredentialStore::default();
    credentials.set_default(
        ProviderId::OpenAi,
        Credential::environment("OPENAI_API_KEY")?,
    );
    let client = ProviderClient::new(credentials)?;
    let request = ChatRequest::new(model, vec![Message::user("Say hello.")]);
    let response = client.complete(
        &ProviderConfig::new(ProviderId::OpenAi), None, &request,
    ).await?;
    println!("{}", response.message.text_content());
    Ok(())
}
```

Model IDs are explicit inputs: the library does not assume a model is enabled on
an account or silently switch providers. A tool-capable model is required when
the request includes tools. Supplying `temperature` remains optional because some
reasoning models do not accept it. Token limits also follow each selected model's
constraints.

## Project keys and configuration

Copy `config/providers.example.toml` to a file ending in `.local.toml`, which Git
ignores. Replace Azure resource and Bedrock region examples where applicable.
Set only the environment variables for the providers you intend to call. Missing
variables do not prevent loading configuration; they produce an error when used.

```rust
use fenix_core::{ChatRequest, Message, ProviderId};
use fenix_providers::{ProviderClient, ProvidersConfig};

async fn complete_for_project(model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string("config/providers.local.toml")?;
    let config = ProvidersConfig::from_toml(&text)?;
    let client = ProviderClient::new(config.credential_store()?)?;
    let project = Some("demo-project");
    let endpoint = config.provider(ProviderId::OpenAi, project)?;
    let request = ChatRequest::new(model, vec![Message::user("Say hello.")]);
    let response = client.complete(&endpoint, project, &request).await?;
    println!("{}", response.message.text_content());
    Ok(())
}
```

Pass `None` only for an intentionally unscoped request. `Some("demo-project")`
requires that project's credential for the chosen provider; it never falls back
to another project or the default key. A project's `providers` list can override
resource endpoints. The same project must be passed to both endpoint lookup and
the inference call. `ProviderClient` is a trusted library API, not an authorization
boundary for untrusted users; the future application must decide which project
a user may request and which endpoint configurations they may change.

`provider_project` and `organization` describe native provider headers; they are
independent of Fénix's project ID. OpenAI supports both, and Anthropic supports a
workspace ID through `provider_project`. Other providers bind keys to accounts,
projects, workspaces, or resources at issuance. Unsupported headers fail explicitly.
See [the provider reference](provider-reference.md) for exact scope semantics.

## Tool calls and conversation replay

Declare available functions with `ToolDefinition`, including a name, description,
and JSON Schema object. The result may contain `ContentBlock::ToolCall` instead
of final text. The calling application validates permissions and arguments,
executes the operation, appends the returned assistant `Message` unchanged, and
appends a `Role::Tool` message with matching `ContentBlock::ToolResult` IDs.

Preserve `Message::provider_data` when storing or replaying assistant messages.
It retains provider-specific reasoning and signatures needed for continued tool
use. Editing visible content without clearing stale metadata, or replaying it
through another protocol, returns an error. Switching providers/models can require
rebuilding history, even when two providers use the same wire protocol.

Returned `FinishReason::Length` is partial output, not task completion.
`ContentFilter` indicates filtered/refused output. Unknown token counts remain
`None`. Anthropic input includes reported cache tokens; Gemini output includes
reported thought tokens. Counts are not prices: billing rules differ by model.

## Transport behavior

The shared client uses Rustls TLS with certificate validation and permits plain
HTTP only for loopback endpoints, including local Ollama and tests. Set
`ClientOptions::allow_http_loopback` to false when HTTPS is required everywhere.
Redirects are not followed, preventing automatic forwarding to another endpoint.
Base URLs cannot contain embedded credentials, query parameters, or fragments.

Timeouts, request sizes, and response sizes are configurable. There are no automatic
inference retries. Errors distinguish authentication/authorization, rate limiting,
timeouts, transport failures, provider HTTP errors, malformed output, and size
limits. `is_retryable()` is a scheduling hint; it does not authorize replaying
tool effects. `RateLimited` carries `Retry-After` when expressed in seconds.
Dropping the request future cancels local I/O but cannot undo provider processing
or charges that have already occurred.

## Local checks

```sh
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The HTTP suite exercises all 17 integrations with fake static and project keys,
concurrent project isolation, native scope headers, protocol round trips,
timeouts, redirects, malformed responses, and bounded bodies including chunked
responses. These tests make no billable model calls.

## Optional live check

Configure a provider's documented key environment variable using your secret
management workflow. Set `FENIX_LIVE_PROVIDER` to an ID from the provider reference
and `FENIX_LIVE_MODEL` to a model/deployment available to your account, then run:

```sh
cargo test -p fenix-providers --test live --locked -- --ignored
```

Optional settings:

| Variable | Purpose |
| --- | --- |
| `FENIX_LIVE_BASE_URL` | Required resource/region URL for Azure and Bedrock; optional override elsewhere. |
| `FENIX_LIVE_PROVIDER_PROJECT` | Native OpenAI project or Anthropic workspace ID. |
| `FENIX_LIVE_ORGANIZATION` | Native OpenAI organization ID. |
| `FENIX_LIVE_PROJECT` | Exercise a Fénix project credential scope. |
| `FENIX_LIVE_MAX_OUTPUT_TOKENS` | Positive output budget; raise it for reasoning models that need more tokens. |

This sends one real inference request and checks for visible response text. It
uses the selected provider's default key variable and does not read the example
TOML file. Run it separately for each account/model you need to verify. A key with
the wrong region, endpoint, permissions, quota, or model access can still fail.
The test is ignored by the default suite and no live results are implied by a
successful local build.
