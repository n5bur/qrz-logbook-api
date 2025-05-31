use thiserror::Error;

/// Result type for QRZ Logbook API operations
pub type QrzLogbookResult<T> = Result<T, QrzLogbookError>;

/// Errors that can occur when using the QRZ Logbook API
#[derive(Error, Debug)]
pub enum QrzLogbookError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response
    #[error("API error: {reason}")]
    Api { reason: String },

    /// Authentication failed or insufficient privileges
    #[error("Authentication failed or insufficient privileges")]
    Auth,

    /// Invalid API key format
    #[error("Invalid API key format")]
    InvalidKey,

    /// Invalid user agent format
    #[error("Invalid user agent: must be 128 characters or less and identifiable")]
    InvalidUserAgent,

    /// ADIF parsing error
    #[error("ADIF parsing error: {0}")]
    AdifParse(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl QrzLogbookError {
    pub fn api_error(reason: impl Into<String>) -> Self {
        Self::Api {
            reason: reason.into(),
        }
    }

    pub fn adif_parse(msg: impl Into<String>) -> Self {
        Self::AdifParse(msg.into())
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }
}
