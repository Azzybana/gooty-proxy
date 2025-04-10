//! # I/O Module
//!
//! This module handles all input/output operations for the gooty proxy system.
//! It includes components for file storage, HTTP requests, and general I/O utilities.
//!
//! ## Components
//!
//! * **filestore** - Manages persistent storage of proxies, sources, and configuration
//! * **requestor** - Handles HTTP requests with proxy support and error handling

pub mod filesystem;
pub mod http;

// Re-exports from modules
pub use filesystem::{AppConfig, Filestore, FilestoreConfig};
pub use http::Requestor;
