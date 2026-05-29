// MCP Integration Tests
// These tests verify the MCP server functionality end-to-end

use minitask::{Task, TaskFile};
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Helper function to create a test task file with sample data
fn setup_test_file(file_path: &str) -> TaskFile {
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["test-epic".to_string()],
                content: "First test task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "in-progress".to_string(),
                depends_on: vec!["TASK-0".to_string()],
                epic: vec![],
                content: "Second test task".to_string(),
            },
            Task {
                name: "TASK-2".to_string(),
                state: "done".to_string(),
                depends_on: vec![],
                epic: vec!["test-epic".to_string()],
                content: "Completed task".to_string(),
            },
        ],
    };
    let content = toml::to_string_pretty(&task_file)
        .expect("Failed to serialize task file");
    fs::write(file_path, content).expect("Failed to write test file");
    task_file
}

/// Mock MCP client for testing
/// Note: This bypasses the stdio transport layer and directly calls McpServer methods.
/// For full protocol compliance testing, consider adding tests that exercise the actual
/// stdio JSON-RPC transport.
struct MockMcpClient {
    server: minitask::mcp_server::McpServer,
}

impl MockMcpClient {
    fn new() -> Self {
        Self {
            server: minitask::mcp_server::McpServer::new(),
        }
    }

    fn initialize(&mut self) -> serde_json::Value {
        let request = minitask::mcp_server::JsonRpcRequest {
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
        let response = self.server.route_request(request);
        assert!(response.error.is_none(), "Initialize failed: {:?}", response.error);
        response.result.unwrap()
    }

    fn call_tool(&mut self, name: &str, arguments: serde_json::Value, id: i32) -> Result<serde_json::Value, String> {
        let request = minitask::mcp_server::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": name,
                "arguments": arguments
            })),
        };
        let response = self.server.route_request(request);
        if let Some(error) = response.error {
            Err(format!("Error {}: {}", error.code, error.message))
        } else {
            Ok(response.result.unwrap())
        }
    }

    fn list_tools(&mut self, id: i32) -> serde_json::Value {
        let request = minitask::mcp_server::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: "tools/list".to_string(),
            params: None,
        };
        let response = self.server.route_request(request);
        assert!(response.error.is_none());
        response.result.unwrap()
    }

    fn list_resources(&mut self, id: i32) -> serde_json::Value {
        let request = minitask::mcp_server::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: "resources/list".to_string(),
            params: None,
        };
        let response = self.server.route_request(request);
        assert!(response.error.is_none());
        response.result.unwrap()
    }

    fn read_resource(&mut self, uri: &str, id: i32) -> Result<serde_json::Value, String> {
        let request = minitask::mcp_server::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": uri
            })),
        };
        let response = self.server.route_request(request);
        if let Some(error) = response.error {
            Err(format!("Error {}: {}", error.code, error.message))
        } else {
            Ok(response.result.unwrap())
        }
    }
}

#[test]
fn test_mcp_initialize_handshake() {
    let mut client = MockMcpClient::new();
    let result = client.initialize();
    
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"].is_object());
    assert_eq!(result["serverInfo"]["name"], "minitask");
}

#[test]
fn test_mcp_double_initialize_error() {
    let mut client = MockMcpClient::new();
    client.initialize();
    
    // Second initialize should fail
    let request = minitask::mcp_server::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
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
    let response = client.server.route_request(request);
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32600);
}

#[test]
fn test_mcp_request_id_matching() {
    let mut client = MockMcpClient::new();
    client.initialize();
    
    // Test with numeric ID
    let request = minitask::mcp_server::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(42),
        method: "tools/list".to_string(),
        params: None,
    };
    let response = client.server.route_request(request);
    assert_eq!(response.id, json!(42));
    
    // Test with string ID
    let request = minitask::mcp_server::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("test-id-123"),
        method: "tools/list".to_string(),
        params: None,
    };
    let response = client.server.route_request(request);
    assert_eq!(response.id, json!("test-id-123"));
}

#[test]
fn test_mcp_tools_list() {
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.list_tools(2);
    let tools = result["tools"].as_array().unwrap();
    
    assert_eq!(tools.len(), 11);
    
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    
    assert_eq!(tool_names, vec![
        "list_tasks",
        "show_task",
        "create_task",
        "edit_task_state",
        "edit_task_content",
        "add_task_content",
        "add_task_dependency",
        "remove_task_dependency",
        "add_task_epic",
        "remove_task_epic",
        "claim_task",
    ]);
}

#[test]
fn test_mcp_list_tasks_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("list_tasks", json!({
        "file_path": file_path
    }), 2).unwrap();
    
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(text.contains("TASK-1"));
    assert!(text.contains("TASK-2"));
}

#[test]
fn test_mcp_list_tasks_with_filters() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    // Test state filter
    let result = client.call_tool("list_tasks", json!({
        "file_path": file_path,
        "state": "todo"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(!text.contains("TASK-1"));
    
    // Test epic filter
    let result = client.call_tool("list_tasks", json!({
        "file_path": file_path,
        "epic": "test-epic"
    }), 3).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(text.contains("TASK-2"));
    assert!(!text.contains("TASK-1"));
}

#[test]
fn test_mcp_list_tasks_verbose() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("list_tasks", json!({
        "file_path": file_path,
        "verbose": true
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(text.contains("First test task"));
    assert!(text.contains("todo"));
}

#[test]
fn test_mcp_list_tasks_combined_filters() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    // Test combined state and epic filter
    let result = client.call_tool("list_tasks", json!({
        "file_path": file_path,
        "state": "todo",
        "epic": "test-epic"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(!text.contains("TASK-1"));
    assert!(!text.contains("TASK-2"));
}

#[test]
fn test_mcp_show_task_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("show_task", json!({
        "file_path": file_path,
        "task_id": "TASK-0"
    }), 2).unwrap();
    
    // Verify response structure
    assert!(result["content"].is_array());
    assert_eq!(result["content"].as_array().unwrap().len(), 1);
    assert_eq!(result["content"][0]["type"], "text");
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.name, "TASK-0");
    assert_eq!(task.state, "todo");
    assert_eq!(task.content, "First test task");
    assert!(task.epic.contains(&"test-epic".to_string()));
    assert!(task.depends_on.is_empty());
}

#[test]
fn test_mcp_show_task_verbose() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("show_task", json!({
        "file_path": file_path,
        "task_id": "TASK-0",
        "verbose": true
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(text.contains("First test task"));
}

#[test]
fn test_mcp_create_task_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    let empty_file = TaskFile { tasks: vec![] };
    let content = toml::to_string_pretty(&empty_file).unwrap();
    fs::write(file_path, content).unwrap();
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("create_task", json!({
        "file_path": file_path,
        "content": "New task from MCP"
    }), 2).unwrap();
    
    // Verify response structure
    assert!(result["content"].is_array());
    assert_eq!(result["content"][0]["type"], "text");
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.name, "TASK-0");
    assert_eq!(task.state, "todo");
    assert_eq!(task.content, "New task from MCP");
    assert!(task.depends_on.is_empty());
    assert!(task.epic.is_empty());
    
    // Verify persistence
    let content = fs::read_to_string(file_path).unwrap();
    let loaded: TaskFile = toml::from_str(&content).unwrap();
    assert_eq!(loaded.tasks.len(), 1);
    assert_eq!(loaded.tasks[0].name, "TASK-0");
    assert_eq!(loaded.tasks[0].content, "New task from MCP");
}

#[test]
fn test_mcp_edit_task_state_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("edit_task_state", json!({
        "file_path": file_path,
        "task_id": "TASK-0",
        "state": "in-progress"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.state, "in-progress");
    
    // Verify persistence
    let content = fs::read_to_string(file_path).unwrap();
    let loaded: TaskFile = toml::from_str(&content).unwrap();
    assert_eq!(loaded.tasks[0].state, "in-progress");
}

#[test]
fn test_mcp_edit_task_content_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("edit_task_content", json!({
        "file_path": file_path,
        "task_id": "TASK-0",
        "content": "Updated content"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.content, "Updated content");
    
    // Verify persistence
    let content = fs::read_to_string(file_path).unwrap();
    let loaded: TaskFile = toml::from_str(&content).unwrap();
    assert_eq!(loaded.tasks[0].content, "Updated content");
}

#[test]
fn test_mcp_add_task_content_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("add_task_content", json!({
        "file_path": file_path,
        "task_id": "TASK-0",
        "content": "\nAppended line"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.content, "First test task\nAppended line");
}

#[test]
fn test_mcp_add_task_dependency_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("add_task_dependency", json!({
        "file_path": file_path,
        "task_id": "TASK-2",
        "depends_on": "TASK-1"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert!(task.depends_on.contains(&"TASK-1".to_string()));
}

#[test]
fn test_mcp_remove_task_dependency_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("remove_task_dependency", json!({
        "file_path": file_path,
        "task_id": "TASK-1",
        "depends_on": "TASK-0"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert!(!task.depends_on.contains(&"TASK-0".to_string()));
}

#[test]
fn test_mcp_add_task_epic_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("add_task_epic", json!({
        "file_path": file_path,
        "task_id": "TASK-1",
        "epic": "new-epic"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert!(task.epic.contains(&"new-epic".to_string()));
}

#[test]
fn test_mcp_remove_task_epic_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("remove_task_epic", json!({
        "file_path": file_path,
        "task_id": "TASK-0",
        "epic": "test-epic"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert!(!task.epic.contains(&"test-epic".to_string()));
}

#[test]
fn test_mcp_claim_task_tool() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("claim_task", json!({
        "file_path": file_path,
        "new_state": "in-progress"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.name, "TASK-0");
    assert_eq!(task.state, "in-progress");
}

#[test]
fn test_mcp_claim_task_with_filters() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    // Claim from specific state and epic
    let result = client.call_tool("claim_task", json!({
        "file_path": file_path,
        "new_state": "in-progress",
        "from_state": "todo",
        "epic": "test-epic"
    }), 2).unwrap();
    
    let text = result["content"][0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.name, "TASK-0");
    assert_eq!(task.state, "in-progress");
    assert!(task.epic.contains(&"test-epic".to_string()));
}

#[test]
fn test_mcp_resources_list() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    // Use custom tasks file path
    client.server = minitask::mcp_server::McpServer::with_tasks_file(file_path.to_string());
    client.initialize();
    
    let result = client.list_resources(2);
    let resources = result["resources"].as_array().unwrap();
    
    assert!(resources.len() >= 6); // 3 tasks * 2 resources each
    
    let uris: Vec<&str> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    
    assert!(uris.contains(&"task://TASK-0"));
    assert!(uris.contains(&"task://TASK-1"));
    assert!(uris.contains(&"task://TASK-2"));
}

#[test]
fn test_mcp_resources_read_full_task() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    // Use custom tasks file path
    client.server = minitask::mcp_server::McpServer::with_tasks_file(file_path.to_string());
    client.initialize();
    
    let result = client.read_resource("task://TASK-0", 2).unwrap();
    let contents = result["contents"].as_array().unwrap();
    
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["mimeType"], "application/json");
    
    let text = contents[0]["text"].as_str().unwrap();
    let task: Task = serde_json::from_str(text).unwrap();
    
    assert_eq!(task.name, "TASK-0");
    assert_eq!(task.state, "todo");
}

#[test]
fn test_mcp_resources_read_content_only() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    // Use custom tasks file path
    client.server = minitask::mcp_server::McpServer::with_tasks_file(file_path.to_string());
    client.initialize();
    
    let result = client.read_resource("task://TASK-0/content", 2).unwrap();
    let contents = result["contents"].as_array().unwrap();
    
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["mimeType"], "text/plain");
    assert_eq!(contents[0]["text"], "First test task");
}

#[test]
fn test_mcp_error_invalid_task_id() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("show_task", json!({
        "file_path": file_path,
        "task_id": "TASK-999"
    }), 2);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_mcp_error_invalid_state() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.call_tool("edit_task_state", json!({
        "file_path": file_path,
        "task_id": "TASK-0",
        "state": "invalid-state"
    }), 2);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid state"));
}

#[test]
fn test_mcp_error_invalid_resource_uri() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    setup_test_file(file_path);
    
    let mut client = MockMcpClient::new();
    client.initialize();
    
    let result = client.read_resource("invalid://uri", 2);
    
    assert!(result.is_err());
}

#[test]
fn test_mcp_protocol_compliance() {
    let mut client = MockMcpClient::new();
    
    // Test initialize returns correct structure
    let result = client.initialize();
    assert!(result.is_object());
    assert!(result.get("protocolVersion").is_some());
    assert!(result.get("capabilities").is_some());
    assert!(result.get("serverInfo").is_some());
    
    // Test tools/list returns correct structure
    let result = client.list_tools(2);
    assert!(result.is_object());
    assert!(result.get("tools").is_some());
    let tools = result["tools"].as_array().unwrap();
    for tool in tools {
        assert!(tool.get("name").is_some());
        assert!(tool.get("description").is_some());
        assert!(tool.get("inputSchema").is_some());
    }
    
    // Test resources/list returns correct structure
    let result = client.list_resources(3);
    assert!(result.is_object());
    assert!(result.get("resources").is_some());
}

// NEW TESTS: Addressing TASK-26-REVIEW-3 findings

#[test]
fn test_stdio_transport_full_flow() {
    // Build the binary first
    let output = Command::new("cargo")
        .args(&["build", "--bin", "minitask"])
        .output()
        .expect("Failed to build minitask binary");
    
    if !output.status.success() {
        panic!("Failed to build minitask: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Start the MCP server process
    let mut child = Command::new("target/debug/minitask")
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn minitask process");
    
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);
    
    // Send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    
    writeln!(stdin, "{}", serde_json::to_string(&init_request).unwrap()).unwrap();
    stdin.flush().unwrap();
    
    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).expect("Failed to read response");
    
    let response: serde_json::Value = serde_json::from_str(&response_line)
        .expect("Failed to parse response");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(response["result"]["serverInfo"]["name"], "minitask");
    
    // Send tools/list request
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    
    writeln!(stdin, "{}", serde_json::to_string(&tools_request).unwrap()).unwrap();
    stdin.flush().unwrap();
    
    // Read tools response
    response_line.clear();
    reader.read_line(&mut response_line).expect("Failed to read tools response");
    
    let tools_response: serde_json::Value = serde_json::from_str(&response_line)
        .expect("Failed to parse tools response");
    
    assert_eq!(tools_response["jsonrpc"], "2.0");
    assert_eq!(tools_response["id"], 2);
    assert!(tools_response["result"]["tools"].is_array());
    assert_eq!(tools_response["result"]["tools"].as_array().unwrap().len(), 11);
    
    // Clean up
    drop(stdin);
    thread::sleep(Duration::from_millis(100));
    let _ = child.kill();
}

#[test]
fn test_message_size_limit_enforcement() {
    let mut server = minitask::mcp_server::McpServer::new();
    
    // Create a message larger than 10MB
    let large_content = "x".repeat(11 * 1024 * 1024); // 11MB
    let large_message = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"data":"{}"}}}}"#,
        large_content
    );
    
    let result = server.process_message(&large_message);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("exceeds limit"));
}

#[test]
fn test_malformed_json_error() {
    let mut server = minitask::mcp_server::McpServer::new();
    
    // Test various malformed JSON inputs
    let malformed_inputs = vec![
        r#"{"invalid json"#,
        r#"not json at all"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"test""#, // Missing closing brace
        r#"{"jsonrpc":"2.0","id":1,}"#, // Trailing comma
        r#"{"jsonrpc":"2.0""id":1}"#, // Missing comma
    ];
    
    for input in malformed_inputs {
        let result = server.process_message(input);
        assert!(result.is_err(), "Expected error for input: {}", input);
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}

#[test]
fn test_initialized_notification_flow() {
    let mut server = minitask::mcp_server::McpServer::new();
    
    // Initialize the server
    let init_request = minitask::mcp_server::JsonRpcRequest {
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
    
    let response = server.route_request(init_request);
    assert!(response.error.is_none());
    
    // Send initialized notification
    let notification = minitask::mcp_server::JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "initialized".to_string(),
        params: None,
    };
    
    let result = server.handle_notification(notification);
    assert!(result.is_ok());
    
    // Try to send initialized notification before initialize (should fail)
    let mut server2 = minitask::mcp_server::McpServer::new();
    let notification2 = minitask::mcp_server::JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "initialized".to_string(),
        params: None,
    };
    
    let result2 = server2.handle_notification(notification2);
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err().code, -32600);
}

#[test]
fn test_unknown_notification_ignored() {
    let mut server = minitask::mcp_server::McpServer::new();
    server.initialized = true;
    
    // Send unknown notification (should be silently ignored per JSON-RPC spec)
    let notification = minitask::mcp_server::JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "unknown_notification".to_string(),
        params: None,
    };
    
    let result = server.handle_notification(notification);
    assert!(result.is_ok());
}