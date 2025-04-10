//! # Configuration Module
//!
//! This module handles configuration management for the gooty proxy system.
//! It includes schema definitions, configuration loading, and validation logic.
//!
//! ## Components
//!
//! * **Schema** - Defines the structure of configuration files
//! * **Loader** - Handles loading and parsing configuration files
//!
//! ## Overview
//!
//! The configuration module is responsible for:
//! - Defining the structure of the application configuration
//! - Loading configuration from TOML files
//! - Validating configuration values
//! - Saving configuration back to disk
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::config::{ConfigLoader, AppConfig};
//! use std::path::Path;
//!
//! let config_loader = ConfigLoader::new(Path::new("./config")).unwrap();
//! let config = config_loader.get_config();
//! println!("Log level: {}", config.application.log_level);
//! ```

pub mod loader;
pub mod schema;

pub use loader::ConfigLoader;
pub use schema::{AppConfig, HttpConfig, JudgeConfig, ProxiesConfig, StorageConfig};
