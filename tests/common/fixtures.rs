use minitask::{Task, TaskFile, save_tasks};
use std::fs;
use tempfile::NamedTempFile;

/// Creates a temporary task file with the given tasks and returns the path.
/// The file will be automatically cleaned up when the returned TempTaskFile is dropped.
pub struct TempTaskFile {
    pub path: String,
}

impl TempTaskFile {
    pub fn new(tasks: Vec<Task>) -> Self {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        
        // Keep the file by persisting it
        let (_, path_buf) = temp_file.keep().unwrap();
        let path = path_buf.to_str().unwrap().to_string();
        
        let task_file = TaskFile { tasks };
        save_tasks(&path, &task_file).unwrap();
        
        TempTaskFile { path }
    }
    
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for TempTaskFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Creates a test task with the given parameters
pub fn create_test_task(name: &str, state: &str, content: &str) -> Task {
    Task {
        name: name.to_string(),
        state: state.to_string(),
        depends_on: vec![],
        epic: vec![],
        content: content.to_string(),
    }
}

/// Creates a test task with dependencies
pub fn create_test_task_with_deps(
    name: &str,
    state: &str,
    content: &str,
    depends_on: Vec<String>,
) -> Task {
    Task {
        name: name.to_string(),
        state: state.to_string(),
        depends_on,
        epic: vec![],
        content: content.to_string(),
    }
}

/// Creates a test task with epic
pub fn create_test_task_with_epic(
    name: &str,
    state: &str,
    content: &str,
    epic: Vec<String>,
) -> Task {
    Task {
        name: name.to_string(),
        state: state.to_string(),
        depends_on: vec![],
        epic,
        content: content.to_string(),
    }
}

/// Creates a fully configured test task
pub fn create_test_task_full(
    name: &str,
    state: &str,
    content: &str,
    depends_on: Vec<String>,
    epic: Vec<String>,
) -> Task {
    Task {
        name: name.to_string(),
        state: state.to_string(),
        depends_on,
        epic,
        content: content.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_task_file_creation() {
        let tasks = vec![create_test_task("TASK-0", "todo", "Test content")];
        let temp_file = TempTaskFile::new(tasks);
        
        // File should exist
        assert!(std::path::Path::new(temp_file.path()).exists());
        
        // Should be able to load tasks
        let loaded = minitask::load_tasks(temp_file.path()).unwrap();
        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(loaded.tasks[0].name, "TASK-0");
    }

    #[test]
    fn test_temp_file_cleanup() {
        let path: String;
        {
            let tasks = vec![create_test_task("TASK-0", "todo", "Test")];
            let temp_file = TempTaskFile::new(tasks);
            path = temp_file.path().to_string();
            
            // File exists while in scope
            assert!(std::path::Path::new(&path).exists());
        }
        
        // File should be cleaned up after drop
        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn test_create_test_task() {
        let task = create_test_task("TASK-1", "in-progress", "Test content");
        assert_eq!(task.name, "TASK-1");
        assert_eq!(task.state, "in-progress");
        assert_eq!(task.content, "Test content");
        assert!(task.depends_on.is_empty());
        assert!(task.epic.is_empty());
    }

    #[test]
    fn test_create_test_task_with_deps() {
        let task = create_test_task_with_deps(
            "TASK-2",
            "todo",
            "Content",
            vec!["TASK-1".to_string()],
        );
        assert_eq!(task.depends_on, vec!["TASK-1"]);
    }

    #[test]
    fn test_create_test_task_with_epic() {
        let task = create_test_task_with_epic(
            "TASK-3",
            "done",
            "Content",
            vec!["epic1".to_string()],
        );
        assert_eq!(task.epic, vec!["epic1"]);
    }
}
