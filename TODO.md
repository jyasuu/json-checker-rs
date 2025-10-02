# help me add more unit test for each check rule


## example test data

```json
{
  "rules": [
    {
      "name": "Type 'b' value must not be empty",
      "json_file": "data/sample.json",
      "jsonpath": "$.data[?(@.type=='b')].val",
      "check": {
        "type": "non_empty"
      }
    },
    {
      "name": "User email must not be empty",
      "json_file": "data/users.json",
      "jsonpath": "$.users[*].email",
      "check": {
        "type": "non_empty"
      }
    },
    {
      "name": "Admin role must be 'admin'",
      "json_file": "data/users.json",
      "jsonpath": "$.users[?(@.role=='admin')].role",
      "check": {
        "type": "equals",
        "value": "admin"
      }
    },
    {
      "name": "Config must contain database settings",
      "json_file": "data/config.json",
      "jsonpath": "$",
      "check": {
        "type": "jsonb_contains",
        "value": {
          "database": {
            "host": "localhost"
          }
        }
      }
    },
    {
      "name": "Features must include 'search'",
      "json_file": "data/config.json",
      "jsonpath": "$.features",
      "check": {
        "type": "contains",
        "value": "search"
      }
    },
    {
      "name": "User metadata must have email and name",
      "json_file": "data/users.json",
      "jsonpath": "$.users[*].metadata",
      "check": {
        "type": "jsonb_exists_all",
        "keys": ["email", "name"]
      }
    },
    {
      "name": "Age must be greater than 18",
      "json_file": "data/users.json",
      "jsonpath": "$.users[*].age",
      "check": {
        "type": "greater_than",
        "value": 18
      }
    },
    {
      "name": "Email format validation",
      "json_file": "data/users.json",
      "jsonpath": "$.users[*].email",
      "check": {
        "type": "regex",
        "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
      }
    },
    {
      "name": "Tags array must have 1-5 items",
      "json_file": "data/posts.json",
      "jsonpath": "$.posts[*].tags",
      "check": {
        "type": "array_length",
        "min": 1,
        "max": 5
      }
    },
    {
      "name": "Optional field should be empty",
      "json_file": "data/config.json",
      "jsonpath": "$.deprecated_settings",
      "check": {
        "type": "empty"
      }
    }
  ]
}

// Example data/sample.json (Your case):
{
  "data": [
    {
      "type": "a",
      "val": "a1"
    },
    {
      "type": "b",
      "val": ""
    }
  ]
}

// Example data/users.json:
{
  "users": [
    {
      "email": "user@example.com",
      "role": "admin",
      "age": 25,
      "metadata": {
        "email": "user@example.com",
        "name": "John Doe"
      }
    },
    {
      "email": "another@example.com",
      "role": "user",
      "age": 30,
      "metadata": {
        "email": "another@example.com",
        "name": "Jane Smith"
      }
    }
  ]
}

// Example data/config.json:
{
  "database": {
    "host": "localhost",
    "port": 5432
  },
  "features": ["search", "auth", "api"]
}

// Example data/posts.json:
{
  "posts": [
    {
      "title": "First Post",
      "tags": ["rust", "programming", "web"]
    }
  ]
}
```