//! MCP (Model Context Protocol) server implementation
//!
//! Provides stdio-based JSON-RPC transport for task management operations.
//! Implements the MCP protocol specification for initialize/initialized handshake
//! and request routing.

pub mod protocol;
pub mod resources;
pub mod tools;

// Re-export protocol types for external use
pub use protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, JsonRpcError};

use protocol::*;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info, warn};

/// MCP server state
pub struct McpServer {
    /// Whether the server has been initialized
    pub initialized: bool,
    /// Whether the initialized notification was received
    pub initialized_notification_received: bool,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Default tasks file path
    pub tasks_file: String,
}

impl McpServer {
    /// Creates a new MCP server instance with default tasks file
    pub fn new() -> Self {
        Self::with_tasks_file("tasks.toml".to_string())
    }

    /// Creates a new MCP server instance with custom tasks file path
    pub fn with_tasks_file(tasks_file: String) -> Self {
        Self {
            initialized: false,
            initialized_notification_received: false,
            capabilities: ServerCapabilities {
                tools: Some(serde_json::json!({
                    "listChanged": false
                })),
                resources: Some(serde_json::json!({
                    "subscribe": false,
                    "listChanged": false
                })),
            },
            tasks_file,
        }
    }

    /// Handles an initialize request
    fn handle_initialize(&mut self, params: InitializeRequest) -> Result<InitializeResult, JsonRpcError> {
        if self.initialized {
            return Err(JsonRpcError {
                code: INVALID_REQUEST,
                message: "Server already initialized".to_string(),
                data: None,
            });
        }

        // Validate protocol version compatibility
        if params.protocol_version != PROTOCOL_VERSION {
            warn!(
                client_version = %params.protocol_version,
                server_version = %PROTOCOL_VERSION,
                "Client protocol version differs from server version"
            );
        }

        self.initialized = true;

        Ok(InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: self.capabilities.clone(),
            server_info: Implementation {
                name: "minitask".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })
    }

    /// Handles a notification
    pub fn handle_notification(&mut self, notification: JsonRpcNotification) -> Result<(), JsonRpcError> {
        match notification.method.as_str() {
            "initialized" => {
                if !self.initialized {
                    return Err(JsonRpcError {
                        code: INVALID_REQUEST,
                        message: "Cannot send initialized notification before initialize request".to_string(),
                        data: None,
                    });
                }
                self.initialized_notification_received = true;
                info!("Client sent initialized notification");
                Ok(())
            }
            _ => {
                // Unknown notifications are silently ignored per JSON-RPC spec
                debug!(method = %notification.method, "Received unknown notification");
                Ok(())
            }
        }
    }

    /// Routes a request to the appropriate handler
    pub fn route_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        if !self.initialized && request.method != "initialize" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_REQUEST,
                    message: "Server not initialized. Call initialize first.".to_string(),
                    data: None,
                }),
            };
        }

        let result = match request.method.as_str() {
            "initialize" => match serde_json::from_value::<InitializeRequest>(
                request.params.clone().unwrap_or(Value::Null),
            ) {
                Ok(params) => match self.handle_initialize(params) {
                    Ok(result) => serde_json::to_value(result).map_err(|e| JsonRpcError {
                        code: INTERNAL_ERROR,
                        message: format!("Internal error: failed to serialize result: {}", e),
                        data: None,
                    }),
                    Err(e) => Err(e),
                },
                Err(e) => Err(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: format!("Invalid params: {}", e),
                    data: None,
                }),
            },
            "tools/list" => {
                let tools = tools::get_tools_list();
                Ok(serde_json::json!({ "tools": tools }))
            }
            "tools/call" => tools::route_tool_call(request.params.unwrap_or(Value::Null)),
            "resources/list" => resources::handle_resources_list(&self.tasks_file),
            "resources/read" => {
                #[derive(serde::Deserialize)]
                struct ResourceReadParams {
                    uri: String,
                }
                match serde_json::from_value::<ResourceReadParams>(request.params.unwrap_or(Value::Null)) {
                    Ok(params) => resources::handle_resources_read(&self.tasks_file, &params.uri),
                    Err(e) => Err(JsonRpcError {
                        code: INVALID_PARAMS,
                        message: format!("Invalid resource read params: {}", e),
                        data: None,
                    }),
                }
            }
            _ => Err(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    /// Processes a single JSON-RPC message
    pub fn process_message(&mut self, line: &str) -> Result<Option<String>, io::Error> {
        if line.len() > MAX_MESSAGE_SIZE {
            error!(size = line.len(), limit = MAX_MESSAGE_SIZE, "Rejected message exceeding size limit");
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message size {} exceeds limit of {} bytes", line.len(), MAX_MESSAGE_SIZE),
            ));
        }

        let message: JsonRpcMessage = serde_json::from_str(line).map_err(|e| {
            error!(error = %e, "JSON parse error");
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON-RPC: {}", e))
        })?;

        match message {
            JsonRpcMessage::Request(request) => {
                debug!(method = %request.method, id = ?request.id, "Processing request");
                let response = self.route_request(request);
                let response_json = serde_json::to_string(&response).map_err(|e| {
                    error!(error = %e, "Failed to serialize response");
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                Ok(Some(response_json))
            }
            JsonRpcMessage::Notification(notification) => {
                debug!(method = %notification.method, "Processing notification");
                if let Err(e) = self.handle_notification(notification) {
                    warn!(error = %e.message, "Notification handling error");
                }
                Ok(None)
            }
            JsonRpcMessage::Response(_) => {
                error!("Unexpected response message received");
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unexpected response message",
                ))
            }
        }
    }

    /// Runs the MCP server with stdio transport (synchronous)
    pub fn run_stdio_sync(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = line?;

            if line.trim().is_empty() {
                continue;
            }

            match self.process_message(&line) {
                Ok(Some(response)) => {
                    writeln!(stdout, "{}", response)?;
                    stdout.flush()?;
                }
                Ok(None) => {}
                Err(e) => {
                    error!(error = %e, "Error processing message");
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: PARSE_ERROR,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    let error_json = serde_json::to_string(&error_response)?;
                    writeln!(stdout, "{}", error_json)?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_initialize_request() {
        let mut server = McpServer::new();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let response = server.route_request(request);

        assert!(response.error.is_none());
        assert!(response.result.is_some());
        assert!(server.initialized);
    }

    #[test]
    fn test_double_initialize_error() {
        let mut server = McpServer::new();
        server.initialized = true;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let response = server.route_request(request);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[test]
    fn test_unknown_method() {
        let mut server = McpServer::new();
        server.initialized = true;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "unknown_method".to_string(),
            params: None,
        };

        let response = server.route_request(request);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_process_message_valid() {
        let mut server = McpServer::new();

        let message = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

        let result = server.process_message(message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_process_message_invalid_json() {
        let mut server = McpServer::new();

        let message = r#"{"invalid json"#;

        let result = server.process_message(message);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_response_format() {
        let mut server = McpServer::new();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "invalid".to_string(),
            params: None,
        };

        let response = server.route_request(request);

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(1));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }
}
