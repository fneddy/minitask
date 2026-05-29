//! Show task tool handler

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS, INTERNAL_ERROR};

#[derive(Deserialize)]
struct ShowTaskArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    task_id: String,
    #[serde(default)]
    verbose: bool,
}

fn default_file_path() -> String {
    "tasks.toml".to_string()
}

pub fn handle_show_task_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: ShowTaskArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for show_task: {}", e),
        data: None,
    })?;

    if args.task_id.trim().is_empty() {
        return Err(JsonRpcError {
            code: INVALID_PARAMS,
            message: "task_id cannot be empty".to_string(),
            data: None,
        });
    }

    let file_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    let task_id = crate::normalize_task_id(&args.task_id);

    let task = crate::get_task(&file_path, &task_id).map_err(|e| JsonRpcError {
        code: INTERNAL_ERROR,
        message: format!("Failed to get task: {}", e),
        data: None,
    })?;

    let output = if args.verbose {
        crate::format_task_verbose(&task)
    } else {
        serde_json::to_string_pretty(&task).map_err(|e| JsonRpcError {
            code: INTERNAL_ERROR,
            message: format!("Failed to serialize task: {}", e),
            data: None,
        })?
    };

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}
