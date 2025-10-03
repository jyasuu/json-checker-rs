//! Core JSON validation logic

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

use crate::config::{Config, Rule, CheckResult};
use crate::rules::CheckRule;
use crate::json_path_finder::find_json_path;

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
            let mut error_msg = format!("✗ Rule '{}' failed at JSONPath '{}'", rule.name, rule.jsonpath);
            
            // Find and show invalid values with their exact positions
            if !selected.is_empty() {
                let invalid_indices = self.find_invalid_value_indices(&selected, &rule.check);
                
                if !invalid_indices.is_empty() {
                    error_msg.push_str("\n   Invalid nodes found at:");
                    
                    // Use a set to track unique paths and avoid duplicates
                    let mut unique_paths = std::collections::HashSet::new();
                    
                    for &index in &invalid_indices {
                        if index < selected.len() {
                            let invalid_value = selected[index];
                            // Use JSONPath context to find the correct path for this specific selected value
                            if let Some(path) = self.find_path_for_selected_value(&json, &rule.jsonpath, index) {
                                let path_with_value = match invalid_value {
                                    Value::String(s) => format!("{} = \"{}\"", path, s),
                                    Value::Number(n) => format!("{} = {}", path, n),
                                    Value::Bool(b) => format!("{} = {}", path, b),
                                    Value::Null => format!("{} = null", path),
                                    Value::Array(arr) => format!("{} = array[{}]", path, arr.len()),
                                    Value::Object(obj) => format!("{} = object{{{}}}", path, obj.len()),
                                };
                                
                                // Only add if we haven't seen this path+value combination before
                                if unique_paths.insert(path_with_value.clone()) {
                                    error_msg.push_str(&format!("\n   • {}", path_with_value));
                                }
                            }
                        }
                    }
                }
            }
            
            error_msg
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

    /// Find indices of values that are causing validation failures
    pub fn find_invalid_value_indices(&self, values: &[&Value], check: &CheckRule) -> Vec<usize> {
        let mut invalid_indices = Vec::new();

        match check {
            CheckRule::Empty => {
                for (i, v) in values.iter().enumerate() {
                    if !self.is_empty_value(v) {
                        invalid_indices.push(i);
                    }
                }
            }
            CheckRule::NonEmpty => {
                for (i, v) in values.iter().enumerate() {
                    if self.is_empty_value(v) {
                        invalid_indices.push(i);
                    }
                }
            }
            CheckRule::Equals { value: target } => {
                let has_match = values.iter().any(|v| *v == target);
                if !has_match {
                    // If no values match, all are invalid
                    invalid_indices.extend(0..values.len());
                }
            }
            CheckRule::NotEquals { value: target } => {
                for (i, v) in values.iter().enumerate() {
                    if *v == target {
                        invalid_indices.push(i);
                    }
                }
            }
            CheckRule::Contains { value: target } => {
                let has_match = values.iter().any(|v| self.contains(v, target));
                if !has_match {
                    invalid_indices.extend(0..values.len());
                }
            }
            CheckRule::ContainedBy { value: container } => {
                for (i, v) in values.iter().enumerate() {
                    if !self.contains(container, v) {
                        invalid_indices.push(i);
                    }
                }
            }
            CheckRule::Regex { pattern } => {
                if let Ok(re) = regex::Regex::new(pattern) {
                    for (i, v) in values.iter().enumerate() {
                        let matches = if let Value::String(s) = v {
                            re.is_match(s)
                        } else {
                            false
                        };
                        if !matches {
                            invalid_indices.push(i);
                        }
                    }
                }
            }
            CheckRule::GreaterThan { value: threshold } => {
                for (i, v) in values.iter().enumerate() {
                    let is_valid = if let Some(n) = v.as_f64() {
                        n > *threshold
                    } else {
                        false
                    };
                    if !is_valid {
                        invalid_indices.push(i);
                    }
                }
            }
            CheckRule::LessThan { value: threshold } => {
                for (i, v) in values.iter().enumerate() {
                    let is_valid = if let Some(n) = v.as_f64() {
                        n < *threshold
                    } else {
                        false
                    };
                    if !is_valid {
                        invalid_indices.push(i);
                    }
                }
            }
            CheckRule::ArrayLength { min, max } => {
                for (i, v) in values.iter().enumerate() {
                    let is_valid = if let Value::Array(arr) = v {
                        let len = arr.len();
                        let min_ok = min.map_or(true, |m| len >= m);
                        let max_ok = max.map_or(true, |m| len <= m);
                        min_ok && max_ok
                    } else {
                        false
                    };
                    if !is_valid {
                        invalid_indices.push(i);
                    }
                }
            }
            _ => {
                // For other rules, if validation failed, consider all values as potentially invalid
                invalid_indices.extend(0..values.len());
            }
        }

        invalid_indices
    }

    /// Find the correct path for a selected value based on JSONPath and its index
    pub fn find_path_for_selected_value(&self, json: &Value, jsonpath: &str, index: usize) -> Option<String> {
        // For simple cases like $.users[*].name or $.users[*].email, we can reconstruct the path
        if jsonpath.contains("[*]") {
            let base_path = jsonpath.replace("[*]", &format!("[{}]", index));
            return Some(base_path);
        }
        
        // For complex JSONPath queries, we need to find all matching paths and return the correct one
        let mut selector = jsonpath_lib::selector(json);
        if let Ok(results) = selector(jsonpath) {
            if index < results.len() {
                let target_value = results[index];
                // Find all occurrences of this value and return the one at the correct index
                return self.find_nth_occurrence_path(json, target_value, index);
            }
        }
        
        None
    }

    /// Find the nth occurrence of a value in JSON and return its path
    fn find_nth_occurrence_path(&self, json: &Value, target: &Value, target_index: usize) -> Option<String> {
        let mut found_paths = Vec::new();
        self.find_all_paths_recursive(json, target, vec!["$".to_string()], &mut found_paths);
        
        if target_index < found_paths.len() {
            Some(found_paths[target_index].clone())
        } else {
            None
        }
    }

    /// Recursively find all paths to a target value
    fn find_all_paths_recursive(&self, value: &Value, target: &Value, path: Vec<String>, found_paths: &mut Vec<String>) {
        if value == target {
            found_paths.push(path.join("."));
        }

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let mut new_path = path.clone();
                    new_path.push(key.clone());
                    self.find_all_paths_recursive(val, target, new_path, found_paths);
                }
            }
            Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let mut new_path = path.clone();
                    new_path.push(format!("[{}]", index));
                    self.find_all_paths_recursive(val, target, new_path, found_paths);
                }
            }
            _ => {}
        }
    }
}