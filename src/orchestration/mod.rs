//! # Orchestration Module
//!
//! Provides core orchestration functionality for managing threads, processes, and tasks.
//!
//! ## Overview
//!
//! This module serves as the central hub for:
//! - Thread and process management
//! - Task scheduling and coordination
//! - Resource allocation and monitoring
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::orchestration;
//!
//! // Example of initializing the orchestration manager
//! let manager = orchestration::Manager::new();
//! assert!(manager.is_ok());
//! ```

pub mod manager;
pub mod processes;
pub mod threading;
