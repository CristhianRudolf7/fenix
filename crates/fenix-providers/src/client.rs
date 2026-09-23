use crate::{
    ProviderError,
    catalog::{AuthStyle, descriptor},
    protocol,
};
use fenix_core::{ChatRequest, ChatResponse, ContentBlock, Protocol, ProviderId, Role};
use fenix_credentials::{CredentialError, CredentialStore, ResolvedCredential};
use reqwest::{
    Url,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::IpAddr, sync::Arc, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConfig {
    pub provider: ProviderId,
    #[serde(default)]
    pub base_url: Option<String>,
}

impl ProviderConfig {
    pub fn new(provider: ProviderId) -> Self {
        Self {
            provider,
            base_url: None,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct ClientOptions {
    pub timeout: Duration,
    /// Disable the total request deadline for externally supervised evaluations.
    pub disable_request_timeout: bool,
    pub connect_timeout: Duration,
    pub max_response_bytes: usize,
    pub max_request_bytes: usize,
    /// Only literal loopback addresses and localhost may use plain HTTP.
    pub allow_http_loopback: bool,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(120),
            disable_request_timeout: false,
            connect_timeout: Duration::from_secs(10),
            max_response_bytes: 8 * 1024 * 1024,
            max_request_bytes: 8 * 1024 * 1024,
            allow_http_loopback: true,
        }
    }
}

/// Cheap clones share the connection pool and credential registry across tasks.
#[derive(Clone)]
pub struct ProviderClient {
    http: reqwest::Client,
    credentials: Arc<CredentialStore>,
    options: ClientOptions,
    redactor: Arc<fenix_core::safety::Redactor>,
}

impl ProviderClient {
    pub fn new(credentials: CredentialStore) -> Result<Self, ProviderError> {
        Self::with_options(credentials, ClientOptions::default())
    }

    pub fn with_options(
        credentials: CredentialStore,
        options: ClientOptions,
    ) -> Result<Self, ProviderError> {
        if options.timeout.is_zero()
            || options.connect_timeout.is_zero()
            || options.max_response_bytes == 0
            || options.max_request_bytes == 0
        {
            return Err(ProviderError::InvalidConfiguration(
                "timeouts and byte limits must be positive",
            ));
        }
        let mut builder = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(options.connect_timeout)
            .user_agent(concat!("fenix/", env!("CARGO_PKG_VERSION")));
        if !options.disable_request_timeout {
            builder = builder.timeout(options.timeout);
        }
        let http = builder.build().map_err(|_| ProviderError::Transport)?;
        Ok(Self {
            http,
            credentials: Arc::new(credentials),
            options,
            redactor: Arc::new(fenix_core::safety::Redactor::default()),
        })
    }

    /// Share only this account's registered credentials across model boundaries.
    pub fn with_redactor(mut self, redactor: Arc<fenix_core::safety::Redactor>) -> Self {
        self.redactor = redactor;
        self
    }

    /// One inference request, including text and structured tool calls/results.
    /// Dropping this future cancels local I/O; the provider may already have billed
    /// or processed it. This method does not execute tools or retry requests.
    pub async fn complete(
        &self,
        config: &ProviderConfig,
        project: Option<&str>,
        request: &ChatRequest,
    ) -> Result<ChatResponse, ProviderError> {
        self.complete_inner(config, project, request, None).await
    }

    /// Preview visible text as it arrives. OpenAI Responses currently supports
    /// streaming; other adapters retain their existing single-request behavior.
    /// Callbacks are provisional: only the returned response may enter history
    /// or supply executable tool calls. No automatic non-streaming retry occurs.
    pub async fn complete_with_preview(
        &self,
        config: &ProviderConfig,
        project: Option<&str>,
        request: &ChatRequest,
        preview: &mut (dyn FnMut(&str) + Send),
    ) -> Result<ChatResponse, ProviderError> {
        self.complete_inner(config, project, request, Some(preview))
            .await
    }

    async fn complete_inner(
        &self,
        config: &ProviderConfig,
        project: Option<&str>,
        request: &ChatRequest,
        mut preview: Option<&mut (dyn FnMut(&str) + Send)>,
    ) -> Result<ChatResponse, ProviderError> {
        validate_request(request)?;
        let provider = descriptor(config.provider);
        let base_url = config
            .base_url
            .as_deref()
            .or(provider.default_base_url)
            .ok_or(ProviderError::InvalidConfiguration(
                "this provider requires an explicit regional or resource base URL",
            ))?;
        let url = endpoint(
            base_url,
            provider.protocol,
            &request.model,
            self.options.allow_http_loopback,
        )?;
        let credential = self
            .credentials
            .resolve_optional(config.provider, project)?;
        if credential.is_none() && config.provider != ProviderId::Ollama {
            return Err(CredentialError::MissingCredential {
                provider: config.provider,
            }
            .into());
        }
        if let Some(key) = &credential {
            self.redactor.add(key.key.expose_secret());
        }
        let mut safe_request = request.clone();
        for message in &mut safe_request.messages {
            self.redactor
                .message(message)
                .map_err(ProviderError::InvalidRequest)?;
        }
        for tool in &mut safe_request.tools {
            tool.description = self.redactor.text(&tool.description);
            self.redactor.value(&mut tool.parameters);
        }
        let request = &safe_request;
        let streaming = preview.is_some() && provider.protocol == Protocol::OpenAiResponses;
        let mut headers = auth_headers(config.provider, credential.as_ref())?;
        let mut body = protocol::encode(provider.protocol, request)?;
        if streaming {
            body["stream"] = true.into();
            headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
        }
        if config.provider == ProviderId::Azure {
            // Azure v1 follows current OpenAI Chat Completions: reasoning models
            // reject the legacy max_tokens field used by other compatible APIs.
            if let Some(object) = body.as_object_mut()
                && let Some(limit) = object.remove("max_tokens")
            {
                object.insert("max_completion_tokens".into(), limit);
            }
        }
        let body = serde_json::to_vec(&body)
            .map_err(|_| ProviderError::InvalidRequest("request cannot be encoded as JSON"))?;
        if body.len() > self.options.max_request_bytes {
            return Err(ProviderError::RequestTooLarge {
                limit: self.options.max_request_bytes,
            });
        }
        let mut response = self
            .http
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(transport_error)?;
        let status = response.status();
        if !status.is_success() {
            // Never echo arbitrary provider error bodies: they may contain keys,
            // prompts, or internal diagnostics. Callers receive classified errors.
            return Err(match status.as_u16() {
                401 | 403 => ProviderError::Authentication {
                    status: status.as_u16(),
                },
                429 => ProviderError::RateLimited {
                    retry_after: response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|header| header.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(Duration::from_secs),
                },
                status => ProviderError::Http { status },
            });
        }
        if response
            .content_length()
            .is_some_and(|length| length > self.options.max_response_bytes as u64)
        {
            return Err(ProviderError::ResponseTooLarge {
                limit: self.options.max_response_bytes,
            });
        }
        if streaming
            && response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.split(';').next().unwrap_or("").trim() == "text/event-stream")
        {
            let callback = preview
                .as_mut()
                .expect("streaming requires a preview callback");
            let mut pending = String::new();
            let mut safe_preview = |delta: &str| {
                pending.push_str(delta);
                let (consumed, safe) = self.redactor.stream_prefix(&pending, false);
                if consumed > 0 {
                    pending.drain(..consumed);
                    if !safe.is_empty() {
                        callback(&safe);
                    }
                }
            };
            let mut result = crate::streaming::read_responses(
                response,
                self.options.max_response_bytes,
                &mut safe_preview,
            )
            .await?;
            self.redactor
                .message(&mut result.message)
                .map_err(ProviderError::InvalidRequest)?;
            if !pending.is_empty() {
                callback(&self.redactor.text(&pending));
            }
            return Ok(result);
        }
        // Some compatible gateways return complete JSON even when streaming was
        // requested. Decode that same response without making another request.
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
            if chunk.len() > self.options.max_response_bytes.saturating_sub(bytes.len()) {
                return Err(ProviderError::ResponseTooLarge {
                    limit: self.options.max_response_bytes,
                });
            }
            bytes.extend_from_slice(&chunk);
        }
        let value = serde_json::from_slice(&bytes).map_err(|_| ProviderError::MalformedResponse)?;
        let mut result = protocol::decode(provider.protocol, value)?;
        self.redactor
            .message(&mut result.message)
            .map_err(ProviderError::InvalidRequest)?;
        Ok(result)
    }
}

pub(crate) fn transport_error(error: reqwest::Error) -> ProviderError {
    if error.is_timeout() {
        ProviderError::Timeout
    } else {
        ProviderError::Transport
    }
}

fn endpoint(
    base_url: &str,
    protocol: Protocol,
    model: &str,
    allow_http_loopback: bool,
) -> Result<Url, ProviderError> {
    let mut url = Url::parse(base_url)
        .map_err(|_| ProviderError::InvalidConfiguration("invalid base URL"))?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ProviderError::InvalidConfiguration(
            "base URL must not contain credentials, a query, or a fragment",
        ));
    }
    let local = url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .trim_matches(['[', ']'])
                .parse::<IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if url.scheme() != "https" && !(url.scheme() == "http" && allow_http_loopback && local) {
        return Err(ProviderError::InvalidConfiguration(
            "HTTPS is required except for enabled loopback HTTP endpoints",
        ));
    }
    {
        let mut segments = url.path_segments_mut().map_err(|_| {
            ProviderError::InvalidConfiguration("base URL cannot have path segments")
        })?;
        segments.pop_if_empty();
        match protocol {
            Protocol::OpenAiResponses => {
                segments.push("responses");
            }
            Protocol::OpenAiChat => {
                segments.push("chat").push("completions");
            }
            Protocol::Anthropic => {
                segments.push("messages");
            }
            Protocol::Gemini => {
                let model = model.strip_prefix("models/").unwrap_or(model);
                if model.is_empty() {
                    return Err(ProviderError::InvalidRequest("model is empty"));
                }
                segments
                    .push("models")
                    .push(&format!("{model}:generateContent"));
            }
            Protocol::Bedrock => {
                segments.push("model").push(model).push("converse");
            }
        }
    }
    Ok(url)
}

fn auth_headers(
    provider_id: ProviderId,
    credential: Option<&ResolvedCredential>,
) -> Result<HeaderMap, ProviderError> {
    let provider = descriptor(provider_id);
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    if provider.protocol == Protocol::Anthropic {
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    }
    if let Some(credential) = credential {
        let (name, mut value) = match provider.auth {
            AuthStyle::Bearer => (
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", credential.key.expose_secret())),
            ),
            AuthStyle::ApiKey(name) => (
                HeaderName::from_static(name),
                HeaderValue::from_str(credential.key.expose_secret()),
            ),
        };
        let value = value.as_mut().map_err(|_| {
            ProviderError::InvalidConfiguration("API key cannot be sent as an HTTP header")
        })?;
        value.set_sensitive(true);
        headers.insert(name, value.clone());
        for (configured, supported, error) in [
            (
                credential.provider_project.as_deref(),
                provider.project_header,
                "this provider does not support an explicit project/workspace header; scope its key at issuance",
            ),
            (
                credential.organization.as_deref(),
                provider.organization_header,
                "this provider does not support an explicit organization header",
            ),
        ] {
            if let Some(value) = configured {
                let header = supported.ok_or(ProviderError::InvalidConfiguration(error))?;
                if value.is_empty() {
                    return Err(ProviderError::InvalidConfiguration(
                        "provider scope header must not be empty",
                    ));
                }
                let mut value = HeaderValue::from_str(value).map_err(|_| {
                    ProviderError::InvalidConfiguration("invalid provider scope header")
                })?;
                value.set_sensitive(true);
                headers.insert(
                    HeaderName::from_bytes(header.as_bytes()).map_err(|_| {
                        ProviderError::InvalidConfiguration("invalid provider scope header name")
                    })?,
                    value,
                );
            }
        }
    }
    Ok(headers)
}

fn validate_request(request: &ChatRequest) -> Result<(), ProviderError> {
    if request.model.trim().is_empty()
        || matches!(request.model.as_str(), "." | "..")
        || request.model.len() > 2048
        || request.model.chars().any(char::is_control)
    {
        return Err(ProviderError::InvalidRequest(
            "model must be a nonempty identifier of at most 2048 bytes",
        ));
    }
    if request.max_output_tokens == 0 {
        return Err(ProviderError::InvalidRequest(
            "max_output_tokens must be positive",
        ));
    }
    if request
        .temperature
        .is_some_and(|v| !v.is_finite() || !(0.0..=2.0).contains(&v))
    {
        return Err(ProviderError::InvalidRequest(
            "temperature must be finite and between 0 and 2",
        ));
    }
    if request.messages.is_empty() {
        return Err(ProviderError::InvalidRequest("messages must not be empty"));
    }
    let mut names = HashSet::new();
    for tool in &request.tools {
        if tool.name.is_empty()
            || tool.name.len() > 64
            || !tool
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        {
            return Err(ProviderError::InvalidRequest(
                "tool names must contain 1–64 ASCII letters, digits, underscores, or hyphens",
            ));
        }
        if !names.insert(&tool.name) {
            return Err(ProviderError::InvalidRequest("tool names must be unique"));
        }
        if !tool.parameters.is_object() {
            return Err(ProviderError::InvalidRequest(
                "tool parameters must be a JSON Schema object",
            ));
        }
    }
    for message in &request.messages {
        if message.content.is_empty() && message.provider_data.is_none() {
            return Err(ProviderError::InvalidRequest("message content is empty"));
        }
        for block in &message.content {
            match block {
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                } => {
                    if message.role != Role::Assistant
                        || id.is_empty()
                        || name.is_empty()
                        || !arguments.is_object()
                    {
                        return Err(ProviderError::InvalidRequest(
                            "tool calls require an assistant role, ID, name, and object arguments",
                        ));
                    }
                }
                ContentBlock::ToolResult { tool_call_id, .. }
                    if message.role != Role::Tool || tool_call_id.is_empty() =>
                {
                    return Err(ProviderError::InvalidRequest(
                        "tool results require a tool role and call ID",
                    ));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
