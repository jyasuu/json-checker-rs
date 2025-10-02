//! Configuration structures for JSON validation rules

use serde::{Deserialize, Serialize};
use crate::rules::CheckRule;

/// Main configuration structure containing all validation rules
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub rules: Vec<Rule>,
}

/// Individual validation rule configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub name: String,
    pub json_file: String,
    pub jsonpath: String,
    pub check: CheckRule,
}

/// Result of executing a validation rule
#[derive(Debug)]
pub struct CheckResult {
    pub rule_name: String,
    pub passed: bool,
    pub message: String,
}

impl CheckResult {
    pub fn new(rule_name: String, passed: bool, message: String) -> Self {
        Self {
            rule_name,
            passed,
            message,
        }
    }
}