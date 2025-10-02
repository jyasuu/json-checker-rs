//! Comprehensive test suite for JSON validation rules

#[cfg(test)]
mod tests {
    use crate::{JsonChecker, Config, CheckRule};
    use serde_json::Value;

    fn create_test_checker() -> JsonChecker {
        JsonChecker::from_config(Config { rules: vec![] })
    }

    // Helper function to create test values and avoid borrow checker issues
    fn test_apply_check(checker: &JsonChecker, values: Vec<Value>, check: &CheckRule) -> bool {
        let value_refs: Vec<&Value> = values.iter().collect();
        checker.apply_check(&value_refs, check).unwrap()
    }

    #[test]
    fn test_empty_check() {
        let checker = create_test_checker();
        let check = CheckRule::Empty;
        
        // Test empty values
        let empty_values = vec![
            serde_json::json!(null),
            serde_json::json!(""),
            serde_json::json!([]),
            serde_json::json!({}),
        ];
        assert!(test_apply_check(&checker, empty_values, &check));
        
        // Test non-empty values
        let non_empty_values = vec![
            serde_json::json!("test"),
            serde_json::json!([1, 2]),
            serde_json::json!({"key": "value"}),
            serde_json::json!(42),
        ];
        assert!(!test_apply_check(&checker, non_empty_values, &check));
        
        // Test no values found (should pass for Empty check)
        assert!(test_apply_check(&checker, vec![], &check));
    }

    #[test]
    fn test_non_empty_check() {
        let checker = create_test_checker();
        let check = CheckRule::NonEmpty;
        
        // Test non-empty values
        let non_empty_values = vec![
            serde_json::json!("test"),
            serde_json::json!([1, 2]),
            serde_json::json!({"key": "value"}),
            serde_json::json!(42),
            serde_json::json!(true),
        ];
        assert!(test_apply_check(&checker, non_empty_values, &check));
        
        // Test empty values
        let empty_values = vec![
            serde_json::json!(null),
            serde_json::json!(""),
            serde_json::json!([]),
            serde_json::json!({}),
        ];
        assert!(!test_apply_check(&checker, empty_values, &check));
        
        // Test mixed values (should fail if any are empty)
        let mixed_values = vec![
            serde_json::json!("test"),
            serde_json::json!(""),
        ];
        assert!(!test_apply_check(&checker, mixed_values, &check));
    }

    #[test]
    fn test_equals_check() {
        let checker = create_test_checker();
        
        let check = CheckRule::Equals { value: serde_json::json!("admin") };
        
        // Test matching values (should pass if any match)
        let matching_values = vec![
            serde_json::json!("admin"),
            serde_json::json!("user"),
        ];
        assert!(test_apply_check(&checker, matching_values, &check));
        
        // Test non-matching values
        let non_matching_values = vec![
            serde_json::json!("user"),
            serde_json::json!("guest"),
        ];
        assert!(!test_apply_check(&checker, non_matching_values, &check));
        
        // Test with numbers
        let number_check = CheckRule::Equals { value: serde_json::json!(42) };
        let number_values = vec![serde_json::json!(42)];
        assert!(test_apply_check(&checker, number_values, &number_check));
    }

    #[test]
    fn test_not_equals_check() {
        let checker = create_test_checker();
        let check = CheckRule::NotEquals { value: serde_json::json!("admin") };
        
        // Test non-matching values (should pass)
        let non_matching_values = vec![
            serde_json::json!("user"),
            serde_json::json!("guest"),
        ];
        assert!(test_apply_check(&checker, non_matching_values, &check));
        
        // Test matching values (should fail if any match)
        let matching_values = vec![
            serde_json::json!("admin"),
            serde_json::json!("user"),
        ];
        assert!(!test_apply_check(&checker, matching_values, &check));
    }

    #[test]
    fn test_contains_check() {
        let checker = create_test_checker();
        
        // Test array contains
        let array_check = CheckRule::Contains { value: serde_json::json!("search") };
        let array_values = vec![serde_json::json!(["search", "auth", "api"])];
        assert!(test_apply_check(&checker, array_values, &array_check));
        
        let array_values_no_match = vec![serde_json::json!(["auth", "api"])];
        assert!(!test_apply_check(&checker, array_values_no_match, &array_check));
        
        // Test string contains
        let string_check = CheckRule::Contains { value: serde_json::json!("test") };
        let string_values = vec![serde_json::json!("testing123")];
        assert!(test_apply_check(&checker, string_values, &string_check));
        
        let string_values_no_match = vec![serde_json::json!("example")];
        assert!(!test_apply_check(&checker, string_values_no_match, &string_check));
        
        // Test object contains
        let object_check = CheckRule::Contains { value: serde_json::json!({"name": "John"}) };
        let object_values = vec![serde_json::json!({"name": "John", "age": 30})];
        assert!(test_apply_check(&checker, object_values, &object_check));
    }

    #[test]
    fn test_contained_by_check() {
        let checker = create_test_checker();
        let check = CheckRule::ContainedBy { value: serde_json::json!(["a", "b", "c", "d"]) };
        
        // Test value contained by larger array
        let contained_values = vec![serde_json::json!("b")];
        assert!(test_apply_check(&checker, contained_values, &check));
        
        // Test value not contained
        let not_contained_values = vec![serde_json::json!("z")];
        assert!(!test_apply_check(&checker, not_contained_values, &check));
    }

    #[test]
    fn test_jsonb_contains() {
        let checker = create_test_checker();

        // Test the helper function directly
        let left = serde_json::json!({"a": 1, "b": 2});
        let right = serde_json::json!({"a": 1});
        assert!(checker.jsonb_contains(&left, &right));

        let left = serde_json::json!({"a": 1});
        let right = serde_json::json!({"a": 1, "b": 2});
        assert!(!checker.jsonb_contains(&left, &right));
        
        // Test with check rule
        let check = CheckRule::JsonbContains { 
            value: serde_json::json!({"database": {"host": "localhost"}}) 
        };
        let values = vec![serde_json::json!({"database": {"host": "localhost", "port": 5432}})];
        assert!(test_apply_check(&checker, values, &check));
    }

    #[test]
    fn test_regex_check() {
        let checker = create_test_checker();
        let check = CheckRule::Regex { 
            pattern: r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string() 
        };
        
        // Test valid emails
        let valid_emails = vec![serde_json::json!("user@example.com")];
        assert!(test_apply_check(&checker, valid_emails, &check));
        
        // Test invalid emails
        let invalid_emails = vec![serde_json::json!("invalid-email")];
        assert!(!test_apply_check(&checker, invalid_emails, &check));
    }

    #[test]
    fn test_greater_than_check() {
        let checker = create_test_checker();
        let check = CheckRule::GreaterThan { value: 18.0 };
        
        let greater_values = vec![serde_json::json!(25)];
        assert!(test_apply_check(&checker, greater_values, &check));
        
        let lesser_values = vec![serde_json::json!(15)];
        assert!(!test_apply_check(&checker, lesser_values, &check));
    }

    #[test]
    fn test_less_than_check() {
        let checker = create_test_checker();
        let check = CheckRule::LessThan { value: 100.0 };
        
        let lesser_values = vec![serde_json::json!(50)];
        assert!(test_apply_check(&checker, lesser_values, &check));
        
        let greater_values = vec![serde_json::json!(150)];
        assert!(!test_apply_check(&checker, greater_values, &check));
    }

    #[test]
    fn test_array_length_check() {
        let checker = create_test_checker();
        let check = CheckRule::ArrayLength { min: Some(1), max: Some(5) };
        
        let valid_arrays = vec![serde_json::json!([1, 2, 3])];
        assert!(test_apply_check(&checker, valid_arrays, &check));
        
        let invalid_arrays = vec![serde_json::json!([])];
        assert!(!test_apply_check(&checker, invalid_arrays, &check));
    }

    #[test]
    fn test_jsonb_exists_checks() {
        let checker = create_test_checker();
        
        // Test JsonbExists
        let exists_check = CheckRule::JsonbExists { key: "email".to_string() };
        let with_key = vec![serde_json::json!({"email": "test@example.com"})];
        assert!(test_apply_check(&checker, with_key, &exists_check));
        
        let without_key = vec![serde_json::json!({"name": "John"})];
        assert!(!test_apply_check(&checker, without_key, &exists_check));
        
        // Test JsonbExistsAll
        let exists_all_check = CheckRule::JsonbExistsAll { 
            keys: vec!["email".to_string(), "name".to_string()] 
        };
        let with_all_keys = vec![serde_json::json!({"email": "test@example.com", "name": "John"})];
        assert!(test_apply_check(&checker, with_all_keys, &exists_all_check));
        
        let with_some_keys = vec![serde_json::json!({"email": "test@example.com"})];
        assert!(!test_apply_check(&checker, with_some_keys, &exists_all_check));
    }

    #[test]
    fn test_jsonb_contained_by_check() {
        let checker = create_test_checker();
        let check = CheckRule::JsonbContainedBy { 
            value: serde_json::json!({"a": 1, "b": 2, "c": 3}) 
        };
        
        let contained_values = vec![serde_json::json!({"a": 1})];
        assert!(test_apply_check(&checker, contained_values, &check));
        
        let not_contained_values = vec![serde_json::json!({"a": 1, "b": 2, "d": 4})];
        assert!(!test_apply_check(&checker, not_contained_values, &check));
    }

    #[test]
    fn test_jsonb_exists_any_check() {
        let checker = create_test_checker();
        let check = CheckRule::JsonbExistsAny { 
            keys: vec!["email".to_string(), "phone".to_string()] 
        };
        
        let with_email = vec![serde_json::json!({"email": "test@example.com", "name": "John"})];
        assert!(test_apply_check(&checker, with_email, &check));
        
        let with_phone = vec![serde_json::json!({"phone": "123-456-7890", "name": "John"})];
        assert!(test_apply_check(&checker, with_phone, &check));
        
        let without_keys = vec![serde_json::json!({"name": "John"})];
        assert!(!test_apply_check(&checker, without_keys, &check));
    }

    #[test]
    fn test_jsonb_path_match_check() {
        let checker = create_test_checker();
        let check = CheckRule::JsonbPathMatch { path: "$.users[*].email".to_string() };
        
        let values = vec![serde_json::json!({"users": [{"email": "test@example.com"}]})];
        // Currently always returns true - this is a simplified implementation
        assert!(test_apply_check(&checker, values, &check));
    }

    #[test]
    fn test_edge_cases_and_comprehensive_coverage() {
        let checker = create_test_checker();
        
        // Test empty values array for different checks
        assert!(test_apply_check(&checker, vec![], &CheckRule::Empty));
        assert!(!test_apply_check(&checker, vec![], &CheckRule::NonEmpty));
        
        // Test array length with edge cases
        let array_check = CheckRule::ArrayLength { min: None, max: Some(3) };
        let small_array = vec![serde_json::json!([1, 2])];
        assert!(test_apply_check(&checker, small_array, &array_check));
        
        // Test regex with non-string values (should fail)
        let regex_check = CheckRule::Regex { pattern: "test".to_string() };
        let non_string = vec![serde_json::json!(42)];
        assert!(!test_apply_check(&checker, non_string, &regex_check));
        
        // Test numeric comparisons with non-numeric values
        let gt_check = CheckRule::GreaterThan { value: 10.0 };
        let non_numeric = vec![serde_json::json!("not_a_number")];
        assert!(!test_apply_check(&checker, non_numeric, &gt_check));
    }

    #[test]
    fn test_helper_functions() {
        let checker = create_test_checker();

        // Test array contains
        let arr = serde_json::json!([1, 2, 3]);
        let val = serde_json::json!(2);
        assert!(checker.contains(&arr, &val));
        
        // Test string contains
        let container = serde_json::json!("hello world");
        let contained = serde_json::json!("world");
        assert!(checker.contains(&container, &contained));
        
        // Test object contains
        let obj_container = serde_json::json!({"a": 1, "b": 2, "c": 3});
        let obj_contained = serde_json::json!({"a": 1, "b": 2});
        assert!(checker.contains(&obj_container, &obj_contained));
        
        // Test is_empty_value
        assert!(checker.is_empty_value(&serde_json::json!(null)));
        assert!(checker.is_empty_value(&serde_json::json!("")));
        assert!(checker.is_empty_value(&serde_json::json!([])));
        assert!(checker.is_empty_value(&serde_json::json!({})));
        assert!(!checker.is_empty_value(&serde_json::json!("test")));
        assert!(!checker.is_empty_value(&serde_json::json!(42)));
        assert!(!checker.is_empty_value(&serde_json::json!(true)));
        assert!(!checker.is_empty_value(&serde_json::json!(false)));
    }
}