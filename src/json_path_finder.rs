//! JSON path finder implementation as requested in TODO.md
//! This module provides functionality to find the exact path to specific values in JSON structures

use serde_json::Value;

/// Find the path to a specific value within a JSON structure
/// Returns the JSONPath-style path to the target value, or None if not found
/// 
/// This is the exact implementation requested in TODO.md
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

/// Demo function that runs the exact example from TODO.md
pub fn demo_find_path() {
    let json_str = r#"
    {
        "user": {
            "name": "Leo",
            "details": {
                "age": 30,
                "location": "Taiwan"
            }
        }
    }
    "#;

    let json: Value = serde_json::from_str(json_str).unwrap();
    let target = Value::String("Taiwan".to_string());

    if let Some(path) = find_json_path(&json, &target, vec!["$".to_string()]) {
        println!("Found path: {}", path);
    } else {
        println!("Value not found.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_todo_md_example() {
        let json_str = r#"
        {
            "user": {
                "name": "Leo",
                "details": {
                    "age": 30,
                    "location": "Taiwan"
                }
            }
        }
        "#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let target = Value::String("Taiwan".to_string());

        let result = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(result, Some("$.user.details.location".to_string()));
    }

    #[test]
    fn test_find_name() {
        let json = json!({
            "user": {
                "name": "Leo",
                "details": {
                    "age": 30,
                    "location": "Taiwan"
                }
            }
        });

        let target = Value::String("Leo".to_string());
        let result = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(result, Some("$.user.name".to_string()));
    }

    #[test]
    fn test_find_number() {
        let json = json!({
            "user": {
                "name": "Leo",
                "details": {
                    "age": 30,
                    "location": "Taiwan"
                }
            }
        });

        let target = Value::Number(serde_json::Number::from(30));
        let result = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(result, Some("$.user.details.age".to_string()));
    }

    #[test]
    fn test_value_not_found() {
        let json = json!({"name": "Leo"});
        let target = Value::String("NotFound".to_string());
        let result = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_array_indexing() {
        let json = json!({
            "items": ["first", "second", "Taiwan"]
        });

        let target = Value::String("Taiwan".to_string());
        let result = find_json_path(&json, &target, vec!["$".to_string()]);
        assert_eq!(result, Some("$.items.[2]".to_string()));
    }
}