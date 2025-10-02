//! Check rule definitions and validation logic

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All available check rule types for JSON validation
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CheckRule {
    /// Check if value is empty (null, empty string, empty array, empty object)
    Empty,
    /// Check if value is non-empty
    NonEmpty,
    /// Check if value equals a specific value
    Equals { value: Value },
    /// Check if value does not equal a specific value
    NotEquals { value: Value },
    /// Check if container contains a specific value
    Contains { value: Value },
    /// Check if value is contained by a container
    ContainedBy { value: Value },
    /// PostgreSQL @> operator: left contains right (JSONB)
    JsonbContains { value: Value },
    /// PostgreSQL <@ operator: left is contained by right (JSONB)
    JsonbContainedBy { value: Value },
    /// PostgreSQL ? operator: check if key exists
    JsonbExists { key: String },
    /// PostgreSQL ?| operator: check if any of the keys exist
    JsonbExistsAny { keys: Vec<String> },
    /// PostgreSQL ?& operator: check if all keys exist
    JsonbExistsAll { keys: Vec<String> },
    /// PostgreSQL @@ operator: JSONPath match (simplified)
    JsonbPathMatch { path: String },
    /// Regular expression pattern matching
    Regex { pattern: String },
    /// Check if numeric value is greater than threshold
    GreaterThan { value: f64 },
    /// Check if numeric value is less than threshold
    LessThan { value: f64 },
    /// Check array length constraints
    ArrayLength { min: Option<usize>, max: Option<usize> },
}