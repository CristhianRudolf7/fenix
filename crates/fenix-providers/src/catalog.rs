//! Provider endpoints and authentication capabilities, independent of credentials.
//!
//! Sources and deployment restrictions are maintained in
//! `docs/en/provider-reference.md` and its Spanish translation.

use fenix_core::{Protocol, ProviderId};

/// How an inference endpoint receives a static API key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStyle {
    Bearer,
    ApiKey(&'static str),
}

/// Transport defaults and documented native account-scoping headers.
///
/// A missing project header does not prevent Fénix from selecting a separate
/// credential per internal project. It means no native project-selection header
/// is implemented for this endpoint. Providers can scope keys at creation time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderDescriptor {
    pub id: ProviderId,
    pub protocol: Protocol,
    /// API root including its version, excluding the inference operation.
    /// Resource-specific services require an explicit configured URL.
    pub default_base_url: Option<&'static str>,
    pub api_key_env: &'static str,
    pub auth: AuthStyle,
    pub project_header: Option<&'static str>,
    pub organization_header: Option<&'static str>,
}

/// Return the verified defaults for a supported provider.
pub fn descriptor(id: ProviderId) -> ProviderDescriptor {
    let (protocol, default_base_url, api_key_env, auth) = match id {
        ProviderId::OpenAi => (
            Protocol::OpenAiResponses,
            Some("https://api.openai.com/v1"),
            "OPENAI_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Anthropic => (
            Protocol::Anthropic,
            Some("https://api.anthropic.com/v1"),
            "ANTHROPIC_API_KEY",
            AuthStyle::ApiKey("x-api-key"),
        ),
        ProviderId::Gemini => (
            Protocol::Gemini,
            Some("https://generativelanguage.googleapis.com/v1beta"),
            "GEMINI_API_KEY",
            AuthStyle::ApiKey("x-goog-api-key"),
        ),
        ProviderId::OpenRouter => (
            Protocol::OpenAiChat,
            Some("https://openrouter.ai/api/v1"),
            "OPENROUTER_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::DeepSeek => (
            Protocol::OpenAiChat,
            Some("https://api.deepseek.com"),
            "DEEPSEEK_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Xai => (
            Protocol::OpenAiChat,
            Some("https://api.x.ai/v1"),
            "XAI_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Mistral => (
            Protocol::OpenAiChat,
            Some("https://api.mistral.ai/v1"),
            "MISTRAL_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Alibaba => (
            Protocol::OpenAiChat,
            Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1"),
            "DASHSCOPE_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Moonshot => (
            Protocol::OpenAiChat,
            Some("https://api.moonshot.ai/v1"),
            "MOONSHOT_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Zai => (
            Protocol::OpenAiChat,
            Some("https://api.z.ai/api/paas/v4"),
            "ZAI_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::MiniMax => (
            Protocol::OpenAiChat,
            Some("https://api.minimax.io/v1"),
            "MINIMAX_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Groq => (
            Protocol::OpenAiChat,
            Some("https://api.groq.com/openai/v1"),
            "GROQ_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Together => (
            Protocol::OpenAiChat,
            Some("https://api.together.ai/v1"),
            "TOGETHER_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Bedrock => (
            Protocol::Bedrock,
            None,
            "AWS_BEARER_TOKEN_BEDROCK",
            AuthStyle::Bearer,
        ),
        ProviderId::Azure => (
            Protocol::OpenAiChat,
            None,
            "AZURE_API_KEY",
            AuthStyle::Bearer,
        ),
        ProviderId::Vertex => (
            Protocol::Gemini,
            Some("https://aiplatform.googleapis.com/v1/publishers/google"),
            "VERTEX_API_KEY",
            AuthStyle::ApiKey("x-goog-api-key"),
        ),
        ProviderId::Ollama => (
            Protocol::OpenAiChat,
            Some("http://localhost:11434/v1"),
            "OLLAMA_API_KEY",
            AuthStyle::Bearer,
        ),
    };

    let project_header = match id {
        ProviderId::OpenAi => Some("OpenAI-Project"),
        ProviderId::Anthropic => Some("anthropic-workspace-id"),
        _ => None,
    };
    let organization_header = match id {
        ProviderId::OpenAi => Some("OpenAI-Organization"),
        _ => None,
    };

    ProviderDescriptor {
        id,
        protocol,
        default_base_url,
        api_key_env,
        auth,
        project_header,
        organization_header,
    }
}
