//! Epic management tool handlers

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS};

#[derive(Deserialize)]
struct AddTaskEpicArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    task_id: String,
    epic: String,
}

#[derive(Deserialize)]
struct RemoveTaskEpicArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    task_id: String,
    epic: String,
}

fn default_file_path() -> String {
    "tasks.toml".to_string()
}

pub fn handle_add_task_epic_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: AddTaskEpicArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for add_task_epic: {}", e),
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

    if args.epic.trim().is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "epic cannot be empty".to_string(),
            data: None,
        });
    }

    let task_id = crate::normalize_task_id(&args.task_id);
    let output = crate::handle_add_epic(
        &validated_path,
        &task_id,
        &args.epic,
        true,
    )?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}

pub fn handle_remove_task_epic_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: RemoveTaskEpicArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for remove_task_epic: {}", e),
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

    if args.epic.trim().is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "epic cannot be empty".to_string(),
            data: None,
        });
    }

    let task_id = crate::normalize_task_id(&args.task_id);
    let output = crate::handle_del_epic(
        &validated_path,
        &task_id,
        &args.epic,
        true,
    )?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}
