//! JSON Checker Library
//! 
//! A library for validating JSON data against configurable rules using JSONPath queries.

pub mod config;
pub mod checker;
pub mod rules;
pub mod cli;
pub mod path_finder;
pub mod json_path_finder;

#[cfg(test)]
mod tests;

pub use config::{Config, Rule, CheckResult};
pub use checker::JsonChecker;
pub use rules::CheckRule;

/// Re-export commonly used types
pub use anyhow::{Context, Result};
pub use serde_json::Value;