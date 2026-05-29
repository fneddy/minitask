//! Create task tool handler

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS, INTERNAL_ERROR};

#[derive(Deserialize)]
struct CreateTaskArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    content: String,
}

fn default_file_path() -> String {
    "tasks.toml".to_string()
}

pub fn handle_create_task_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: CreateTaskArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for create_task: {}", e),
        data: None,
    })?;

    let file_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    let task = crate::create_task(&file_path, &args.content)?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&task).map_err(|e| JsonRpcError {
                code: INTERNAL_ERROR,
                message: format!("Failed to serialize task: {}", e),
                data: None,
            })?
        }]
    }))
}
