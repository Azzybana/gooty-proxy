//! # Configuration Loader
//!
//! This module provides functionality for loading, saving, and managing application configuration files.
//! It ensures that configuration files are validated, defaults are created when missing, and changes are persisted.
//!
//! ## Overview
//!
//! The configuration loader is responsible for:
//! - Loading configuration from TOML files
//! - Validating configuration values
//! - Saving configuration back to disk
//! - Managing configuration snapshots for backup and restore
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::config::loader::ConfigLoader;
//! use std::path::Path;
//!
//! let mut loader = ConfigLoader::new(Path::new("./config")).unwrap();
//! let config = loader.get_config();
//! println!("Log level: {}", config.application.log_level);
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use log::{debug, info, warn};

use crate::config::schema::AppConfig;
use crate::definitions::errors::{ConfigError, ConfigResult};

/// Configuration loader that handles loading and saving configuration files
pub struct ConfigLoader {
    /// Directory containing configuration files
    config_dir: PathBuf,

    /// Base configuration file name
    config_filename: String,

    /// Current configuration
    config: AppConfig,
}

impl ConfigLoader {
    /// Create a new `ConfigLoader` with the specified directory and default filename
    pub fn new<P: AsRef<Path>>(config_dir: P) -> ConfigResult<Self> {
        Self::with_filename(config_dir, "config.toml")
    }

    /// Create a new `ConfigLoader` with a specified directory and filename
    pub fn with_filename<P: AsRef<Path>>(config_dir: P, filename: &str) -> ConfigResult<Self> {
        let config_dir = config_dir.as_ref().to_path_buf();

        // Create the directory if it doesn't exist
        if !config_dir.exists() {
            info!("Creating configuration directory: {config_dir:?}");
            fs::create_dir_all(&config_dir).map_err(ConfigError::IoError)?;
        }

        // Build the config path
        let config_path = config_dir.join(filename);

        // Load or create default configuration
        let config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            info!("Configuration file not found, creating default");
            let default_config = AppConfig::default();
            Self::save_to_file(&default_config, &config_path)?;
            default_config
        };

        Ok(ConfigLoader {
            config_dir,
            config_filename: filename.to_string(),
            config,
        })
    }

    /// Get the current configuration
    #[must_use] pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// Get a mutable reference to the current configuration
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Update the configuration and save changes to disk
    pub fn update_config(&mut self, config: AppConfig) -> ConfigResult<()> {
        let config_path = self.config_dir.join(&self.config_filename);
        Self::save_to_file(&config, &config_path)?;
        self.config = config;
        debug!("Configuration updated and saved to {config_path:?}");
        Ok(())
    }

    /// Reload configuration from disk
    pub fn reload(&mut self) -> ConfigResult<()> {
        let config_path = self.config_dir.join(&self.config_filename);
        if config_path.exists() {
            self.config = Self::load_from_file(&config_path)?;
            debug!("Configuration reloaded from {config_path:?}");
            Ok(())
        } else {
            warn!("Configuration file not found at {config_path:?}");
            Err(ConfigError::MissingConfig(config_path))
        }
    }

    /// Save the current configuration to disk
    pub fn save(&self) -> ConfigResult<()> {
        let config_path = self.config_dir.join(&self.config_filename);
        Self::save_to_file(&self.config, &config_path)?;
        debug!("Configuration saved to {config_path:?}");
        Ok(())
    }

    /// Reset the configuration to default values and save to disk
    pub fn reset_to_defaults(&mut self) -> ConfigResult<()> {
        self.config = AppConfig::default();
        self.save()?;
        info!("Configuration reset to defaults");
        Ok(())
    }

    /// Get the path to the configuration file
    #[must_use] pub fn get_config_path(&self) -> PathBuf {
        self.config_dir.join(&self.config_filename)
    }

    /// Check if the configuration file exists
    #[must_use] pub fn config_exists(&self) -> bool {
        self.get_config_path().exists()
    }

    /// Load configuration from a file
    fn load_from_file(path: &Path) -> ConfigResult<AppConfig> {
        debug!("Loading configuration from {path:?}");
        let content = fs::read_to_string(path).map_err(ConfigError::IoError)?;

        let config: AppConfig = toml::from_str(&content).map_err(ConfigError::TomlDeError)?;

        Ok(config)
    }

    /// Save configuration to a file
    fn save_to_file(config: &AppConfig, path: &Path) -> ConfigResult<()> {
        debug!("Saving configuration to {path:?}");
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(ConfigError::IoError)?;
            }
        }

        // Convert to TOML with pretty formatting
        let toml_string = if config.storage.pretty_print {
            toml::to_string_pretty(config).map_err(ConfigError::TomlSerError)?
        } else {
            toml::to_string(config).map_err(ConfigError::TomlSerError)?
        };

        fs::write(path, toml_string).map_err(ConfigError::IoError)?;

        Ok(())
    }

    /// Validate the current configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate log level
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        let log_level = self.config.application.log_level.to_lowercase();

        if !valid_log_levels.contains(&log_level.as_str()) {
            return Err(ConfigError::InvalidValue(format!(
                "Invalid log_level: {log_level}. Must be one of: error, warn, info, debug, trace"
            )));
        }

        // Validate HTTP settings
        if self.config.http.request_timeout_secs == 0 {
            return Err(ConfigError::InvalidValue(
                "request_timeout_secs must be greater than 0".to_string(),
            ));
        }

        // Validate judge settings
        if self.config.judge.parallel_validations == 0 {
            return Err(ConfigError::InvalidValue(
                "parallel_validations must be greater than 0".to_string(),
            ));
        }

        // Validate proxies settings
        if self.config.proxies.min_success_rate < 0.0 || self.config.proxies.min_success_rate > 1.0
        {
            return Err(ConfigError::InvalidValue(
                "min_success_rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate storage settings
        if self.config.storage.auto_save_interval_secs == 0 {
            return Err(ConfigError::InvalidValue(
                "auto_save_interval_secs must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a snapshot of the configuration with the current timestamp
    pub fn create_snapshot(&self) -> ConfigResult<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let snapshot_filename = format!("config_backup_{timestamp}.toml");
        let snapshot_path = self.config_dir.join("backups").join(&snapshot_filename);

        // Ensure the backups directory exists
        if let Some(parent) = snapshot_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(ConfigError::IoError)?;
            }
        }

        Self::save_to_file(&self.config, &snapshot_path)?;
        info!("Configuration snapshot created at {snapshot_path:?}");

        Ok(snapshot_path)
    }

    /// List all configuration snapshots
    pub fn list_snapshots(&self) -> ConfigResult<Vec<PathBuf>> {
        let backups_dir = self.config_dir.join("backups");

        if !backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        for entry in fs::read_dir(backups_dir).map_err(ConfigError::IoError)? {
            let entry = entry.map_err(ConfigError::IoError)?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                snapshots.push(path);
            }
        }

        // Sort by modification time, newest first
        snapshots.sort_by(|a, b| {
            let a_meta = fs::metadata(a).ok();
            let b_meta = fs::metadata(b).ok();

            match (a_meta, b_meta) {
                (Some(a_meta), Some(b_meta)) => {
                    let a_modified = a_meta.modified().ok().map(DateTime::<Utc>::from);
                    let b_modified = b_meta.modified().ok().map(DateTime::<Utc>::from);

                    match (b_modified, a_modified) {
                        (Some(b_time), Some(a_time)) => b_time.cmp(&a_time),
                        _ => std::cmp::Ordering::Equal,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(snapshots)
    }

    /// Restore configuration from a snapshot
    pub fn restore_from_snapshot(&mut self, snapshot_path: &Path) -> ConfigResult<()> {
        if !snapshot_path.exists() {
            return Err(ConfigError::MissingConfig(snapshot_path.to_path_buf()));
        }

        let snapshot_config = Self::load_from_file(snapshot_path)?;
        self.update_config(snapshot_config)?;

        info!("Configuration restored from snapshot {snapshot_path:?}");
        Ok(())
    }
}
