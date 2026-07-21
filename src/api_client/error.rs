use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiClientError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("unauthorized: no valid token")]
    Unauthorized,

    #[error("server returned error: {0}")]
    Server(String),

    #[error("client is offline, action has been queued")]
    Offline,

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}