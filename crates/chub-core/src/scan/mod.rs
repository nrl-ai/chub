//! Secret scanning engine — gitleaks/betterleaks-compatible drop-in replacement.
//!
//! Provides directory scanning, git history scanning, stdin scanning, and
//! multiple output formats (JSON, SARIF, CSV). Reuses the redaction engine's
//! rule set and adds location tracking, fingerprinting, and report generation.

pub mod config;
pub mod finding;
pub mod report;
pub mod scanner;
