//! List tasks tool handler

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS, INTERNAL_ERROR};

#[derive(Deserialize)]
struct ListTasksArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    epic: Option<String>,
    #[serde(default)]
    verbose: bool,
}

fn default_file_path() -> String {
    "tasks.toml".to_string()
}

pub fn handle_list_tasks_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: ListTasksArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for list_tasks: {}", e),
        data: None,
    })?;

    let validated_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    if let Some(ref state) = args.state {
        crate::validate_state(state).map_err(|e| JsonRpcError {
            code: INVALID_PARAMS,
            message: e.to_string(),
            data: None,
        })?;
    }

    let output = crate::handle_list(
        &validated_path,
        args.state.as_deref(),
        args.epic.as_deref(),
        args.verbose,
        !args.verbose,
    )
    .map_err(|e| JsonRpcError {
        code: INTERNAL_ERROR,
        message: format!("Failed to list tasks: {}", e),
        data: None,
    })?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}
