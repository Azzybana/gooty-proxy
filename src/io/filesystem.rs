//! # Filesystem Module
//!
//! This module provides functionality for managing file-based storage of proxies, sources, and configuration.
//! It includes methods for loading, saving, and managing data in TOML format.
//!
//! ## Components
//!
//! * **Filestore** - A struct for managing file-based storage
//! * **AppConfig** - A struct for application-wide configuration settings
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::io::filesystem::{Filestore, FilestoreConfig};
//!
//! let filestore = Filestore::new().unwrap();
//! let proxies = filestore.load_proxies("my_proxies").unwrap();
//! ```

use crate::definitions::{
    defaults,
    errors::{FilestoreError, FilestoreResult},
    proxy::Proxy,
    source::Source,
};
use crate::utils::SerializableRegex;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration settings for the filestore
///
/// Controls how the filestore loads, saves, and manages data files.
///
/// # Examples
///
/// ```
/// use gooty_proxy::io::filestore::FilestoreConfig;
///
/// // Create a custom configuration
/// let config = FilestoreConfig {
///     data_dir: "my_proxy_data".to_string(),
///     create_defaults_if_missing: true,
///     auto_save_interval_secs: 600, // 10 minutes
///     pretty_print: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilestoreConfig {
    /// Directory where proxy data is stored
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// Whether to create default files when they don't exist
    #[serde(default = "default_true")]
    pub create_defaults_if_missing: bool,

    /// How often to auto-save data (in seconds)
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_secs: u64,

    /// Whether to pretty-print TOML output
    #[serde(default = "default_true")]
    pub pretty_print: bool,
}

// Helper functions for default values
fn default_data_dir() -> String {
    "data".to_string()
}

fn default_true() -> bool {
    true
}

fn default_auto_save_interval() -> u64 {
    defaults::persistence::AUTO_SAVE_INTERVAL_SECS
}

/// Configuration for the entire application
///
/// Contains all configuration settings for the different components
/// of the application, combining them into a single structure.
///
/// # Examples
///
/// ```
/// use gooty_proxy::io::filestore::AppConfig;
///
/// // Create a default configuration
/// let config = AppConfig::default();
///
/// // Access a configuration value
/// assert!(config.request_timeout_secs > 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Filestore configuration
    pub filestore: FilestoreConfig,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Number of retry attempts for failed requests
    pub request_retries: u32,

    /// Delay between sequential requests (ms)
    pub request_delay_ms: u64,

    /// Number of proxies to validate in parallel
    pub parallel_validations: usize,

    /// Maximum acceptable latency for proxies (ms)
    pub max_acceptable_latency_ms: u32,

    /// Minimum success rate for proxy rotation
    pub min_success_rate: f64,

    /// Logging level (error, warn, info, debug, trace)
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            filestore: FilestoreConfig::default(),
            request_timeout_secs: defaults::DEFAULT_REQUEST_TIMEOUT_SECS,
            request_retries: defaults::DEFAULT_REQUEST_RETRIES,
            request_delay_ms: defaults::DEFAULT_REQUEST_DELAY_MS,
            parallel_validations: defaults::DEFAULT_PARALLEL_VALIDATIONS,
            max_acceptable_latency_ms: defaults::DEFAULT_MAX_ACCEPTABLE_LATENCY_MS,
            min_success_rate: defaults::rotation::MIN_SUCCESS_RATE,
            log_level: "info".to_string(),
        }
    }
}

/// Container for storing proxies in TOML format
#[derive(Debug, Serialize, Deserialize)]
struct ProxiesContainer {
    last_updated: String,
    proxies: Vec<Proxy>,
}

/// Container for storing sources in TOML format
#[derive(Debug, Serialize, Deserialize)]
struct SourcesContainer {
    last_updated: String,
    sources: Vec<Source>,
}

/// File-based storage manager for proxies, sources, and configuration
///
/// The Filestore provides methods for loading and saving data to the
/// filesystem, managing proxy lists, source lists, and configuration files.
///
/// It handles file operations, serialization/deserialization, and ensures
/// data consistency when reading and writing files.
///
/// # Examples
///
/// ```
/// use gooty_proxy::io::filestore::{Filestore, FilestoreConfig};
///
/// // Create a filestore with default configuration
/// let filestore = Filestore::new().unwrap();
///
/// // Create a filestore with custom configuration
/// let config = FilestoreConfig {
///     data_dir: "custom_data".to_string(),
///     ..Default::default()
/// };
/// let filestore = Filestore::with_config(config).unwrap();
///
/// // Load proxies from a file
/// let proxies = filestore.load_proxies("my_proxies").unwrap();
/// ```
pub struct Filestore {
    /// Configuration for this filestore instance
    config: FilestoreConfig,

    /// Base directory for all data files
    base_dir: PathBuf,
}

impl Filestore {
    /// Create a new filestore with default configuration
    ///
    /// Creates a new filestore that stores data in the "data" directory.
    ///
    /// # Returns
    ///
    /// A new Filestore instance
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be created or accessed
    pub fn new() -> FilestoreResult<Self> {
        Self::with_config(FilestoreConfig::default())
    }

    /// Create a new filestore with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for the filestore
    ///
    /// # Returns
    ///
    /// A new Filestore instance with the specified configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be created or accessed
    pub fn with_config(config: FilestoreConfig) -> FilestoreResult<Self> {
        let base_dir = PathBuf::from(&config.data_dir);

        // Create the directory if it doesn't exist
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(|e| {
                FilestoreError::IoError(format!("Failed to create directory: {:?}", e))
            })?;
        }

        Ok(Filestore { config, base_dir })
    }

    /// Load proxies from a file
    ///
    /// # Arguments
    ///
    /// * `name` - Base name of the file (without extension)
    ///
    /// # Returns
    ///
    /// A vector of Proxy objects loaded from the file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file doesn't exist and create_defaults_if_missing is false
    /// * The file exists but cannot be read
    /// * The file content is not valid TOML
    /// * The TOML cannot be deserialized into proxies
    pub fn load_proxies(&self, name: &str) -> FilestoreResult<Vec<Proxy>> {
        let file_path = self.get_file_path(name, "toml");

        if !file_path.exists() {
            if self.config.create_defaults_if_missing {
                // Create an empty proxies file
                self.save_proxies(&Vec::new(), name)?;
                return Ok(Vec::new());
            } else {
                return Err(FilestoreError::FileNotFound(
                    file_path.to_string_lossy().to_string(),
                ));
            }
        }

        // Read the file content
        let content = fs::read_to_string(&file_path)
            .map_err(|e| FilestoreError::IoError(format!("Failed to read file: {:?}", e)))?;

        // Parse TOML
        let container: ProxiesContainer = toml::from_str(&content)
            .map_err(|e| FilestoreError::ParseError(format!("Failed to parse TOML: {:?}", e)))?;

        Ok(container.proxies)
    }

    /// Save proxies to a file
    ///
    /// # Arguments
    ///
    /// * `proxies` - Vector of proxies to save
    /// * `name` - Base name of the file (without extension)
    ///
    /// # Returns
    ///
    /// Ok(()) if the proxies were successfully saved
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file cannot be created or written to
    /// * The proxies cannot be serialized to TOML
    pub fn save_proxies(&self, proxies: &[Proxy], name: &str) -> FilestoreResult<()> {
        let file_path = self.get_file_path(name, "toml");

        // Ensure the directory exists
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    FilestoreError::IoError(format!("Failed to create directory: {:?}", e))
                })?;
            }
        }

        // Create a container with metadata
        let container = ProxiesContainer {
            last_updated: Utc::now().to_rfc3339(),
            proxies: proxies.to_vec(),
        };

        // Serialize to TOML
        let toml_content = if self.config.pretty_print {
            toml::to_string_pretty(&container).map_err(|e| {
                FilestoreError::SerializationError(format!("Failed to serialize to TOML: {:?}", e))
            })?
        } else {
            toml::to_string(&container).map_err(|e| {
                FilestoreError::SerializationError(format!("Failed to serialize to TOML: {:?}", e))
            })?
        };

        // Write to file
        fs::write(&file_path, toml_content)
            .map_err(|e| FilestoreError::IoError(format!("Failed to write file: {:?}", e)))?;

        Ok(())
    }

    /// Load sources from a file
    ///
    /// # Arguments
    ///
    /// * `name` - Base name of the file (without extension)
    ///
    /// # Returns
    ///
    /// A vector of Source objects loaded from the file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file doesn't exist and create_defaults_if_missing is false
    /// * The file exists but cannot be read
    /// * The file content is not valid TOML
    /// * The TOML cannot be deserialized into sources
    pub fn load_sources(&self, name: &str) -> FilestoreResult<Vec<Source>> {
        let file_path = self.get_file_path(name, "toml");

        if !file_path.exists() {
            if self.config.create_defaults_if_missing {
                // Create an empty sources file
                self.save_sources(&Vec::new(), name)?;
                return Ok(Vec::new());
            } else {
                return Err(FilestoreError::FileNotFound(
                    file_path.to_string_lossy().to_string(),
                ));
            }
        }

        // Read the file content
        let content = fs::read_to_string(&file_path)
            .map_err(|e| FilestoreError::IoError(format!("Failed to read file: {:?}", e)))?;

        // Parse TOML
        let container: SourcesContainer = toml::from_str(&content)
            .map_err(|e| FilestoreError::ParseError(format!("Failed to parse TOML: {:?}", e)))?;

        // Recompile regex patterns in sources
        let mut sources = container.sources;
        for source in &mut sources {
            if let Ok(regex) = SerializableRegex::new(&source.regex_pattern) {
                source.compiled_regex = Some(regex);
            }
        }

        Ok(sources)
    }

    /// Save sources to a file
    ///
    /// # Arguments
    ///
    /// * `sources` - Vector of sources to save
    /// * `name` - Base name of the file (without extension)
    ///
    /// # Returns
    ///
    /// Ok(()) if the sources were successfully saved
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file cannot be created or written to
    /// * The sources cannot be serialized to TOML
    pub fn save_sources(&self, sources: &[Source], name: &str) -> FilestoreResult<()> {
        let file_path = self.get_file_path(name, "toml");

        // Ensure the directory exists
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    FilestoreError::IoError(format!("Failed to create directory: {:?}", e))
                })?;
            }
        }

        // Create a container with metadata
        let container = SourcesContainer {
            last_updated: Utc::now().to_rfc3339(),
            sources: sources.to_vec(),
        };

        // Serialize to TOML
        let toml_content = if self.config.pretty_print {
            toml::to_string_pretty(&container).map_err(|e| {
                FilestoreError::SerializationError(format!("Failed to serialize to TOML: {:?}", e))
            })?
        } else {
            toml::to_string(&container).map_err(|e| {
                FilestoreError::SerializationError(format!("Failed to serialize to TOML: {:?}", e))
            })?
        };

        // Write to file
        fs::write(&file_path, toml_content)
            .map_err(|e| FilestoreError::IoError(format!("Failed to write file: {:?}", e)))?;

        Ok(())
    }

    /// Load application configuration from a file
    ///
    /// # Arguments
    ///
    /// * `name` - Base name of the file (without extension)
    ///
    /// # Returns
    ///
    /// An AppConfig object loaded from the file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file doesn't exist and create_defaults_if_missing is false
    /// * The file exists but cannot be read
    /// * The file content is not valid TOML
    /// * The TOML cannot be deserialized into AppConfig
    pub fn load_config(&self, name: &str) -> FilestoreResult<AppConfig> {
        let file_path = self.get_file_path(name, "toml");

        if !file_path.exists() {
            if self.config.create_defaults_if_missing {
                // Create a default config file
                let default_config = AppConfig::default();
                self.save_config(&default_config, name)?;
                return Ok(default_config);
            } else {
                return Err(FilestoreError::FileNotFound(
                    file_path.to_string_lossy().to_string(),
                ));
            }
        }

        // Read the file content
        let content = fs::read_to_string(&file_path)
            .map_err(|e| FilestoreError::IoError(format!("Failed to read file: {:?}", e)))?;

        // Parse TOML
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| FilestoreError::ParseError(format!("Failed to parse TOML: {:?}", e)))?;

        Ok(config)
    }

    /// Save application configuration to a file
    ///
    /// # Arguments
    ///
    /// * `config` - AppConfig object to save
    /// * `name` - Base name of the file (without extension)
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration was successfully saved
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file cannot be created or written to
    /// * The configuration cannot be serialized to TOML
    pub fn save_config(&self, config: &AppConfig, name: &str) -> FilestoreResult<()> {
        let file_path = self.get_file_path(name, "toml");

        // Ensure the directory exists
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    FilestoreError::IoError(format!("Failed to create directory: {:?}", e))
                })?;
            }
        }

        // Serialize to TOML
        let toml_content = if self.config.pretty_print {
            toml::to_string_pretty(config).map_err(|e| {
                FilestoreError::SerializationError(format!("Failed to serialize to TOML: {:?}", e))
            })?
        } else {
            toml::to_string(config).map_err(|e| {
                FilestoreError::SerializationError(format!("Failed to serialize to TOML: {:?}", e))
            })?
        };

        // Write to file
        fs::write(&file_path, toml_content)
            .map_err(|e| FilestoreError::IoError(format!("Failed to write file: {:?}", e)))?;

        Ok(())
    }

    /// Get the base directory where files are stored
    ///
    /// # Returns
    ///
    /// Reference to the base directory path
    pub fn get_base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Get the current filestore configuration
    ///
    /// # Returns
    ///
    /// Reference to the current configuration
    pub fn get_config(&self) -> &FilestoreConfig {
        &self.config
    }

    /// Create a file path by joining the base directory with the name and extension
    ///
    /// # Arguments
    ///
    /// * `name` - Base name of the file
    /// * `extension` - File extension (without dot)
    ///
    /// # Returns
    ///
    /// The complete file path
    fn get_file_path(&self, name: &str, extension: &str) -> PathBuf {
        self.base_dir.join(format!("{}.{}", name, extension))
    }
}
