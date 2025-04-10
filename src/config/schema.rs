//! # Configuration Schema
//!
//! This module defines the structure of the application's configuration.
//! It includes settings for various components such as HTTP client, proxy management, and storage.
//!
//! ## Overview
//!
//! The schema is responsible for:
//! - Defining configuration structures
//! - Providing default values for configuration fields
//! - Serializing and deserializing configuration to/from TOML
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::config::schema::AppConfig;
//!
//! let config = AppConfig::default();
//! println!("Default log level: {}", config.application.log_level);
//! ```

use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Application-wide settings
    #[serde(default)]
    pub application: ApplicationConfig,

    /// HTTP client settings
    #[serde(default)]
    pub http: HttpConfig,

    /// Proxy judge settings
    #[serde(default)]
    pub judge: JudgeConfig,

    /// Proxy management settings
    #[serde(default)]
    pub proxies: ProxiesConfig,

    /// Storage and persistence settings
    #[serde(default)]
    pub storage: StorageConfig,
}

/// Application-wide configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Logging level (error, warn, info, debug, trace)
    pub log_level: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
        }
    }
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Number of retry attempts for failed requests
    pub request_retries: u32,

    /// Delay between sequential requests in milliseconds
    pub request_delay_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            request_timeout_secs: 30,
            request_retries: 3,
            request_delay_ms: 500,
        }
    }
}

/// Configuration for proxy judge services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    /// Number of proxies to validate in parallel
    pub parallel_validations: usize,

    /// Maximum acceptable latency for proxies in milliseconds
    pub max_acceptable_latency_ms: u32,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            parallel_validations: 20,
            max_acceptable_latency_ms: 2000,
        }
    }
}

/// Proxy management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxiesConfig {
    /// Minimum success rate threshold for proxies
    pub min_success_rate: f64,
}

impl Default for ProxiesConfig {
    fn default() -> Self {
        Self {
            min_success_rate: 0.7,
        }
    }
}

/// Storage and persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory where data is stored
    pub data_dir: String,

    /// Whether to create default files when they don't exist
    pub create_defaults_if_missing: bool,

    /// How often to auto-save data (in seconds)
    pub auto_save_interval_secs: u64,

    /// Whether to pretty-print TOML output
    pub pretty_print: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "data".to_string(),
            create_defaults_if_missing: true,
            auto_save_interval_secs: 300,
            pretty_print: true,
        }
    }
}
