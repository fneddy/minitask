//! Claim task tool handler

use serde::Deserialize;
use serde_json::Value;
use crate::mcp_server::protocol::{JsonRpcError, INVALID_PARAMS};

#[derive(Deserialize)]
struct ClaimTaskArgs {
    #[serde(default = "default_file_path")]
    file_path: String,
    new_state: String,
    #[serde(default = "default_state")]
    state: String,
    #[serde(default)]
    epic: Option<String>,
}

fn default_file_path() -> String {
    "tasks.toml".to_string()
}

fn default_state() -> String {
    "todo".to_string()
}

pub fn handle_claim_task_tool(arguments: Value) -> Result<Value, JsonRpcError> {
    let args: ClaimTaskArgs = serde_json::from_value(arguments).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid arguments for claim_task: {}", e),
        data: None,
    })?;

    let validated_path = crate::validate_file_path(&args.file_path).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    crate::validate_state(&args.new_state).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    crate::validate_state(&args.state).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: e.to_string(),
        data: None,
    })?;

    let output = crate::handle_claim(
        &validated_path,
        &args.new_state,
        &args.state,
        args.epic.as_deref(),
        true,
    )?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    }))
}
