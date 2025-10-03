//! JSON Checker CLI Application
//! 
//! A command-line tool for validating JSON data against configurable rules.

use json_checker_rs::cli::Cli;
use json_checker_rs::Result;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let config_file = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };
    
    Cli::run(config_file)
}