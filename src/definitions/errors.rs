use reqwest::StatusCode;
use std::path::PathBuf;
use thiserror::Error;

/// Represents error types that can occur during CIDR operations.
///
/// This enum provides detailed error variants for invalid CIDR formats,
/// IP address issues, and prefix length validation.
///
/// # Examples
///
/// ```
/// use gooty_proxy::definitions::errors::CidrError;
///
/// let error = CidrError::InvalidFormat("Invalid CIDR".to_string());
/// println!("Error: {}", error);
/// ```
#[derive(Debug, Error)]
pub enum CidrError {
    #[error("Invalid CIDR format: {0}")]
    InvalidFormat(String),

    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    #[error("Invalid prefix length: {0}")]
    InvalidPrefixLength(String),

    #[error("IP version mismatch")]
    IpVersionMismatch,
}

/// Result type for CIDR operations
pub type CidrResult<T> = Result<T, CidrError>;

/// Error types that can occur during HTTP requests
#[derive(Debug, Error)]
pub enum RequestorError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    #[error("Server returned status code {0}: {1}")]
    StatusError(StatusCode, String),

    #[error("Proxy connection error: {0}")]
    ProxyError(String),
}

/// Result type for HTTP requests
pub type RequestResult<T> = Result<T, RequestorError>;

/// Errors that can occur in the filestore
#[derive(Debug, Error)]
pub enum FilestoreError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("JSON serialization error: {0}")]
    JsonSerError(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Directory creation failed: {0}")]
    DirectoryCreationFailed(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for filestore operations
pub type FilestoreResult<T> = Result<T, FilestoreError>;

/// Errors that can occur during configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("Missing required configuration file: {0}")]
    MissingConfig(PathBuf),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Missing required configuration section: {0}")]
    MissingSection(String),

    #[error("Configuration schema error: {0}")]
    SchemaError(String),

    #[error("Configuration directory not found: {0}")]
    DirectoryNotFound(PathBuf),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors that can occur when validating or working with proxies
#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("Invalid port number: {0}")]
    InvalidPort(u16),

    #[error("Missing required authentication for proxy type")]
    MissingAuthentication,

    #[error("Invalid proxy configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// Represents an error that can occur when working with proxy sources
#[derive(Debug, Error)]
pub enum SourceError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidRegexPattern(String),

    #[error("Failed to fetch from source: {0}")]
    FetchFailure(String),

    #[error("Failed to parse source response: {0}")]
    ParseError(String),
}

/// Result type for source operations
pub type SourceResult<T> = Result<T, SourceError>;

/// Error types that can occur during proxy judgement
#[derive(Debug, Error)]
pub enum JudgementError {
    #[error("Request error: {0}")]
    RequestError(#[from] RequestorError),

    #[error("Judge URL not configured")]
    NoJudgeUrl,

    #[error("Failed to parse judge response: {0}")]
    ParseError(String),

    #[error("Proxy check timed out")]
    Timeout,

    #[error("Proxy check failed: {0}")]
    ProxyFailure(String),

    #[error("{0}")]
    Other(String),
}

/// Result type for judgement operations
pub type JudgementResult<T> = Result<T, JudgementError>;

/// Error types for utility functions
#[derive(Debug, Error)]
pub enum UtilError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    #[error("Invalid port number: {0}")]
    InvalidPort(u16),

    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),
}

/// Result type for utility functions
pub type UtilResult<T> = Result<T, UtilError>;

/// Error types that can occur during ASN and organization lookups
#[derive(Debug, Error)]
pub enum OwnershipError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limited")]
    RateLimited,
}

/// Result type for ownership operations
pub type OwnershipResult<T> = Result<T, OwnershipError>;

/// Errors that can occur during IP lookup operations
#[derive(Debug, Error)]
pub enum SleuthError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Ownership lookup error: {0}")]
    OwnershipError(#[from] OwnershipError),
}

/// Result type for Sleuth operations
pub type SleuthResult<T> = Result<T, SleuthError>;

/// Errors that can occur in the proxy manager
#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Proxy validation error: {0}")]
    ProxyError(#[from] ProxyError),

    #[error("Source error: {0}")]
    SourceError(#[from] SourceError),

    #[error("Judgment error: {0}")]
    JudgementError(#[from] JudgementError),

    #[error("Requestor error: {0}")]
    RequestorError(#[from] RequestorError),

    #[error("Sleuth error: {0}")]
    SleuthError(#[from] SleuthError),

    #[error("Invalid proxy ID: {0}")]
    InvalidProxyId(String),

    #[error("Invalid source ID: {0}")]
    InvalidSourceId(String),
}

/// Result type for proxy manager operations
pub type ManagerResult<T> = Result<T, ManagerError>;
