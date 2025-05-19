use std::fmt;

/// Errors that can occur when using the Gemini API client.
#[derive(Debug)]
pub enum GeminiError {
    /// Error related to configuration loading or validation.
    ConfigurationError(String),

    /// Error related to API requests.
    ApiError(String),

    /// Error related to JSON serialization or deserialization.
    SerializationError(String),

    /// Error related to HTTP requests.
    HttpError(String),

    /// Error related to rate limiting.
    RateLimitError(String),

    /// Other errors.
    Other(String),
}

impl fmt::Display for GeminiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeminiError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            GeminiError::ApiError(msg) => write!(f, "API error: {}", msg),
            GeminiError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            GeminiError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            GeminiError::RateLimitError(msg) => write!(f, "Rate limit error: {}", msg),
            GeminiError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for GeminiError {}

impl From<reqwest::Error> for GeminiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GeminiError::HttpError(format!("Request timed out: {}", err))
        } else if err.is_connect() {
            GeminiError::HttpError(format!("Connection error: {}", err))
        } else if err.status().is_some_and(|s| s.as_u16() == 429) {
            GeminiError::RateLimitError(format!("Rate limit exceeded: {}", err))
        } else {
            GeminiError::HttpError(format!("HTTP error: {}", err))
        }
    }
}

impl From<serde_json::Error> for GeminiError {
    fn from(err: serde_json::Error) -> Self {
        GeminiError::SerializationError(format!("JSON error: {}", err))
    }
}
