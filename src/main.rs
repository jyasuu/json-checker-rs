//! JSON Checker CLI Application
//! 
//! A command-line tool for validating JSON data against configurable rules.

use json_checker_rs::cli::Cli;
use json_checker_rs::Result;

fn main() -> Result<()> {
    // In a real CLI, you'd parse command-line arguments here
    // For now, we'll use the default config file
    Cli::run(None)
}