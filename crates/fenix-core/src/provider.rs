use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Stable identifiers used in configuration and credential scopes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "xai")]
    Xai,
    #[serde(rename = "mistral")]
    Mistral,
    #[serde(rename = "alibaba")]
    Alibaba,
    #[serde(rename = "moonshot")]
    Moonshot,
    #[serde(rename = "zai")]
    Zai,
    #[serde(rename = "minimax")]
    MiniMax,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "together")]
    Together,
    #[serde(rename = "bedrock")]
    Bedrock,
    #[serde(rename = "azure")]
    Azure,
    #[serde(rename = "vertex")]
    Vertex,
    #[serde(rename = "ollama")]
    Ollama,
}

impl ProviderId {
    pub const ALL: [Self; 17] = [
        Self::OpenAi,
        Self::Anthropic,
        Self::Gemini,
        Self::OpenRouter,
        Self::DeepSeek,
        Self::Xai,
        Self::Mistral,
        Self::Alibaba,
        Self::Moonshot,
        Self::Zai,
        Self::MiniMax,
        Self::Groq,
        Self::Together,
        Self::Bedrock,
        Self::Azure,
        Self::Vertex,
        Self::Ollama,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::OpenRouter => "openrouter",
            Self::DeepSeek => "deepseek",
            Self::Xai => "xai",
            Self::Mistral => "mistral",
            Self::Alibaba => "alibaba",
            Self::Moonshot => "moonshot",
            Self::Zai => "zai",
            Self::MiniMax => "minimax",
            Self::Groq => "groq",
            Self::Together => "together",
            Self::Bedrock => "bedrock",
            Self::Azure => "azure",
            Self::Vertex => "vertex",
            Self::Ollama => "ollama",
        }
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    type Err = ParseProviderError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|id| id.as_str() == value)
            .ok_or(ParseProviderError)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseProviderError;

impl fmt::Display for ParseProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown model provider")
    }
}

impl std::error::Error for ParseProviderError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    OpenAiResponses,
    OpenAiChat,
    Anthropic,
    Gemini,
    Bedrock,
}
