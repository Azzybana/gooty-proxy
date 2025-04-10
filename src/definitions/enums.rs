//! # Models for Gooty Proxy
//!
//! This module contains data structures and type definitions for the core components
//! of the Gooty Proxy application.
//!
//! ## Main Components
//!
//! * **Proxies** - Representation of individual proxies with their connection details,
//!   authentication, and metadata
//! * **Proxy Sources** - Providers and services that supply proxy information
//! * **Browsers** - Browser configurations and profiles for proxy usage
//! * **Regex Patterns** - Regular expression patterns for validation, matching and filtering
//! * **Connection** - Connection states, statistics, and health monitoring
//! * **User Settings** - User configuration and preferences
//!
//! The structures defined here are used throughout the application for data persistence,
//! configuration management, and service integration.
//!

use serde::{Deserialize, Serialize};
use std::fmt;

// Re-export the Proxy struct from the proxy module
pub use super::proxy::Proxy;

/// # Proxy Type
///
/// Represents the protocol used by a proxy server.
///
/// Different proxy types offer varying levels of functionality, security, and speed:
/// - HTTP/HTTPS proxies only work with web traffic
/// - SOCKS proxies can handle any TCP/UDP traffic
/// - SOCKS5 adds authentication and IPv6 support over SOCKS4
///
/// ## Examples
///
/// ```
/// use gooty_proxy::definitions::enums::ProxyType;
/// use std::str::FromStr;
///
/// // Creating from string
/// let proxy_type = ProxyType::from_str("http").unwrap();
/// assert_eq!(proxy_type, ProxyType::Http);
///
/// // Converting to string
/// assert_eq!(ProxyType::Socks5.to_string(), "SOCKS5");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyType {
    /// HTTP proxy protocol - widely supported but unencrypted
    Http,

    /// HTTPS proxy protocol - encrypted connection to the proxy
    Https,

    /// SOCKS4 proxy protocol - supports TCP connections
    Socks4,

    /// SOCKS5 proxy protocol - supports TCP, UDP, and authentication
    Socks5,
}

impl ProxyType {
    /// Returns the default port for this proxy type
    ///
    /// # Returns
    ///
    /// The standard port number commonly used for this proxy type
    ///
    /// # Examples
    ///
    /// ```
    /// use gooty_proxy::definitions::enums::ProxyType;
    ///
    /// assert_eq!(ProxyType::Http.default_port(), 8080);
    /// assert_eq!(ProxyType::Socks5.default_port(), 1080);
    /// ```
    #[must_use] pub fn default_port(&self) -> u16 {
        match self {
            ProxyType::Http => 8080,
            ProxyType::Https => 8443,
            ProxyType::Socks4 => 1080,
            ProxyType::Socks5 => 1080,
        }
    }
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Https => write!(f, "HTTPS"),
            ProxyType::Socks4 => write!(f, "SOCKS4"),
            ProxyType::Socks5 => write!(f, "SOCKS5"),
        }
    }
}

impl std::str::FromStr for ProxyType {
    type Err = String;

    /// Attempts to convert a string to a `ProxyType`
    ///
    /// # Arguments
    ///
    /// * `s` - The string to convert
    ///
    /// # Returns
    ///
    /// * `Ok(ProxyType)` - If the string matches a known proxy type
    /// * `Err(String)` - If the string doesn't match any known proxy type
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(ProxyType::Http),
            "https" => Ok(ProxyType::Https),
            "socks4" => Ok(ProxyType::Socks4),
            "socks5" => Ok(ProxyType::Socks5),
            _ => Err(format!("Unknown proxy type: {s}")),
        }
    }
}

/// Represents the anonymity level of a proxy.
///
/// This enum categorizes proxies based on how much information about the client
/// they reveal to the target server.
///
/// # Variants
///
/// * `Transparent` - The proxy reveals the client's IP address in headers.
/// * `Anonymous` - The proxy reveals it is a proxy but hides the client's IP.
/// * `Elite` - The proxy does not reveal any proxy information or client IP.
///
/// # Examples
///
/// ```
/// use gooty_proxy::definitions::enums::AnonymityLevel;
///
/// let level = AnonymityLevel::Elite;
/// assert_eq!(level.to_string(), "Elite");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnonymityLevel {
    /// Your real IP address is visible in headers (least anonymous)
    Transparent,

    /// Your real IP is hidden but the server knows you're using a proxy
    Anonymous,

    /// Neither your IP nor proxy usage is detectable (most anonymous)
    Elite,
}

impl fmt::Display for AnonymityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnonymityLevel::Transparent => write!(f, "Transparent"),
            AnonymityLevel::Anonymous => write!(f, "Anonymous"),
            AnonymityLevel::Elite => write!(f, "Elite"),
        }
    }
}

impl std::str::FromStr for AnonymityLevel {
    type Err = String;

    /// Converts a string to an `AnonymityLevel`
    ///
    /// # Arguments
    ///
    /// * `s` - The string to convert
    ///
    /// # Returns
    ///
    /// * `Ok(AnonymityLevel)` - If the string matches a known anonymity level
    /// * `Err(String)` - If the string doesn't match any known anonymity level
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "transparent" => Ok(AnonymityLevel::Transparent),
            "anonymous" => Ok(AnonymityLevel::Anonymous),
            "elite" | "high_anonymous" | "high anonymous" => Ok(AnonymityLevel::Elite),
            _ => Err(format!("Unknown anonymity level: {s}")),
        }
    }
}

/// A comparison method to determine which anonymity level is better
impl Ord for AnonymityLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Elite > Anonymous > Transparent
        use std::cmp::Ordering;
        match (self, other) {
            (AnonymityLevel::Elite, AnonymityLevel::Elite) => Ordering::Equal,
            (AnonymityLevel::Elite, _) => Ordering::Greater,
            (AnonymityLevel::Anonymous, AnonymityLevel::Elite) => Ordering::Less,
            (AnonymityLevel::Anonymous, AnonymityLevel::Anonymous) => Ordering::Equal,
            (AnonymityLevel::Anonymous, AnonymityLevel::Transparent) => Ordering::Greater,
            (AnonymityLevel::Transparent, AnonymityLevel::Transparent) => Ordering::Equal,
            (AnonymityLevel::Transparent, _) => Ordering::Less,
        }
    }
}

impl PartialOrd for AnonymityLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents the state of a proxy validation check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationState {
    Pending,
    InProgress,
    Success,
    Failed,
}

impl fmt::Display for ValidationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationState::Pending => write!(f, "Pending"),
            ValidationState::InProgress => write!(f, "In Progress"),
            ValidationState::Success => write!(f, "Success"),
            ValidationState::Failed => write!(f, "Failed"),
        }
    }
}

/// Represents the different rotation strategies for proxy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationStrategy {
    /// Round-robin selection without considering performance
    Sequential,
    /// Random selection without considering performance
    Random,
    /// Select based on lowest latency
    Performance,
    /// Select based on successful requests history
    Reliability,
    /// Weighted random selection based on performance metrics
    Weighted,
}

impl fmt::Display for RotationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RotationStrategy::Sequential => write!(f, "Sequential"),
            RotationStrategy::Random => write!(f, "Random"),
            RotationStrategy::Performance => write!(f, "Performance"),
            RotationStrategy::Reliability => write!(f, "Reliability"),
            RotationStrategy::Weighted => write!(f, "Weighted"),
        }
    }
}

/// Represents the status of a proxy source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceStatus {
    /// Source is active and being used
    Active,
    /// Source is temporarily disabled
    Disabled,
    /// Source has failed too many times and is blacklisted
    Blacklisted,
    /// Source is being used for the first time
    New,
}

impl fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceStatus::Active => write!(f, "Active"),
            SourceStatus::Disabled => write!(f, "Disabled"),
            SourceStatus::Blacklisted => write!(f, "Blacklisted"),
            SourceStatus::New => write!(f, "New"),
        }
    }
}

/// # Log Level
///
/// Represents the level of detail in logging throughout the application.
///
/// Log levels help filter the verbosity of log output, from the most
/// critical-only messages (Error) to highly detailed debugging information (Trace).
///
/// ## Examples
///
/// ```
/// use gooty_proxy::definitions::enums::LogLevel;
/// use std::fmt;
///
/// // Display a log level as string
/// assert_eq!(LogLevel::Info.to_string(), "INFO");
///
/// // Use in logging context
/// let level = LogLevel::Debug;
/// if level == LogLevel::Debug {
///     println!("Debug information: {}", "details");
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Critical errors that may cause application failure
    Error,
    /// Issues that should be addressed but don't prevent operation
    Warn,
    /// General operational messages about system state
    Info,
    /// Detailed information for debugging purposes
    Debug,
    /// Extremely verbose information for tracing execution
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

/// Represents a verification method for proxy testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Simple connectivity test
    Connectivity,
    /// Check if proxy can access specific target
    TargetAccess,
    /// Full anonymity check using judge services
    AnonymityCheck,
    /// Extended verification with multiple judges and targets
    Comprehensive,
}

impl fmt::Display for VerificationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationMethod::Connectivity => write!(f, "Connectivity"),
            VerificationMethod::TargetAccess => write!(f, "Target Access"),
            VerificationMethod::AnonymityCheck => write!(f, "Anonymity Check"),
            VerificationMethod::Comprehensive => write!(f, "Comprehensive"),
        }
    }
}
