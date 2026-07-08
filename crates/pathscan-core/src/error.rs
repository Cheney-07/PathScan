use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathScanError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("wordlist not found: {0}")]
    WordlistNotFound(String),

    #[error("wordlist read error: {0}")]
    WordlistRead(#[from] std::io::Error),

    #[error("scan cancelled by user")]
    Cancelled,

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PathScanError>;
