//! Command-line interface for the JSON checker

use anyhow::Result;
use serde_json::Value;
use crate::checker::JsonChecker;
use crate::path_finder::find_json_path;

/// CLI application entry point
pub struct Cli;

impl Cli {
    /// Find and print the path to a specific value in a JSON file
    /// This implements the exact functionality requested in TODO.md
    pub fn find_json_node_path(json_file: &str, target_value: &str) -> Result<()> {
        println!("🔍 Finding JSON node path...");
        println!("📄 JSON file: {}", json_file);
        println!("🎯 Target value: \"{}\"", target_value);
        
        // Read and parse JSON file
        let json_content = std::fs::read_to_string(json_file)?;
        let json: Value = serde_json::from_str(&json_content)?;
        
        // Create target value for searching
        let target = Value::String(target_value.to_string());
        
        // Find the path using the exact algorithm from TODO.md
        if let Some(path) = find_json_path(&json, &target, vec!["$".to_string()]) {
            println!("✅ Found path: {}", path);
        } else {
            println!("❌ Value not found.");
        }
        
        Ok(())
    }
    
    /// Run the CLI application with the given arguments
    pub fn run(config_path: Option<&str>) -> Result<()> {
        let config_file = config_path.unwrap_or("rules.json");
        
        println!("🔍 Starting JSON Checker...");
        println!("📄 Using config file: {}", config_file);
        
        let checker = JsonChecker::new(config_file)?;
        let results = checker.run()?;

        Self::print_results(&results);
        
        let failed_count = results.iter().filter(|r| !r.passed).count();
        if failed_count > 0 {
            std::process::exit(1);
        }

        Ok(())
    }

    /// Print validation results in a formatted way
    fn print_results(results: &[crate::config::CheckResult]) {
        println!("\n=== JSON Checker Results ===\n");
        
        let mut passed = 0;
        let mut failed = 0;

        for result in results {
            println!("{}", result.message);
            
            if result.passed {
                passed += 1;
            } else {
                failed += 1;
                
                // Print detailed failure information
                if !result.invalid_positions.is_empty() {
                    println!("   📍 Invalid positions found:");
                    for position in &result.invalid_positions {
                        println!("      • {}", position);
                    }
                }
                
                if !result.values_found.is_empty() {
                    println!("   📄 Values found:");
                    for (i, value) in result.values_found.iter().enumerate() {
                        let value_str = match value {
                            serde_json::Value::String(s) => format!("\"{}\"", s),
                            _ => value.to_string(),
                        };
                        println!("      [{}] {}", i, value_str);
                    }
                }
                
                if !result.invalid_positions.is_empty() || !result.values_found.is_empty() {
                    println!(); // Add spacing after detailed info
                }
            }
        }

        println!("----------------------------");
        println!("📊 Summary:");
        println!("   Total: {} rules", results.len());
        println!("   Passed: {} ✅", passed);
        println!("   Failed: {} ❌", failed);
        
        if failed > 0 {
            println!("\n⚠️  Some validation rules failed!");
        } else {
            println!("\n🎉 All validation rules passed!");
        }
    }
}