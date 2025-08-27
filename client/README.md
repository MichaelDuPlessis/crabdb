# CrabDB Python Client

A Python client for interacting with CrabDB using the binary TCP protocol.

## Features

- **Interactive Mode**: Full-featured REPL with command history and graceful Ctrl+C handling
- **Single Command Mode**: Execute individual commands and exit
- **Type Support**: All CrabDB data types (Null, Int, Text, List, Map, Link)
- **Link Resolution**: Automatic link resolution with configurable depth
- **JSON Support**: Parse JSON values for complex data structures

## Usage

### Interactive Mode

```bash
python cli.py
```

This starts an interactive session where you can execute multiple commands:

```
CrabDB Interactive Client
Type 'help' for commands or 'quit' to exit
Press Ctrl+C to exit gracefully
Connected to localhost:7227
crabdb> set user1 "John Doe"
Set user1 = "John Doe"
crabdb> set user2 {"name": "Jane", "age": 30}
Set user2 = {
  "name": "Jane",
  "age": 30
}
crabdb> get user1
"John Doe"
crabdb> quit
```

### Single Command Mode

```bash
python3 cli.py --command "get user1"
python3 cli.py -c "set numbers [1, 2, 3, 4, 5]"
```

### Connection Options

```bash
python3 cli.py --host 192.168.1.100 --port 7227
```

## Commands

- `get <key> [link_depth]` - Get value by key, optionally resolve links
- `set <key> <value>` - Set key to value (supports JSON)
- `delete <key>` - Delete key
- `link <key> <target_key>` - Create a link from key to target_key
- `help` - Show available commands
- `quit`/`exit` - Exit the client

## Examples

### Basic Operations
```
set name "Alice"
set age 25
get name
delete age
```

### Complex Data Types
```
set user {"name": "Bob", "email": "bob@example.com", "active": true}
set tags ["python", "database", "rust"]
set config {"debug": false, "max_connections": 100}
```

### Links and Resolution
```
set user1 {"name": "Alice", "role": "admin"}
set user2 {"name": "Bob", "role": "user"}
link current_user user1
get current_user        # Returns link object
get current_user 1      # Resolves link and returns user1 data
```

### Links in Maps and Lists
```
# Create target data first
set user1 {"name": "Alice", "role": "admin"}
set user2 {"name": "Bob", "role": "user"}

# Links as map field values
set team {
  "lead": {"_link": "user1"},
  "member": {"_link": "user2"},
  "project": "CrabDB"
}

# Links in lists
set user_list [
  {"_link": "user1"},
  {"_link": "user2"},
  "some string",
  42
]

# Complex nested structure with links
set organization {
  "name": "TechCorp",
  "departments": [
    {
      "name": "Engineering", 
      "head": {"_link": "user1"},
      "members": [{"_link": "user2"}, {"_link": "user1"}]
    }
  ],
  "ceo": {"_link": "user1"}
}

# Resolve links at different depths
get team 1              # Resolves direct links
get organization 2      # Resolves nested links
```

## Requirements

- Python 3.6+
- No external dependencies (uses only standard library)

## Error Handling

The client handles various error conditions gracefully:
- Connection failures
- Server errors
- Invalid commands
- Malformed data
- Ctrl+C interruption (graceful shutdown)
