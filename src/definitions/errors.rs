use reqwest::StatusCode;
use std::path::PathBuf;
use thiserror::Error;

//! # Error Types
//!
//! This module provides a comprehensive set of error types used throughout the gooty-proxy crate.
//! Each error type is designed to encapsulate specific failure modes for different subsystems.
//!
//! ## Overview
//!
//! The module contains several error enums, each focused on a specific domain:
//!
//! - `CidrError`: For errors related to CIDR notation and subnet operations
//! - `RequestorError`: For HTTP request related failures
//! - `FilestoreError`: For disk operations and file management errors
//! - `ConfigError`: For configuration parsing and validation errors
//! - `ProxyError`: For proxy-specific validation and connection errors
//! - `SourceError`: For proxy source acquisition failures
//! - `JudgementError`: For proxy validation and testing errors
//! - `UtilError`: For general utility function failures
//! - `OwnershipError`: For ASN and organization lookup failures
//! - `SleuthError`: For IP investigation failures
//! - `ManagerError`: For high-level proxy management errors
//!
//! Each error type has a corresponding `Result` type alias for more convenient function signatures.
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::definitions::errors::{CidrError, CidrResult};
//!
//! fn parse_cidr(input: &str) -> CidrResult<()> {
//!     if !input.contains('/') {
//!         return Err(CidrError::InvalidFormat("Missing prefix".to_string()));
//!     }
//!     // Additional parsing logic...
//!     Ok(())
//! }
//! ```

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
    /// Indicates that the provided CIDR string does not follow the correct format.
    ///
    /// The format should be IP_ADDRESS/PREFIX_LENGTH, such as "192.168.1.0/24".
    #[error("Invalid CIDR format: {0}")]
    InvalidFormat(String),

    /// Indicates that the IP address portion of a CIDR string is not valid.
    ///
    /// The IP address should be a valid IPv4 or IPv6 address.
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    /// Indicates that the prefix length portion of a CIDR string is not valid.
    ///
    /// For IPv4, the prefix length should be between 0 and 32.
    /// For IPv6, the prefix length should be between 0 and 128.
    #[error("Invalid prefix length: {0}")]
    InvalidPrefixLength(String),

    /// Indicates that there's a mismatch between the IP version and the context.
    ///
    /// This typically occurs when trying to use an IPv4 address in an IPv6 context or vice versa.
    #[error("IP version mismatch")]
    IpVersionMismatch,
}

/// Result type for CIDR operations
pub type CidrResult<T> = Result<T, CidrError>;

/// Error types that can occur during HTTP requests
#[derive(Debug, Error)]
pub enum RequestorError {
    /// Encapsulates an underlying reqwest library error.
    ///
    /// This typically occurs for network-level issues such as DNS failures,
    /// connection problems, or TLS errors.
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Indicates that a request did not complete within the specified timeout period.
    ///
    /// The associated value represents the number of seconds the system waited before timing out.
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    /// Indicates that the server responded with a non-success status code.
    ///
    /// Includes both the status code and any error message from the response body.
    #[error("Server returned status code {0}: {1}")]
    StatusError(StatusCode, String),

    /// Represents errors specific to proxy connection failures.
    ///
    /// This could include authentication failures, connection refused errors,
    /// or other proxy-specific connectivity issues.
    #[error("Proxy connection error: {0}")]
    ProxyError(String),
}

/// Result type for HTTP requests
pub type RequestResult<T> = Result<T, RequestorError>;

/// Errors that can occur in the filestore
#[derive(Debug, Error)]
pub enum FilestoreError {
    /// Represents general I/O errors that occur during file operations.
    ///
    /// This includes file not found errors, permission issues, etc.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Represents errors that occur when serializing data to TOML format.
    ///
    /// This typically happens when data structures contain types that cannot be serialized to TOML.
    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    /// Represents errors that occur when deserializing data from TOML format.
    ///
    /// This can happen when the TOML content doesn't match the expected structure.
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    /// Represents errors that occur when serializing or deserializing JSON data.
    ///
    /// This typically occurs when JSON data doesn't match the expected structure
    /// or when data structures cannot be serialized to valid JSON.
    #[error("JSON serialization error: {0}")]
    JsonSerError(#[from] serde_json::Error),

    /// Indicates that a provided file or directory path is invalid.
    ///
    /// This could be due to path components containing invalid characters,
    /// or paths that are too long for the operating system.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Indicates that creating a directory failed.
    ///
    /// This could be due to permissions, the parent directory not existing,
    /// or the path already exists as a file.
    #[error("Directory creation failed: {0}")]
    DirectoryCreationFailed(String),

    /// Indicates that a requested file could not be found.
    ///
    /// This typically occurs when trying to read from a non-existent file.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Represents errors that occur when parsing file contents.
    ///
    /// This could include syntax errors in configuration files or
    /// invalid data formats.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Represents errors that occur when serializing data to any format.
    ///
    /// This is a general error for serialization issues that aren't specific
    /// to a particular format like TOML or JSON.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for filestore operations
pub type FilestoreResult<T> = Result<T, FilestoreError>;

/// Errors that can occur during configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Encapsulates an underlying I/O error from the standard library.
    ///
    /// This typically occurs when reading from or writing to configuration files.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents errors that occur when serializing data to TOML format.
    ///
    /// This typically happens when configuration data contains types
    /// that cannot be serialized to TOML.
    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    /// Represents errors that occur when deserializing data from TOML format.
    ///
    /// This can happen when the TOML configuration content doesn't match
    /// the expected structure.
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    /// Indicates that a required configuration file was not found.
    ///
    /// The associated `PathBuf` indicates which file was missing.
    #[error("Missing required configuration file: {0}")]
    MissingConfig(PathBuf),

    /// Indicates that a configuration value is invalid or out of acceptable range.
    ///
    /// This could include type mismatches, values out of range, or invalid formats.
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    /// Indicates that a required section in a configuration file is missing.
    ///
    /// Configuration files are often organized into sections, and this error
    /// indicates that a required section was not found.
    #[error("Missing required configuration section: {0}")]
    MissingSection(String),

    /// Indicates that the configuration doesn't match the expected schema.
    ///
    /// This could include missing fields, incorrect data types, or
    /// constraint violations.
    #[error("Configuration schema error: {0}")]
    SchemaError(String),

    /// Indicates that a configuration directory was not found.
    ///
    /// This typically occurs when trying to load multiple configuration
    /// files from a directory that doesn't exist.
    #[error("Configuration directory not found: {0}")]
    DirectoryNotFound(PathBuf),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors that can occur when validating or working with proxies
#[derive(Debug, Error)]
pub enum ProxyError {
    /// Indicates that a port number is invalid.
    ///
    /// Port numbers must be between 0 and 65535, with some ports requiring
    /// special permissions.
    #[error("Invalid port number: {0}")]
    InvalidPort(u16),

    /// Indicates that authentication is required but was not provided.
    ///
    /// Some proxy types require username/password authentication.
    #[error("Missing required authentication for proxy type")]
    MissingAuthentication,

    /// Indicates that the proxy configuration is invalid.
    ///
    /// This could include invalid protocols, malformed URLs, or
    /// incompatible options.
    #[error("Invalid proxy configuration: {0}")]
    InvalidConfiguration(String),

    /// Represents errors that occur when connecting to a proxy.
    ///
    /// This includes connection refused, authentication failures,
    /// or timeouts when trying to establish a connection.
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// Represents an error that can occur when working with proxy sources
#[derive(Debug, Error)]
pub enum SourceError {
    /// Indicates that a URL is invalid or malformed.
    ///
    /// This typically occurs when a proxy source URL doesn't follow RFC 3986.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Indicates that a regular expression pattern is invalid.
    ///
    /// This can occur when extracting proxy information using regex patterns.
    #[error("Invalid regex pattern: {0}")]
    InvalidRegexPattern(String),

    /// Indicates that fetching proxies from a source failed.
    ///
    /// This could be due to network issues, rate limiting, or the source being offline.
    #[error("Failed to fetch from source: {0}")]
    FetchFailure(String),

    /// Indicates that the response from a source couldn't be parsed.
    ///
    /// This typically occurs when a source returns data in an unexpected format.
    #[error("Failed to parse source response: {0}")]
    ParseError(String),
}

/// Result type for source operations
pub type SourceResult<T> = Result<T, SourceError>;

/// Error types that can occur during proxy judgement
#[derive(Debug, Error)]
pub enum JudgementError {
    /// Encapsulates an underlying requestor error.
    ///
    /// This occurs when HTTP requests made during proxy testing fail.
    #[error("Request error: {0}")]
    RequestError(#[from] RequestorError),

    /// Indicates that no judge URL was configured for testing proxies.
    ///
    /// A judge URL is required to verify that proxies are working correctly.
    #[error("Judge URL not configured")]
    NoJudgeUrl,

    /// Indicates that the response from a judge couldn't be parsed.
    ///
    /// This typically occurs when a judge returns data in an unexpected format.
    #[error("Failed to parse judge response: {0}")]
    ParseError(String),

    /// Indicates that a proxy check operation timed out.
    ///
    /// This occurs when a proxy takes too long to respond during testing.
    #[error("Proxy check timed out")]
    Timeout,

    /// Indicates that a proxy check failed.
    ///
    /// This could be due to the proxy being offline, returning incorrect data,
    /// or otherwise not behaving as expected.
    #[error("Proxy check failed: {0}")]
    ProxyFailure(String),

    /// Represents miscellaneous errors that don't fit other categories.
    ///
    /// This is a catch-all for errors that aren't covered by more specific variants.
    #[error("{0}")]
    Other(String),
}

/// Result type for judgement operations
pub type JudgementResult<T> = Result<T, JudgementError>;

/// Error types for utility functions
#[derive(Debug, Error)]
pub enum UtilError {
    /// Indicates that a URL is invalid or malformed.
    ///
    /// This typically occurs when a URL doesn't follow RFC 3986.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Indicates that an IP address is invalid or malformed.
    ///
    /// This can occur when an address doesn't follow IPv4 or IPv6 format.
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    /// Indicates that a port number is invalid.
    ///
    /// Port numbers must be between 0 and 65535.
    #[error("Invalid port number: {0}")]
    InvalidPort(u16),

    /// Indicates that a regular expression pattern is invalid.
    ///
    /// This can occur when constructing regex patterns for various parsing operations.
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),
}

/// Result type for utility functions
pub type UtilResult<T> = Result<T, UtilError>;

/// Error types that can occur during ASN and organization lookups
#[derive(Debug, Error)]
pub enum OwnershipError {
    /// Represents network-related errors during ownership lookups.
    ///
    /// This includes connection failures, DNS issues, or timeouts.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Indicates that the response from an ownership lookup couldn't be parsed.
    ///
    /// This typically occurs when an API returns data in an unexpected format.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Represents errors returned by external APIs during ownership lookups.
    ///
    /// This could include authentication failures or invalid request errors.
    #[error("API error: {0}")]
    ApiError(String),

    /// Indicates that requested ownership information was not found.
    ///
    /// This can occur when looking up ASN/organization information for an IP
    /// that isn't allocated or isn't in the database.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Indicates that requests are being rate-limited by an external API.
    ///
    /// This typically requires waiting before making additional requests.
    #[error("Rate limited")]
    RateLimited,
}

/// Result type for ownership operations
pub type OwnershipResult<T> = Result<T, OwnershipError>;

/// Errors that can occur during IP lookup operations
#[derive(Debug, Error)]
pub enum SleuthError {
    /// Represents network-related errors during IP lookups.
    ///
    /// This includes connection failures, DNS issues, or timeouts.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Indicates that the response from an IP lookup couldn't be parsed.
    ///
    /// This typically occurs when an API returns data in an unexpected format.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Represents errors returned by external APIs during IP lookups.
    ///
    /// This could include authentication failures or invalid request errors.
    #[error("API error: {0}")]
    ApiError(String),

    /// Indicates that requested IP information was not found.
    ///
    /// This can occur when looking up information for an IP that isn't
    /// allocated or isn't in the database.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Indicates that requests are being rate-limited by an external API.
    ///
    /// This typically requires waiting before making additional requests.
    #[error("Rate limited")]
    RateLimited,

    /// Encapsulates an underlying ownership lookup error.
    ///
    /// This occurs when ownership lookup operations fail during IP investigation.
    #[error("Ownership lookup error: {0}")]
    OwnershipError(#[from] OwnershipError),
}

/// Result type for Sleuth operations
pub type SleuthResult<T> = Result<T, SleuthError>;

/// Errors that can occur in the proxy manager
#[derive(Debug, Error)]
pub enum ManagerError {
    /// Encapsulates an underlying proxy validation error.
    ///
    /// This occurs when proxy objects fail validation checks.
    #[error("Proxy validation error: {0}")]
    ProxyError(#[from] ProxyError),

    /// Encapsulates an underlying source error.
    ///
    /// This occurs when operations related to proxy sources fail.
    #[error("Source error: {0}")]
    SourceError(#[from] SourceError),

    /// Encapsulates an underlying judgment error.
    ///
    /// This occurs when proxy testing and validation operations fail.
    #[error("Judgment error: {0}")]
    JudgementError(#[from] JudgementError),

    /// Encapsulates an underlying requestor error.
    ///
    /// This occurs when HTTP requests made during proxy management fail.
    #[error("Requestor error: {0}")]
    RequestorError(#[from] RequestorError),

    /// Encapsulates an underlying sleuth error.
    ///
    /// This occurs when IP investigation operations fail.
    #[error("Sleuth error: {0}")]
    SleuthError(#[from] SleuthError),

    /// Indicates that a proxy ID is invalid or not found in the system.
    ///
    /// This typically occurs when operations reference proxies that don't exist.
    #[error("Invalid proxy ID: {0}")]
    InvalidProxyId(String),

    /// Indicates that a source ID is invalid or not found in the system.
    ///
    /// This typically occurs when operations reference sources that don't exist.
    #[error("Invalid source ID: {0}")]
    InvalidSourceId(String),
}

/// Result type for proxy manager operations
pub type ManagerResult<T> = Result<T, ManagerError>;
