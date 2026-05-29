# MCP Server Documentation

## Overview

The `minitask` MCP (Model Context Protocol) server provides task management capabilities through a JSON-RPC 2.0 interface over stdio. It implements the MCP protocol specification version `2024-11-05` and exposes 11 tools for managing tasks, plus resource URIs for accessing task data.

## Quick Start

### Starting the Server

```bash
minitask mcp
```

The server runs in stdio mode, reading JSON-RPC requests from stdin and writing responses to stdout.

### Configuration

#### Bob Shell Configuration

Add to `~/.bob/config.json`:

```json
{
  "mcpServers": {
    "minitask": {
      "command": "minitask",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

#### Claude Desktop Configuration

Add to Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "minitask": {
      "command": "/path/to/minitask",
      "args": ["mcp"]
    }
  }
}
```

Replace `/path/to/minitask` with the actual path to your minitask binary (e.g., `~/.cargo/bin/minitask`).

## Available Tools

### 1. list_tasks

Lists tasks with optional filtering by state and epic.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `state` (string, optional): Filter tasks by state. Must be one of: `todo`, `in-progress`, or `done`.
- `epic` (string, optional): Filter tasks by epic name.
- `verbose` (boolean, optional): Show verbose output with full task details. Defaults to `false`.

**Example:**
```json
{
  "name": "list_tasks",
  "arguments": {
    "state": "todo",
    "verbose": true
  }
}
```

### 2. show_task

Shows details of a specific task by ID.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to show (e.g., `TASK-0` or `0`).
- `verbose` (boolean, optional): Show verbose output with full task details. Defaults to `false`.

**Example:**
```json
{
  "name": "show_task",
  "arguments": {
    "task_id": "TASK-5",
    "verbose": true
  }
}
```

### 3. create_task

Creates a new task with the given content.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `content` (string, required): Task content (description and details).

**Example:**
```json
{
  "name": "create_task",
  "arguments": {
    "content": "Implement user authentication\n\nAdd JWT-based authentication with refresh tokens."
  }
}
```

**Returns:** JSON representation of the newly created task.

### 4. edit_task_state

Updates a task's state and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `state` (string, required): New task state. Must be one of: `todo`, `in-progress`, or `done`.

**Example:**
```json
{
  "name": "edit_task_state",
  "arguments": {
    "task_id": "TASK-3",
    "state": "in-progress"
  }
}
```

**Returns:** JSON representation of the updated task.

### 5. edit_task_content

Replaces a task's content and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `content` (string, required): Replacement task content.

**Example:**
```json
{
  "name": "edit_task_content",
  "arguments": {
    "task_id": "TASK-2",
    "content": "Updated task description\n\nWith new implementation details."
  }
}
```

**Returns:** JSON representation of the updated task.

### 6. add_task_content

Appends content to existing task content and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `content` (string, required): Content to append to existing task content.

**Example:**
```json
{
  "name": "add_task_content",
  "arguments": {
    "task_id": "TASK-1",
    "content": "\n\nAdditional notes: Remember to update tests."
  }
}
```

**Returns:** JSON representation of the updated task.

### 7. add_task_dependency

Adds a dependency between tasks and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `depends_on` (string, required): Task ID that this task depends on (e.g., `TASK-1` or `1`).

**Example:**
```json
{
  "name": "add_task_dependency",
  "arguments": {
    "task_id": "TASK-5",
    "depends_on": "TASK-3"
  }
}
```

**Returns:** JSON representation of the updated task.

### 8. remove_task_dependency

Removes a dependency between tasks and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `depends_on` (string, required): Task ID dependency to remove (e.g., `TASK-1` or `1`).

**Example:**
```json
{
  "name": "remove_task_dependency",
  "arguments": {
    "task_id": "TASK-5",
    "depends_on": "TASK-3"
  }
}
```

**Returns:** JSON representation of the updated task.

### 9. add_task_epic

Adds an epic to a task and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `epic` (string, required): Epic name to add to the task.

**Example:**
```json
{
  "name": "add_task_epic",
  "arguments": {
    "task_id": "TASK-7",
    "epic": "authentication"
  }
}
```

**Returns:** JSON representation of the updated task.

### 10. remove_task_epic

Removes an epic from a task and returns the updated task as JSON.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `task_id` (string, required): Task ID to update (e.g., `TASK-0` or `0`).
- `epic` (string, required): Epic name to remove from the task.

**Example:**
```json
{
  "name": "remove_task_epic",
  "arguments": {
    "task_id": "TASK-7",
    "epic": "authentication"
  }
}
```

**Returns:** JSON representation of the updated task.

### 11. claim_task

Claims the next available task matching filters and moves it to a new state.

**Parameters:**
- `file_path` (string, optional): Path to the tasks file. Defaults to `tasks.toml`.
- `new_state` (string, required): New state for the claimed task.
- `state` (string, optional): Filter by source state. Defaults to `todo`.
- `epic` (string, optional): Filter by epic name.

**Example:**
```json
{
  "name": "claim_task",
  "arguments": {
    "new_state": "in-progress",
    "state": "todo",
    "epic": "backend"
  }
}
```

**Returns:** JSON representation of the claimed task, or an error if no tasks are available.

## Resource URIs

The MCP server exposes task data through resource URIs that can be read by MCP clients.

### URI Patterns

#### Full Task Data
```
task://TASK-ID
```
Returns complete task information as JSON, including name, state, dependencies, epics, and content.

**Example:** `task://TASK-0`

**Response:**
```json
{
  "name": "TASK-0",
  "state": "in-progress",
  "depends_on": ["TASK-1"],
  "epic": ["backend"],
  "content": "Implement user authentication\n\nAdd JWT-based authentication."
}
```

#### Content Only
```
task://TASK-ID/content
```
Returns only the task content as plain text.

**Example:** `task://TASK-0/content`

**Response:**
```
Implement user authentication

Add JWT-based authentication.
```

### Listing Resources

Use the `resources/list` method to get all available task resources:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list",
  "params": {}
}
```

## Task ID Normalization

Task IDs can be specified in two formats:
- Full format: `TASK-0`, `TASK-1`, etc.
- Short format: `0`, `1`, etc.

The server automatically normalizes short format IDs to full format (e.g., `5` becomes `TASK-5`).

## Security Features

### Path Traversal Protection

All file path parameters are validated to prevent path traversal attacks. Paths containing `..` are rejected with an error.

### Message Size Limits

The server enforces a maximum message size of 10MB to prevent denial-of-service attacks.

### Input Validation

- Task IDs cannot be empty or whitespace-only
- State values must be one of: `todo`, `in-progress`, `done`
- Epic names cannot be empty or whitespace-only
- Content cannot be empty or whitespace-only for create operations

## Error Handling

The server returns standard JSON-RPC 2.0 error responses:

### Error Codes

- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid request (e.g., server not initialized)
- `-32601`: Method not found
- `-32602`: Invalid params (e.g., invalid state, missing task)
- `-32603`: Internal error (e.g., file I/O error)

### Example Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid state 'invalid'. Must be 'todo', 'in-progress', or 'done'"
  }
}
```

## Protocol Flow

### 1. Initialize

Client sends initialize request:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "my-client",
      "version": "1.0.0"
    }
  }
}
```

Server responds with capabilities:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {
        "listChanged": false
      },
      "resources": {
        "subscribe": false,
        "listChanged": false
      }
    },
    "serverInfo": {
      "name": "minitask",
      "version": "0.1.0"
    }
  }
}
```

### 2. Initialized Notification

Client sends initialized notification:

```json
{
  "jsonrpc": "2.0",
  "method": "initialized",
  "params": {}
}
```

### 3. Tool Calls

After initialization, client can call tools:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "list_tasks",
    "arguments": {
      "state": "todo"
    }
  }
}
```

## Troubleshooting

### Server Not Responding

**Problem:** No response from server after sending requests.

**Solution:** 
- Ensure the server is properly initialized with the `initialize` request before sending other requests
- Check that the `initialized` notification was sent after receiving the initialize response
- Verify that JSON-RPC messages are properly formatted with `\n` line endings

### Task Not Found Errors

**Problem:** Getting "task not found" errors when task exists.

**Solution:**
- Verify the task ID format (use either `TASK-0` or `0`)
- Check that the `file_path` parameter points to the correct tasks file
- Ensure the tasks file is valid TOML format

### Permission Denied Errors

**Problem:** Cannot read or write tasks file.

**Solution:**
- Check file permissions on the tasks file
- Ensure the directory containing the tasks file is writable
- Verify the user running the MCP server has appropriate permissions

### Invalid State Errors

**Problem:** Getting "invalid state" errors.

**Solution:**
- Ensure state values are exactly one of: `todo`, `in-progress`, or `done`
- Check for typos or extra whitespace in state values
- State values are case-sensitive

### Path Traversal Errors

**Problem:** Getting "path traversal not allowed" errors.

**Solution:**
- Do not use `..` in file paths
- Use absolute paths or paths relative to the current working directory
- Avoid symbolic links that point outside the working directory

## Best Practices

### Task Management Workflow

1. **List available tasks**: Use `list_tasks` with `state: "todo"` to see what's available
2. **Claim a task**: Use `claim_task` to atomically move a task to `in-progress`
3. **Work on the task**: Implement, test, and document
4. **Update task state**: Use `edit_task_state` to mark as `done`
5. **Add notes**: Use `add_task_content` to append progress notes or findings

### Dependency Management

- Add dependencies before starting work to prevent conflicts
- Use `claim_task` which respects dependencies automatically
- Remove dependencies only when the blocking task is complete

### Epic Organization

- Use epics to group related tasks
- Filter by epic when claiming tasks to focus on specific features
- Keep epic names consistent across tasks

### Content Formatting

- Use clear, concise task descriptions
- Include implementation details in the content
- Add test criteria and acceptance criteria
- Use newlines to separate sections

## Examples

### Complete Workflow Example

```json
// 1. List available tasks
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_tasks",
    "arguments": {
      "state": "todo",
      "epic": "authentication"
    }
  }
}

// 2. Claim a task
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "claim_task",
    "arguments": {
      "new_state": "in-progress",
      "epic": "authentication"
    }
  }
}

// 3. Add progress notes
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add_task_content",
    "arguments": {
      "task_id": "TASK-5",
      "content": "\n\nProgress: Implemented JWT generation and validation."
    }
  }
}

// 4. Mark as done
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "edit_task_state",
    "arguments": {
      "task_id": "TASK-5",
      "state": "done"
    }
  }
}
```

### Creating a Task with Dependencies

```json
// 1. Create the task
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_task",
    "arguments": {
      "content": "Implement password reset flow\n\nRequires email service to be set up first."
    }
  }
}

// 2. Add dependency
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "add_task_dependency",
    "arguments": {
      "task_id": "TASK-10",
      "depends_on": "TASK-8"
    }
  }
}

// 3. Add epic
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add_task_epic",
    "arguments": {
      "task_id": "TASK-10",
      "epic": "authentication"
    }
  }
}
```

## Version History

- **0.1.0**: Initial MCP server implementation with 11 tools and resource URIs
