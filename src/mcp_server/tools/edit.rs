//! Edit task tool handlers

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS};

#[derive(Deserialize)]
struct EditTaskStateArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    task_id: String,
    state: String,
}

#[derive(Deserialize)]
struct EditTaskContentArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    task_id: String,
    content: String,
}

#[derive(Deserialize)]
struct AddTaskContentArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    task_id: String,
    content: String,
}

fn default_file_path() -> String {
    "tasks.toml".to_string()
}

pub fn handle_edit_task_state_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: EditTaskStateArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for edit_task_state: {}", e),
        data: None,
    })?;

    let file_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    if args.task_id.trim().is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "task_id cannot be empty".to_string(),
            data: None,
        });
    }

    let task_id = crate::normalize_task_id(&args.task_id);
    let output = crate::handle_edit_state(
        &file_path,
        &task_id,
        &args.state,
        true,
    )?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}

pub fn handle_edit_task_content_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: EditTaskContentArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for edit_task_content: {}", e),
        data: None,
    })?;

    let validated_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    if args.task_id.trim().is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "task_id cannot be empty".to_string(),
            data: None,
        });
    }

    let task_id = crate::normalize_task_id(&args.task_id);
    let output = crate::handle_edit_content(
        &validated_path,
        &task_id,
        &args.content,
        true,
    )?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}

pub fn handle_add_task_content_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: AddTaskContentArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for add_task_content: {}", e),
        data: None,
    })?;

    let validated_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    if args.task_id.trim().is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "task_id cannot be empty".to_string(),
            data: None,
        });
    }

    let task_id = crate::normalize_task_id(&args.task_id);
    let output = crate::handle_add_content(
        &validated_path,
        &task_id,
        &args.content,
        true,
    )?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}
