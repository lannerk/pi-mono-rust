use std::fmt;
use std::io;

use thiserror::Error;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] ReqwestError),

    #[error("JSON error: {0}")]
    Json(#[from] SerdeJsonError),

    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("API error: {0} - {1}")]
    ApiError(u16, String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Unsupported provider type: {0}")]
    UnsupportedProviderType(String),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),

    #[error("Invalid header name: {0}")]
    InvalidHeaderName(String),
}

pub type Result<T> = std::result::Result<T, Error>;
