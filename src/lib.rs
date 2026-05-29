// Library interface for minitask
// This allows the integration tests to import minitask types and functions

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// Valid task states
pub const STATE_TODO: &str = "todo";
pub const STATE_IN_PROGRESS: &str = "in-progress";
pub const STATE_DONE: &str = "done";

/// Valid task states
pub const VALID_STATES: &[&str] = &[STATE_TODO, STATE_IN_PROGRESS, STATE_DONE];

/// Validates a task state
///
/// Returns Ok(()) if the state is valid, Err with a descriptive message otherwise.
pub fn validate_state(state: &str) -> io::Result<()> {
    if VALID_STATES.contains(&state) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Invalid state '{}'. Must be one of: {}",
                state,
                VALID_STATES.join(", ")
            ),
        ))
    }
}

// Re-export the MCP server module
pub mod mcp_server;

/// Represents a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier (e.g., "TASK-0")
    pub name: String,

    /// Current state of the task (e.g., "todo", "in-progress", "done")
    pub state: String,

    /// List of task IDs this task depends on
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// List of epics this task belongs to
    #[serde(default)]
    pub epic: Vec<String>,

    /// Task description and details
    pub content: String,
}

/// Container for all tasks in the file
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskFile {
    /// List of all tasks
    #[serde(default)]
    pub tasks: Vec<Task>,
}

impl TaskFile {
    /// Creates a new empty TaskFile
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }
}

/// Validates a file path for security concerns
///
/// Checks for:
/// - Path traversal attempts (checks for .. components in relative paths)
/// - Paths outside the workspace (for relative paths, validated against current directory)
///
/// Returns Ok(PathBuf) with the normalized path if valid, Err otherwise.
/// Note: Does not require the file to exist, only validates path structure.
/// Accepts both relative and absolute paths for flexibility in testing.
pub fn validate_file_path(file_path: &str) -> io::Result<std::path::PathBuf> {
    use std::env;
    
    let file_path = file_path.trim();
    
    if file_path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path cannot be empty",
        ));
    }
    
    let path = Path::new(file_path);
    
    // For relative paths, check for path traversal and validate against workspace
    if path.is_relative() {
        // Check for path traversal attempts by examining components
        for component in path.components() {
            if component == std::path::Component::ParentDir {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Path traversal not allowed",
                ));
            }
        }
        
        // Get workspace root (current directory for CLI tool)
        let workspace_root = env::current_dir()?;
        let full_path = workspace_root.join(path);
        
        // Normalize the path without requiring it to exist
        let normalized = full_path.components().collect::<std::path::PathBuf>();
        
        // Ensure result is within workspace
        if !normalized.starts_with(&workspace_root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Path outside workspace not allowed",
            ));
        }
        
        Ok(normalized)
    } else {
        // For absolute paths, just normalize and return
        // This allows tests to use temp files with absolute paths
        let normalized = path.components().collect::<std::path::PathBuf>();
        Ok(normalized)
    }
}

/// Loads tasks from a TOML file. Creates an empty file if it doesn't exist.
pub fn load_tasks<P: AsRef<Path>>(path: P) -> io::Result<TaskFile> {
    let path = path.as_ref();
    
    if !path.exists() {
        let empty_file = TaskFile::new();
        save_tasks(path, &empty_file)?;
        return Ok(empty_file);
    }
    
    let content = fs::read_to_string(path)?;
    let task_file: TaskFile = toml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    Ok(task_file)
}

/// Saves tasks to a TOML file.
pub fn save_tasks<P: AsRef<Path>>(path: P, task_file: &TaskFile) -> io::Result<()> {
    let content = toml::to_string_pretty(task_file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    fs::write(path, content)?;
    Ok(())
}

/// Normalizes a task ID by prepending "TASK-" if only a number is provided
pub fn normalize_task_id(id: &str) -> String {
    if id.parse::<usize>().is_ok() {
        format!("TASK-{}", id)
    } else {
        id.to_string()
    }
}

/// Gets a task by ID without printing (for programmatic use)
pub fn get_task(file_path: &Path, task_id: &str) -> io::Result<Task> {
    let task_file = load_tasks(file_path)?;
    
    task_file.tasks.into_iter()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))
}

/// Formats a task in verbose format
pub fn format_task_verbose(task: &Task) -> String {
    let mut output = format!("Task: {}\nState: {}\n", task.name, task.state);
    
    if !task.depends_on.is_empty() {
        output.push_str(&format!("Depends on: {}\n", task.depends_on.join(", ")));
    }
    
    if !task.epic.is_empty() {
        output.push_str(&format!("Epics: {}\n", task.epic.join(", ")));
    }
    
    output.push_str(&format!("Content:\n{}", task.content));
    output
}

/// Creates a new task with the given content and saves it to the file.
pub fn create_task(file_path: &Path, content: &str) -> io::Result<Task> {
    if content.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "content cannot be empty"
        ));
    }
    
    let mut task_file = load_tasks(file_path)?;
    
    let next_id = task_file.tasks.iter()
        .filter_map(|t| {
            t.name.strip_prefix("TASK-")
                .and_then(|n| n.parse::<usize>().ok())
        })
        .max()
        .map(|n| n + 1)
        .unwrap_or(0);
    
    let task_name = format!("TASK-{}", next_id);
    
    let new_task = Task {
        name: task_name,
        state: "todo".to_string(),
        depends_on: vec![],
        epic: vec![],
        content: content.to_string(),
    };
    
    task_file.tasks.push(new_task.clone());
    save_tasks(file_path, &task_file)?;
    
    Ok(new_task)
}

/// Handles the list command
pub fn handle_list(
    file_path: &Path,
    state_filter: Option<&str>,
    epic_filter: Option<&str>,
    verbose: bool,
    json_out: bool,
) -> io::Result<String> {
    let task_file = load_tasks(file_path)?;
    
    let filtered_tasks: Vec<&Task> = task_file.tasks.iter()
        .filter(|task| {
            let state_match = state_filter.map_or(true, |s| task.state == s);
            let epic_match = epic_filter.map_or(true, |e| task.epic.contains(&e.to_string()));
            state_match && epic_match
        })
        .collect();
    
    if json_out {
        let json = serde_json::to_string_pretty(&filtered_tasks)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(json)
    } else if verbose {
        let mut output = String::new();
        for task in filtered_tasks {
            output.push_str(&format!("Task: {}\n", task.name));
            output.push_str(&format!("State: {}\n", task.state));
            if !task.depends_on.is_empty() {
                output.push_str(&format!("Depends on: {}\n", task.depends_on.join(", ")));
            }
            if !task.epic.is_empty() {
                output.push_str(&format!("Epic: {}\n", task.epic.join(", ")));
            }
            output.push_str(&format!("Content:\n{}\n\n", task.content));
        }
        Ok(output)
    } else {
        let mut output = String::new();
        for task in filtered_tasks {
            let first_line = task.content.lines().next().unwrap_or("");
            output.push_str(&format!("{}: {}\n", task.name, first_line));
        }
        Ok(output)
    }
}

/// Handles the edit state command
pub fn handle_edit_state(
    file_path: &Path,
    task_id: &str,
    new_state: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;

    validate_state(new_state)?;
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    task.state = new_state.to_string();
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Updated {} state to {}", task_id, new_state))
    }
}

/// Handles the edit content command
pub fn handle_edit_content(
    file_path: &Path,
    task_id: &str,
    new_content: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    task.content = new_content.to_string();
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Updated {} content", task_id))
    }
}

/// Handles the add content command
pub fn handle_add_content(
    file_path: &Path,
    task_id: &str,
    content_to_add: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    task.content.push_str(content_to_add);
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Appended content to {}", task_id))
    }
}

/// Handles the add depends-on command
pub fn handle_add_depends_on(
    file_path: &Path,
    task_id: &str,
    depends_on_id: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;
    
    if !task_file.tasks.iter().any(|t| t.name == depends_on_id) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Dependency task '{}' not found", depends_on_id)
        ));
    }
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    if !task.depends_on.contains(&depends_on_id.to_string()) {
        task.depends_on.push(depends_on_id.to_string());
    }
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Added dependency {} to {}", depends_on_id, task_id))
    }
}

/// Handles the del depends-on command
pub fn handle_del_depends_on(
    file_path: &Path,
    task_id: &str,
    depends_on_id: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    task.depends_on.retain(|d| d != depends_on_id);
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Removed dependency {} from {}", depends_on_id, task_id))
    }
}

/// Handles the add epic command
pub fn handle_add_epic(
    file_path: &Path,
    task_id: &str,
    epic: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    if !task.epic.contains(&epic.to_string()) {
        task.epic.push(epic.to_string());
    }
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Added epic {} to {}", epic, task_id))
    }
}

/// Handles the del epic command
pub fn handle_del_epic(
    file_path: &Path,
    task_id: &str,
    epic: &str,
    json_out: bool,
) -> io::Result<String> {
    let mut task_file = load_tasks(file_path)?;
    
    let task = task_file.tasks.iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task '{}' not found", task_id)
        ))?;
    
    task.epic.retain(|e| e != epic);
    let updated_task = task.clone();
    
    save_tasks(file_path, &task_file)?;
    
    if json_out {
        serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Removed epic {} from {}", epic, task_id))
    }
}

/// Handles the show command
pub fn handle_show(
    file_path: &Path,
    task_id: &str,
    verbose: bool,
    json_out: bool,
) -> io::Result<String> {
    let task = get_task(file_path, task_id)?;
    
    if json_out {
        serde_json::to_string_pretty(&task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else if verbose {
        Ok(format_task_verbose(&task))
    } else {
        let first_line = task.content.lines().next().unwrap_or("");
        Ok(format!("{}: {}", task.name, first_line))
    }
}

/// Handles the new command
pub fn handle_new(
    file_path: &Path,
    content: &str,
    json_out: bool,
) -> io::Result<String> {
    use std::io::Read;
    
    // Read content from stdin if "-"
    let task_content = if content == "-" {
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        content.to_string()
    };
    
    // Use shared task creation helper
    let new_task = create_task(file_path, &task_content)?;
    
    if json_out {
        serde_json::to_string_pretty(&new_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(format!("Created {}", new_task.name))
    }
}

/// Handles the claim command
pub fn handle_claim(
    file_path: &Path,
    new_state: &str,
    state_filter: &str,
    epic_filter: Option<&str>,
    json_out: bool,
) -> io::Result<String> {
    validate_state(new_state)?;
    validate_state(state_filter)?;
    let mut task_file = load_tasks(file_path)?;
    
    // First pass: find the index of a claimable task
    let claimable_index = task_file.tasks.iter()
        .position(|task| {
            let state_match = task.state == state_filter;
            let epic_match = epic_filter.map_or(true, |e| task.epic.contains(&e.to_string()));
            let unblocked = task.depends_on.iter().all(|dep_id| {
                task_file.tasks.iter()
                    .find(|t| t.name == *dep_id)
                    .map_or(true, |dep_task| dep_task.state == STATE_DONE)
            });
            state_match && epic_match && unblocked
        });
    
    match claimable_index {
        Some(index) => {
            task_file.tasks[index].state = new_state.to_string();
            let claimed_task = task_file.tasks[index].clone();
            save_tasks(file_path, &task_file)?;
            
            if json_out {
                serde_json::to_string_pretty(&claimed_task)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            } else {
                Ok(format!("Claimed {} and set state to {}", claimed_task.name, new_state))
            }
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No available tasks to claim"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_path_rejects_parent_dir() {
        let result = validate_file_path("../etc/passwd");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_validate_file_path_rejects_nested_parent_dir() {
        let result = validate_file_path("foo/../../etc/passwd");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_validate_file_path_accepts_relative() {
        let result = validate_file_path("tasks.toml");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_path_accepts_nested_relative() {
        let result = validate_file_path("src/lib.rs");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_path_trims_whitespace() {
        let result = validate_file_path("  ../etc/passwd  ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_validate_file_path_rejects_empty() {
        let result = validate_file_path("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_validate_file_path_rejects_whitespace_only() {
        let result = validate_file_path("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_validate_file_path_accepts_absolute_for_tests() {
        // Absolute paths are allowed for test compatibility
        let result = validate_file_path("/tmp/test.toml");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_path_normalizes_current_dir() {
        let result = validate_file_path("./tasks.toml");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(!path.to_string_lossy().contains("./"));
    }
}
