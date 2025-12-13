use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("PayPal API error: {message}")]
    Api {
        message: String,
        debug_id: Option<String>,
        details: Vec<ErrorDetail>,
    },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ErrorDetail {
    pub field: Option<String>,
    pub value: Option<String>,
    pub issue: String,
    pub description: Option<String>,
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(field) = &self.field {
            write!(f, "[{}] ", field)?;
        }
        write!(f, "{}", self.issue)?;
        if let Some(desc) = &self.description {
            write!(f, " - {}", desc)?;
        }
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ApiErrorResponse {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub message: Option<String>,
    pub debug_id: Option<String>,
    #[serde(default)]
    pub details: Vec<ErrorDetail>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

impl From<ApiErrorResponse> for Error {
    fn from(resp: ApiErrorResponse) -> Self {
        if let Some(error) = resp.error {
            return Error::Auth(resp.error_description.unwrap_or(error));
        }

        Error::Api {
            message: resp.message.unwrap_or_else(|| "Unknown error".to_string()),
            debug_id: resp.debug_id,
            details: resp.details,
        }
    }
}
