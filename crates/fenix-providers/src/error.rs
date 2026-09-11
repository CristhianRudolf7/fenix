use crate::protocol::CodecError;
use fenix_credentials::CredentialError;
use std::time::Duration;
use thiserror::Error;

/// Errors intentionally omit HTTP response bodies, URLs, and secret headers.
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error(transparent)]
    Credential(#[from] CredentialError),
    #[error("invalid provider configuration: {0}")]
    InvalidConfiguration(&'static str),
    #[error("invalid model request: {0}")]
    InvalidRequest(&'static str),
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error("provider rejected authentication or authorization (HTTP {status})")]
    Authentication { status: u16 },
    #[error("provider rate limit reached")]
    RateLimited { retry_after: Option<Duration> },
    #[error("provider request failed (HTTP {status})")]
    Http { status: u16 },
    #[error("provider request timed out")]
    Timeout,
    #[error("provider connection failed")]
    Transport,
    #[error("provider response exceeded the {limit}-byte limit")]
    ResponseTooLarge { limit: usize },
    #[error("provider request exceeded the {limit}-byte limit")]
    RequestTooLarge { limit: usize },
    #[error("provider returned an invalid JSON response")]
    MalformedResponse,
}

impl ProviderError {
    /// A scheduling hint, not permission to replay a task or its tool effects.
    /// ProviderClient never automatically repeats an inference request.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. } | Self::Timeout | Self::Transport => true,
            Self::Http { status } => matches!(status, 408 | 500 | 502 | 503 | 504 | 529),
            _ => false,
        }
    }
}
