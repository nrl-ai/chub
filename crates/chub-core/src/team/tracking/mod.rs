//! Entire.io-compatible AI session tracking.
//!
//! This module provides session state management, transcript parsing,
//! checkpoint storage, and attribution tracking with data formats
//! compatible with the entire.io CLI.

pub mod branch_store;
pub mod checkpoint;
pub mod redact;
pub mod session_state;
pub mod transcript;
pub mod types;
