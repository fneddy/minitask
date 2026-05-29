//! MCP resources implementation for task access

use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS, INTERNAL_ERROR};

/// Handles resources/list request
pub fn handle_resources_list(tasks_file: &str) -> Result<Value, JsonRpcError> {
    // Load tasks to get current task list
    let task_file = crate::load_tasks(std::path::Path::new(tasks_file))
        .map_err(|e| JsonRpcError {
            code: INTERNAL_ERROR,
            message: format!("Failed to load tasks: {}", e),
            data: None,
        })?;

    let mut resources = Vec::new();

    // Add resource for each task
    for task in &task_file.tasks {
        resources.push(serde_json::json!({
            "uri": format!("task://{}", task.name),
            "name": format!("Task {}", task.name),
            "description": format!("Full task details for {}", task.name),
            "mimeType": "application/json"
        }));

        resources.push(serde_json::json!({
            "uri": format!("task://{}/content", task.name),
            "name": format!("Task {} content", task.name),
            "description": format!("Content only for {}", task.name),
            "mimeType": "text/plain"
        }));
    }

    Ok(serde_json::json!({ "resources": resources }))
}

/// Handles resources/read request
pub fn handle_resources_read(tasks_file: &str, uri: &str) -> Result<Value, JsonRpcError> {
    // Parse URI: task://TASK-ID or task://TASK-ID/content
    if !uri.starts_with("task://") {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: format!("Invalid URI scheme. Expected 'task://', got '{}'", uri),
            data: None,
        });
    }

    let path = &uri[7..]; // Remove "task://"
    let parts: Vec<&str> = path.split('/').collect();

    if parts.is_empty() || parts[0].is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "Invalid URI: missing task ID".to_string(),
            data: None,
        });
    }

    let task_id = parts[0];
    let content_only = parts.len() > 1 && parts[1] == "content";

    // Load task
    let task = crate::get_task(std::path::Path::new(tasks_file), task_id)?;

    if content_only {
        // Return just the content as text
        Ok(serde_json::json!({
            "contents": [{
                "uri": uri,
                "mimeType": "text/plain",
                "text": task.content
            }]
        }))
    } else {
        // Return full task as JSON
        let task_json = serde_json::to_string_pretty(&task)
            .map_err(|e| JsonRpcError {
                code: INTERNAL_ERROR,
                message: format!("Failed to serialize task: {}", e),
                data: None,
            })?;

        Ok(serde_json::json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": task_json
            }]
        }))
    }
}
