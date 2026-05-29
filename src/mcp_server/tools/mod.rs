//! MCP tool handlers module

pub mod list;
pub mod show;
pub mod create;
pub mod edit;
pub mod dependency;
pub mod epic;
pub mod claim;

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, METHOD_NOT_FOUND, INVALID_PARAMS};

#[derive(Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

/// Routes a tool call to the appropriate handler
pub fn route_tool_call(params: Value) -> Result<Value, JsonRpcError> {
    let tool_params: ToolCallParams = serde_json::from_value(params).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid tool call params: {}", e),
        data: None,
    })?;

    match tool_params.name.as_str() {
        "list_tasks" => list::handle_list_tasks_tool(tool_params.arguments),
        "show_task" => show::handle_show_task_tool(tool_params.arguments),
        "create_task" => create::handle_create_task_tool(tool_params.arguments),
        "edit_task_state" => edit::handle_edit_task_state_tool(tool_params.arguments),
        "edit_task_content" => edit::handle_edit_task_content_tool(tool_params.arguments),
        "add_task_content" => edit::handle_add_task_content_tool(tool_params.arguments),
        "add_task_dependency" => dependency::handle_add_task_dependency_tool(tool_params.arguments),
        "remove_task_dependency" => dependency::handle_remove_task_dependency_tool(tool_params.arguments),
        "add_task_epic" => epic::handle_add_task_epic_tool(tool_params.arguments),
        "remove_task_epic" => epic::handle_remove_task_epic_tool(tool_params.arguments),
        "claim_task" => claim::handle_claim_task_tool(tool_params.arguments),
        _ => Err(JsonRpcError {
            code: METHOD_NOT_FOUND,
            message: format!("Unknown tool: {}", tool_params.name),
            data: None,
        }),
    }
}

/// Returns the list of available tools with their schemas
pub fn get_tools_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "list_tasks",
            "description": "List tasks with optional filtering by state and epic",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "state": {
                        "type": "string",
                        "description": format!("Filter tasks by state (must be one of: {})", crate::VALID_STATES.join(", ")),
                        "enum": crate::VALID_STATES.iter().map(|s| s.to_string()).collect::<Vec<_>>()
                    },
                    "epic": {
                        "type": "string",
                        "description": "Filter tasks by epic name"
                    },
                    "verbose": {
                        "type": "boolean",
                        "description": "Show verbose output with full task details",
                        "default": false
                    }
                }
            }
        }),
        serde_json::json!({
            "name": "show_task",
            "description": "Show details of a specific task by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to show (e.g., 'TASK-0' or '0')"
                    },
                    "verbose": {
                        "type": "boolean",
                        "description": "Show verbose output with full task details",
                        "default": false
                    }
                },
                "required": ["task_id"]
            }
        }),
        serde_json::json!({
            "name": "create_task",
            "description": "Create a new task with the given content",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "content": {
                        "type": "string",
                        "description": "Task content (description and details)"
                    }
                },
                "required": ["content"]
            }
        }),
        serde_json::json!({
            "name": "edit_task_state",
            "description": "Update task state and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "state": {
                        "type": "string",
                        "description": format!("New task state (must be one of: {})", crate::VALID_STATES.join(", ")),
                        "enum": crate::VALID_STATES.iter().map(|s| s.to_string()).collect::<Vec<_>>()
                    }
                },
                "required": ["task_id", "state"]
            }
        }),
        serde_json::json!({
            "name": "edit_task_content",
            "description": "Replace task content and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "content": {
                        "type": "string",
                        "description": "Replacement task content"
                    }
                },
                "required": ["task_id", "content"]
            }
        }),
        serde_json::json!({
            "name": "add_task_content",
            "description": "Append content to existing task content and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to append to existing task content"
                    }
                },
                "required": ["task_id", "content"]
            }
        }),
        serde_json::json!({
            "name": "add_task_dependency",
            "description": "Add a dependency between tasks and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "depends_on": {
                        "type": "string",
                        "description": "Task ID that this task depends on (e.g., 'TASK-1' or '1')"
                    }
                },
                "required": ["task_id", "depends_on"]
            }
        }),
        serde_json::json!({
            "name": "remove_task_dependency",
            "description": "Remove a dependency between tasks and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "depends_on": {
                        "type": "string",
                        "description": "Task ID dependency to remove (e.g., 'TASK-1' or '1')"
                    }
                },
                "required": ["task_id", "depends_on"]
            }
        }),
        serde_json::json!({
            "name": "add_task_epic",
            "description": "Add an epic to a task and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "epic": {
                        "type": "string",
                        "description": "Epic name to add to the task"
                    }
                },
                "required": ["task_id", "epic"]
            }
        }),
        serde_json::json!({
            "name": "remove_task_epic",
            "description": "Remove an epic from a task and return updated task JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to update (e.g., 'TASK-0' or '0')"
                    },
                    "epic": {
                        "type": "string",
                        "description": "Epic name to remove from the task"
                    }
                },
                "required": ["task_id", "epic"]
            }
        }),
        serde_json::json!({
            "name": "claim_task",
            "description": "Claim the next available task matching filters and move it to a new state",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the tasks file (defaults to 'tasks.toml')",
                        "default": "tasks.toml"
                    },
                    "new_state": {
                        "type": "string",
                        "description": "New state for the claimed task"
                    },
                    "state": {
                        "type": "string",
                        "description": "Filter by source state (defaults to 'todo')",
                        "default": "todo"
                    },
                    "epic": {
                        "type": "string",
                        "description": "Filter by epic name"
                    }
                },
                "required": ["new_state"]
            }
        })
    ]
}
