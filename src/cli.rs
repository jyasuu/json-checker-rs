//! Command-line interface for the JSON checker

use anyhow::Result;
use crate::checker::JsonChecker;

/// CLI application entry point
pub struct Cli;

impl Cli {
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
            }
        }

        println!("\n----------------------------");
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