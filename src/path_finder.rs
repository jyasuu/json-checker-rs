//! JSON path finding utilities for locating specific values within JSON structures

use serde_json::Value;

/// Find the path to a specific value within a JSON structure
/// 
/// Returns the JSONPath-style path to the target value, or None if not found
/// 
/// # Arguments
/// * `value` - The JSON value to search within
/// * `target` - The target value to find
/// * `path` - Current path components (used for recursion)
/// 
/// # Examples
/// ```
/// use serde_json::Value;
/// use json_checker_rs::path_finder::find_json_path;
/// 
/// let json: Value = serde_json::from_str(r#"{"user": {"name": "Leo"}}"#).unwrap();
/// let target = Value::String("Leo".to_string());
/// let path = find_json_path(&json, &target, vec!["$".to_string()]);
/// assert_eq!(path, Some("$.user.name".to_string()));
/// ```
pub fn find_json_path(value: &Value, target: &Value, path: Vec<String>) -> Option<String> {
    if value == target {
        return Some(path.join("."));
    }

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let mut new_path = path.clone();
                new_path.push(key.clone());
                if let Some(found) = find_json_path(val, target, new_path) {
                    return Some(found);
                }
            }
        }
        Value::Array(arr) => {
            for (index, val) in arr.iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", index));
                if let Some(found) = find_json_path(val, target, new_path) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }

    None
}

/// Find all paths to values that match a predicate function
/// 
/// Returns a vector of JSONPath-style paths where the predicate returns true
/// 
/// # Arguments
/// * `value` - The JSON value to search within
/// * `predicate` - Function that returns true for target values
/// * `path` - Current path components (used for recursion)
pub fn find_json_paths_matching<F>(
    value: &Value, 
    predicate: F, 
    path: Vec<String>
) -> Vec<String> 
where
    F: Fn(&Value) -> bool + Copy,
{
    let mut results = Vec::new();
    
    if predicate(value) {
        results.push(path.join("."));
    }

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let mut new_path = path.clone();
                new_path.push(key.clone());
                results.extend(find_json_paths_matching(val, predicate, new_path));
            }
        }
        Value::Array(arr) => {
            for (index, val) in arr.iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", index));
                results.extend(find_json_paths_matching(val, predicate, new_path));
            }
        }
        _ => {}
    }

    results
}

/// Find paths to values that fail a specific validation check
/// 
/// This is useful for identifying exactly which nodes in a JSON structure
/// are causing validation failures
/// 
/// # Arguments
/// * `value` - The JSON value to search within
/// * `check_fn` - Function that returns false for invalid values
/// * `path` - Current path components (used for recursion)
pub fn find_invalid_paths<F>(
    value: &Value, 
    check_fn: F, 
    path: Vec<String>
) -> Vec<String>
where
    F: Fn(&Value) -> bool + Copy,
{
    find_json_paths_matching(value, |v| !check_fn(v), path)
}

/// Get a human-readable path representation
/// 
/// Converts a path like "$.user.details[0].name" to a more readable format
pub fn format_path_readable(path: &str) -> String {
    path.replace("$.", "")
        .replace("[", " → item ")
        .replace("]", "")
        .replace(".", " → ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_find_json_path_simple() {
        let json = json!({"name": "Leo"});
        let target = Value::String("Leo".to_string());
        let path = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(path, Some("$.name".to_string()));
    }

    #[test]
    fn test_find_json_path_nested() {
        let json = json!({
            "user": {
                "details": {
                    "location": "Taiwan"
                }
            }
        });
        let target = Value::String("Taiwan".to_string());
        let path = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(path, Some("$.user.details.location".to_string()));
    }

    #[test]
    fn test_find_json_path_array() {
        let json = json!(["first", "second", "third"]);
        let target = Value::String("second".to_string());
        let path = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(path, Some("$.[1]".to_string()));
    }

    #[test]
    fn test_find_json_path_not_found() {
        let json = json!({"name": "Leo"});
        let target = Value::String("NotFound".to_string());
        let path = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(path, None);
    }

    #[test]
    fn test_find_json_paths_matching() {
        let json = json!({
            "numbers": [1, 2, 3, 4, 5],
            "nested": {
                "value": 10
            }
        });
        
        let paths = find_json_paths_matching(
            &json, 
            |v| v.as_f64().map_or(false, |n| n > 3.0),
            vec!["$".to_string()]
        );
        
        assert!(paths.contains(&"$.numbers.[3]".to_string())); // 4
        assert!(paths.contains(&"$.numbers.[4]".to_string())); // 5
        assert!(paths.contains(&"$.nested.value".to_string())); // 10
    }

    #[test]
    fn test_format_path_readable() {
        assert_eq!(
            format_path_readable("$.user.details[0].name"),
            "user → details → item 0 → name"
        );
        assert_eq!(
            format_path_readable("$.simple"),
            "simple"
        );
    }
}