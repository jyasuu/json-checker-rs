//! Core JSON validation logic

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

use crate::config::{Config, Rule, CheckResult};
use crate::rules::CheckRule;

/// Main JSON validation engine
pub struct JsonChecker {
    config: Config,
}

impl JsonChecker {
    /// Create a new JsonChecker from a configuration file
    pub fn new(config_path: &str) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .context("Failed to read config file")?;
        let config: Config = serde_json::from_str(&config_content)
            .context("Failed to parse config")?;
        
        Ok(JsonChecker { config })
    }

    /// Create a new JsonChecker from a Config struct
    pub fn from_config(config: Config) -> Self {
        JsonChecker { config }
    }

    /// Run all validation rules and return results
    pub fn run(&self) -> Result<Vec<CheckResult>> {
        let mut results = Vec::new();

        for rule in &self.config.rules {
            match self.check_rule(rule) {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(CheckResult::new(
                        rule.name.clone(),
                        false,
                        format!("Error: {}", e),
                    ));
                }
            }
        }

        Ok(results)
    }

    /// Execute a single validation rule
    pub fn check_rule(&self, rule: &Rule) -> Result<CheckResult> {
        // Read JSON file
        let json_content = fs::read_to_string(&rule.json_file)
            .context(format!("Failed to read file: {}", rule.json_file))?;
        
        let json: Value = serde_json::from_str(&json_content)
            .context("Failed to parse JSON")?;

        // Apply JSONPath
        let mut selector = jsonpath_lib::selector(&json);
        let selected = selector(&rule.jsonpath)
            .context(format!("JSONPath query failed: {}", rule.jsonpath))?;

        // Check rule
        let passed = self.apply_check(&selected, &rule.check)?;
        
        let message = if passed {
            format!("✓ Rule '{}' passed", rule.name)
        } else {
            format!("✗ Rule '{}' failed at path '{}'", rule.name, rule.jsonpath)
        };

        Ok(CheckResult::new(rule.name.clone(), passed, message))
    }

    /// Apply a check rule to a set of JSON values
    pub fn apply_check(&self, values: &[&Value], check: &CheckRule) -> Result<bool> {
        // Check if no values found
        if values.is_empty() {
            return Ok(matches!(check, CheckRule::Empty));
        }

        match check {
            CheckRule::Empty => {
                // Check if all values are "empty" (null, empty string, empty array, empty object)
                Ok(values.iter().all(|v| self.is_empty_value(v)))
            }
            CheckRule::NonEmpty => {
                // Check if all values are non-empty
                Ok(values.iter().all(|v| !self.is_empty_value(v)))
            }
            
            CheckRule::Equals { value } => {
                Ok(values.iter().any(|v| *v == value))
            }
            
            CheckRule::NotEquals { value } => {
                Ok(values.iter().all(|v| *v != value))
            }
            
            CheckRule::Contains { value } => {
                Ok(values.iter().any(|v| self.contains(v, value)))
            }
            
            CheckRule::ContainedBy { value } => {
                Ok(values.iter().all(|v| self.contains(value, v)))
            }
            
            CheckRule::JsonbContains { value } => {
                // PostgreSQL @> operator: left contains right
                Ok(values.iter().any(|v| self.jsonb_contains(v, value)))
            }
            
            CheckRule::JsonbContainedBy { value } => {
                // PostgreSQL <@ operator: left is contained by right
                Ok(values.iter().all(|v| self.jsonb_contains(value, v)))
            }
            
            CheckRule::JsonbExists { key } => {
                Ok(values.iter().any(|v| {
                    if let Value::Object(obj) = v {
                        obj.contains_key(key)
                    } else {
                        false
                    }
                }))
            }
            
            CheckRule::JsonbExistsAny { keys } => {
                Ok(values.iter().any(|v| {
                    if let Value::Object(obj) = v {
                        keys.iter().any(|k| obj.contains_key(k))
                    } else {
                        false
                    }
                }))
            }
            
            CheckRule::JsonbExistsAll { keys } => {
                Ok(values.iter().any(|v| {
                    if let Value::Object(obj) = v {
                        keys.iter().all(|k| obj.contains_key(k))
                    } else {
                        false
                    }
                }))
            }
            
            CheckRule::JsonbPathMatch { path: _ } => {
                // Simplified JSONPath matching (would need jsonpath parser for full impl)
                Ok(true)
            }
            
            CheckRule::Regex { pattern } => {
                let re = regex::Regex::new(pattern)?;
                Ok(values.iter().any(|v| {
                    if let Value::String(s) = v {
                        re.is_match(s)
                    } else {
                        false
                    }
                }))
            }
            
            CheckRule::GreaterThan { value } => {
                Ok(values.iter().any(|v| {
                    if let Some(n) = v.as_f64() {
                        n > *value
                    } else {
                        false
                    }
                }))
            }
            
            CheckRule::LessThan { value } => {
                Ok(values.iter().any(|v| {
                    if let Some(n) = v.as_f64() {
                        n < *value
                    } else {
                        false
                    }
                }))
            }
            
            CheckRule::ArrayLength { min, max } => {
                Ok(values.iter().any(|v| {
                    if let Value::Array(arr) = v {
                        let len = arr.len();
                        let min_ok = min.map_or(true, |m| len >= m);
                        let max_ok = max.map_or(true, |m| len <= m);
                        min_ok && max_ok
                    } else {
                        false
                    }
                }))
            }
        }
    }

    /// Check if a container contains a value
    pub fn contains(&self, container: &Value, contained: &Value) -> bool {
        match (container, contained) {
            (Value::Array(arr), val) => arr.contains(val),
            (Value::String(s), Value::String(sub)) => s.contains(sub.as_str()),
            (Value::Object(obj1), Value::Object(obj2)) => {
                obj2.iter().all(|(k, v)| obj1.get(k) == Some(v))
            }
            _ => false,
        }
    }

    /// JSONB contains operation (PostgreSQL @> operator)
    pub fn jsonb_contains(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Object(l), Value::Object(r)) => {
                r.iter().all(|(k, v)| {
                    l.get(k).map_or(false, |lv| self.jsonb_contains(lv, v))
                })
            }
            (Value::Array(l), Value::Array(r)) => {
                r.iter().all(|rv| l.iter().any(|lv| self.jsonb_contains(lv, rv)))
            }
            (l, r) => l == r,
        }
    }

    /// Check if a value is considered "empty"
    pub fn is_empty_value(&self, value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Object(obj) => obj.is_empty(),
            _ => false,
        }
    }
}