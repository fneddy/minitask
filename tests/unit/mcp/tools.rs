use minitask::{Task, TaskFile, save_tasks, load_tasks, handle_edit_state, handle_edit_content};
use minitask::mcp_server::{McpServer, JsonRpcRequest};
use serde_json::json;
use std::fs;

#[test]
fn test_mcp_list_tasks_tool() {
    let temp_file = "test_mcp_list.toml";
    
    // Create test tasks
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["test-epic".to_string()],
                content: "Test task 1".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "in-progress".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test task 2".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Initialize MCP server
    let mut server = McpServer::new();
    
    // Initialize the server first
    let init_request = JsonRpcRequest {
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
    
    let init_response = server.route_request(init_request);
    assert!(init_response.error.is_none());
    
    // Test tools/list
    let list_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/list".to_string(),
        params: None,
    };
    
    let list_response = server.route_request(list_request);
    assert!(list_response.error.is_none());
    assert!(list_response.result.is_some());
    
    let result = list_response.result.unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|tool| tool.get("name").unwrap().as_str().unwrap())
        .collect();
    assert_eq!(tool_names.len(), 11);
    assert_eq!(
        tool_names,
        vec![
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
        ]
    );
    
    // Test tools/call with list_tasks
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(3),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "state": "todo"
            }
        })),
    };
    
    let call_response = server.route_request(call_request);
    assert!(call_response.error.is_none());
    assert!(call_response.result.is_some());
    
    let result = call_response.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0].get("type").unwrap().as_str().unwrap(), "text");
    
    // Cleanup
    fs::remove_file(temp_file).unwrap();
}


#[test]
fn test_mcp_list_tasks_invalid_state() {
    let mut server = McpServer::new();
    
    // Initialize server
    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);
    
    // Test with invalid state
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "state": "invalid-state"
            }
        })),
    };
    
    let response = server.route_request(call_request);
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602); // INVALID_PARAMS
    assert!(error.message.contains("Invalid state"));
}

#[test]
fn test_mcp_list_tasks_epic_filter() {
    let temp_file = "test_mcp_epic_filter.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["epic1".to_string()],
                content: "Epic 1 task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["epic2".to_string()],
                content: "Epic 2 task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let mut server = McpServer::new();
    
    // Initialize
    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);
    
    // Test epic filter
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "file_path": temp_file,
                "epic": "epic1"
            }
        })),
    };
    
    let response = server.route_request(call_request);
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let text = content[0].get("text").unwrap().as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(!text.contains("TASK-1"));
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_mcp_list_tasks_verbose() {
    let temp_file = "test_mcp_verbose.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec!["epic1".to_string()],
                content: "Test content".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let mut server = McpServer::new();
    
    // Initialize
    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);
    
    // Test verbose output
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "file_path": temp_file,
                "verbose": true
            }
        })),
    };
    
    let response = server.route_request(call_request);
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let text = content[0].get("text").unwrap().as_str().unwrap();
    assert!(text.contains("State:"));
    assert!(text.contains("Depends on:"));
    assert!(text.contains("Epic:"));
    assert!(text.contains("Content:"));
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_mcp_list_tasks_combined_filters() {
    let temp_file = "test_mcp_combined.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["epic1".to_string()],
                content: "Match both".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "done".to_string(),
                depends_on: vec![],
                epic: vec!["epic1".to_string()],
                content: "Wrong state".to_string(),
            },
            Task {
                name: "TASK-2".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["epic2".to_string()],
                content: "Wrong epic".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let mut server = McpServer::new();
    
    // Initialize
    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);
    
    // Test combined filters
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "file_path": temp_file,
                "state": "todo",
                "epic": "epic1"
            }
        })),
    };
    
    let response = server.route_request(call_request);
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let text = content[0].get("text").unwrap().as_str().unwrap();
    assert!(text.contains("TASK-0"));
    assert!(!text.contains("TASK-1"));
    assert!(!text.contains("TASK-2"));
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_mcp_list_tasks_empty_results() {
    let temp_file = "test_mcp_empty.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "done".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Done task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let mut server = McpServer::new();
    
    // Initialize
    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);
    
    // Test empty results
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "file_path": temp_file,
                "state": "todo"
            }
        })),
    };
    
    let response = server.route_request(call_request);
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_mcp_list_tasks_custom_file_path() {
    let temp_file = "test_mcp_custom_path.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Custom file task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let mut server = McpServer::new();
    
    // Initialize
    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);
    
    // Test custom file path
    let call_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "file_path": temp_file
            }
        })),
    };
    
    let response = server.route_request(call_request);
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let text = content[0].get("text").unwrap().as_str().unwrap();
    assert!(text.contains("TASK-0"));
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_handle_edit_state_rejects_invalid_state() {
    let temp_file = "test_edit_state_invalid_state.toml";

    let task_file = TaskFile {
        tasks: vec![Task {
            name: "TASK-0".to_string(),
            state: "todo".to_string(),
            depends_on: vec![],
            epic: vec![],
            content: "Test task".to_string(),
        }],
    };
    save_tasks(temp_file, &task_file).unwrap();

    let result = handle_edit_state(std::path::Path::new(temp_file), "TASK-0", "blocked", true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_handle_edit_state_returns_updated_task_json() {
    let temp_file = "test_edit_state_json.toml";

    let task_file = TaskFile {
        tasks: vec![Task {
            name: "TASK-0".to_string(),
            state: "todo".to_string(),
            depends_on: vec!["TASK-1".to_string()],
            epic: vec!["epic1".to_string()],
            content: "Test task".to_string(),
        }],
    };
    save_tasks(temp_file, &task_file).unwrap();

    let output = handle_edit_state(std::path::Path::new(temp_file), "TASK-0", "done", true).unwrap();
    let updated: Task = serde_json::from_str(&output).unwrap();

    assert_eq!(updated.name, "TASK-0");
    assert_eq!(updated.state, "done");
    assert_eq!(updated.depends_on, vec!["TASK-1"]);
    assert_eq!(updated.epic, vec!["epic1"]);
    assert_eq!(updated.content, "Test task");

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_handle_edit_content_returns_updated_task_json() {
    let temp_file = "test_edit_content_json.toml";

    let task_file = TaskFile {
        tasks: vec![Task {
            name: "TASK-0".to_string(),
            state: "in-progress".to_string(),
            depends_on: vec!["TASK-1".to_string()],
            epic: vec!["epic1".to_string()],
            content: "Old content".to_string(),
        }],
    };
    save_tasks(temp_file, &task_file).unwrap();

    let output = handle_edit_content(std::path::Path::new(temp_file), "TASK-0", "New content", true).unwrap();
    let updated: Task = serde_json::from_str(&output).unwrap();

    assert_eq!(updated.name, "TASK-0");
    assert_eq!(updated.state, "in-progress");
    assert_eq!(updated.depends_on, vec!["TASK-1"]);
    assert_eq!(updated.epic, vec!["epic1"]);
    assert_eq!(updated.content, "New content");

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_mcp_edit_task_tools() {
    let temp_file = "test_mcp_edit_tools.toml";

    let task_file = TaskFile {
        tasks: vec![Task {
            name: "TASK-0".to_string(),
            state: "todo".to_string(),
            depends_on: vec![],
            epic: vec!["mcp".to_string()],
            content: "Original".to_string(),
        }],
    };
    save_tasks(temp_file, &task_file).unwrap();

    let mut server = McpServer::new();

    let init_request = JsonRpcRequest {
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
    let init_response = server.route_request(init_request);
    assert!(init_response.error.is_none());

    let state_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "edit_task_state",
            "arguments": {
                "file_path": temp_file,
                "task_id": "0",
                "state": "in-progress"
            }
        })),
    };
    let state_response = server.route_request(state_request);
    assert!(state_response.error.is_none());
    let state_text = state_response.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let updated_state_task: Task = serde_json::from_str(&state_text).unwrap();
    assert_eq!(updated_state_task.state, "in-progress");

    let content_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(3),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "edit_task_content",
            "arguments": {
                "file_path": temp_file,
                "task_id": "TASK-0",
                "content": "Updated from MCP"
            }
        })),
    };
    let content_response = server.route_request(content_request);
    assert!(content_response.error.is_none());
    let content_text = content_response.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let updated_content_task: Task = serde_json::from_str(&content_text).unwrap();
    assert_eq!(updated_content_task.content, "Updated from MCP");
    assert_eq!(updated_content_task.state, "in-progress");

    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].state, "in-progress");
    assert_eq!(reloaded.tasks[0].content, "Updated from MCP");

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_mcp_edit_task_state_invalid_state() {
    let mut server = McpServer::new();

    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "edit_task_state",
            "arguments": {
                "task_id": "TASK-0",
                "state": "blocked"
            }
        })),
    };

    let response = server.route_request(request);
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32602);
}

#[test]
fn test_mcp_edit_task_content_missing_task() {
    let temp_file = "test_mcp_edit_missing.toml";
    save_tasks(temp_file, &TaskFile { tasks: vec![] }).unwrap();

    let mut server = McpServer::new();

    let init_request = JsonRpcRequest {
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
    server.route_request(init_request);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "edit_task_content",
            "arguments": {
                "file_path": temp_file,
                "task_id": "TASK-999",
                "content": "Nope"
            }
        })),
    };

    let response = server.route_request(request);
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32602);

    fs::remove_file(temp_file).unwrap();
}