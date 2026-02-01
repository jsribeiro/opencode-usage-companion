use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuotaError {
    #[error("Authentication file not found: {0}")]
    AuthFileNotFound(String),

    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, QuotaError>;
