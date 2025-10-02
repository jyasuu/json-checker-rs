# JSON Checker RS 🔍

A powerful, flexible JSON validation tool built in Rust that validates JSON data against configurable rules using JSONPath queries.

## ✨ Features

### Core Validation Rules

| Rule Type | Description | Example Use Case |
|-----------|-------------|------------------|
| `empty` | Validates values are empty (null, "", [], {}) | Ensure optional fields are not set |
| `non_empty` | Validates values are not empty | Required field validation |
| `equals` | Validates values equal a specific value | Enum validation, status checks |
| `not_equals` | Validates values don't equal a value | Blacklist validation |
| `contains` | Validates containers include a value | Array membership, string contains |
| `contained_by` | Validates value is contained by another | Whitelist validation |
| `regex` | Validates strings match a regex pattern | Email, phone, format validation |
| `greater_than` | Validates numbers are above threshold | Age limits, size constraints |
| `less_than` | Validates numbers are below threshold | Maximum limits |
| `array_length` | Validates array length constraints | Collection size validation |

### PostgreSQL-Style JSONB Operations

| Operation | PostgreSQL | Description | Use Case |
|-----------|------------|-------------|----------|
| `jsonb_contains` | `@>` | Left JSON contains right JSON | Configuration validation |
| `jsonb_contained_by` | `<@` | Left JSON contained by right | Subset validation |
| `jsonb_exists` | `?` | Key exists in JSON object | Required property check |
| `jsonb_exists_any` | `?|` | Any of the keys exist | Optional property groups |
| `jsonb_exists_all` | `?&` | All keys exist | Required property sets |
| `jsonb_path_match` | `@@` | JSONPath expression match | Complex path validation |

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd json-checker-rs

# Build the project
cargo build --release
```

### Basic Usage

1. **Create a rules configuration file** (`rules.json`):

```json
{
  "rules": [
    {
      "name": "User emails must be valid",
      "json_file": "data/users.json",
      "jsonpath": "$.users[*].email",
      "check": {
        "type": "regex",
        "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
      }
    }
  ]
}
```

2. **Run validation**:

```bash
# Using the binary
./target/release/json-checker-rs

# Using cargo
cargo run
```

3. **View results**:

```
🔍 Starting JSON Checker...
📄 Using config file: rules.json

=== JSON Checker Results ===

✓ Rule 'User emails must be valid' passed

----------------------------
📊 Summary:
   Total: 1 rules
   Passed: 1 ✅
   Failed: 0 ❌

🎉 All validation rules passed!
```

## 📖 Rule Configuration Reference

### Rule Structure

```json
{
  "name": "Human-readable rule description",
  "json_file": "path/to/target.json",
  "jsonpath": "JSONPath query expression",
  "check": {
    "type": "rule_type",
    // Additional parameters based on rule type
  }
}
```

### JSONPath Examples

| Pattern | Description | Matches |
|---------|-------------|---------|
| `$.users[*].email` | All user emails | Every email in users array |
| `$[?(@.type=='admin')]` | Objects where type is admin | Admin objects |
| `$.config.features` | Features array in config | Configuration features |
| `$..[?(@.required==true)]` | All required fields | Any required field at any level |
| `$.products[?(@.price>100)]` | Expensive products | Products over $100 |

### Check Rule Examples

#### String Validation
```json
{
  "type": "regex",
  "pattern": "^[A-Z]{2,3}-\\d{4}$"
}
```

#### Numeric Validation
```json
{
  "type": "greater_than",
  "value": 18
}
```

#### Array Validation
```json
{
  "type": "array_length",
  "min": 1,
  "max": 10
}
```

#### Object Validation
```json
{
  "type": "jsonb_exists_all",
  "keys": ["name", "email", "id"]
}
```

## 🏗️ Architecture

### Project Structure

```
src/
├── lib.rs          # Library entry point & public API
├── main.rs         # CLI application binary
├── config.rs       # Configuration structures
├── checker.rs      # Core validation engine
├── rules.rs        # Rule type definitions
├── cli.rs          # Command-line interface
└── tests.rs        # Comprehensive test suite
```

### Library Usage

```rust
use json_checker_rs::{JsonChecker, Config, CheckRule};

// From config file
let checker = JsonChecker::new("rules.json")?;
let results = checker.run()?;

// Programmatic configuration
let config = Config {
    rules: vec![
        Rule {
            name: "Custom validation".to_string(),
            json_file: "data.json".to_string(),
            jsonpath: "$.field".to_string(),
            check: CheckRule::NonEmpty,
        }
    ]
};
let checker = JsonChecker::from_config(config);
```

## 🎯 Real-World Use Cases

### Configuration Validation
Ensure your JSON configuration files meet requirements:

```json
{
  "name": "Database config must have required fields",
  "json_file": "config/database.json",
  "jsonpath": "$",
  "check": {
    "type": "jsonb_exists_all",
    "keys": ["host", "port", "database", "username"]
  }
}
```

### API Response Validation
Validate API responses match expected structure:

```json
{
  "name": "API response has success status",
  "json_file": "response.json",
  "jsonpath": "$.status",
  "check": {
    "type": "equals",
    "value": "success"
  }
}
```

### Data Quality Checks
Ensure data integrity in JSON datasets:

```json
{
  "name": "All products have valid prices",
  "json_file": "products.json",
  "jsonpath": "$.products[*].price",
  "check": {
    "type": "greater_than",
    "value": 0
  }
}
```

### UI Component Validation
Validate UI component configurations:

```json
{
  "name": "DateBox components should use Date-notime",
  "json_file": "ui-config.json",
  "jsonpath": "$..[?(@.xtype=='DateBox')].dataType",
  "check": {
    "type": "equals",
    "value": "Date-notime"
  }
}
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
cargo test
```

The project includes 17+ unit tests covering:
- All rule types and edge cases
- JSONPath query validation
- Helper function testing
- Error condition handling

## 🔧 Development

### Adding New Rule Types

1. Add the rule variant to `CheckRule` enum in `src/rules.rs`
2. Implement validation logic in `src/checker.rs`
3. Add comprehensive tests in `src/tests.rs`

### Extending CLI Features

Modify `src/cli.rs` to add new command-line options and output formats.

## 📊 Performance

- **Fast validation**: Efficient JSONPath processing
- **Memory efficient**: Streaming JSON processing where possible
- **Concurrent capable**: Ready for parallel rule execution
- **Minimal dependencies**: Clean dependency tree

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

[Add your license information here]

## 🆘 Support

- **Issues**: Report bugs and feature requests via GitHub Issues
- **Documentation**: This README and inline code documentation
- **Examples**: Check the `examples/` directory for more use cases

---

**JSON Checker RS** - Making JSON validation simple, powerful, and reliable! 🚀