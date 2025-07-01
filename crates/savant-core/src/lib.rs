//! # Savant Core
//! 
//! Shared types and utilities for all Savant AI components.
//! Follows UNIX philosophy: provides common data structures for component communication.

pub mod types;
pub mod config;
pub mod error;

pub use types::*;
pub use config::*;
pub use error::*;