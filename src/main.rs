use clap::{Parser, Subcommand};
use minitask::normalize_task_id;
use minitask::{handle_list, handle_edit_state, handle_edit_content, handle_add_content};
use minitask::{handle_add_depends_on, handle_del_depends_on, handle_add_epic, handle_del_epic, handle_claim};
use minitask::{handle_show, handle_new};
use minitask::STATE_TODO;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber;

/// Task management CLI tool
#[derive(Parser, Debug)]
#[command(name = "minitask")]
#[command(about = "A simple task management tool", long_about = None)]
struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    json_out: bool,

    /// Accept input as JSON from stdin
    #[arg(long, global = true)]
    json_in: bool,

    /// Path to the tasks file
    #[arg(long, global = true, default_value = "tasks.toml")]
    file: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum Commands {
    /// List tasks
    List {
        /// Filter by state
        #[arg(long)]
        state: Option<String>,

        /// Filter by epic
        #[arg(long)]
        epic: Option<String>,

        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Show a specific task
    Show {
        /// Task ID to show
        task_id: String,
        
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Create a new task
    New {
        /// Task content (use "-" to read from stdin)
        content: String,
    },
    /// Edit task properties
    Edit {
        #[command(subcommand)]
        edit_command: EditCommands,
    },
    /// Add to task properties
    Add {
        #[command(subcommand)]
        add_command: AddCommands,
    },
    /// Delete from task properties
    Del {
        #[command(subcommand)]
        del_command: DelCommands,
    },
    /// Claim the next available task
    Claim {
        /// New state for the claimed task
        new_state: String,

        /// Filter by source state
        #[arg(long, default_value = STATE_TODO)]
        state: String,

        /// Filter by epic
        #[arg(long)]
        epic: Option<String>,
    },
    /// Start MCP server
    Serve {
        /// Transport type (currently only stdio is supported)
        #[arg(long)]
        transport: Option<String>,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum EditCommands {
    /// Edit task state
    State {
        /// Task ID
        task_id: String,
        /// New state
        state: String,
    },
    /// Edit task content
    Content {
        /// Task ID
        task_id: String,
        /// New content
        content: String,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum AddCommands {
    /// Append to task content
    Content {
        /// Task ID
        task_id: String,
        /// Content to append
        content: String,
    },
    /// Add a dependency
    DependsOn {
        /// Task ID
        task_id: String,
        /// Dependency task ID
        depends_on: String,
    },
    /// Add task to an epic
    Epic {
        /// Task ID
        task_id: String,
        /// Epic name
        epic: String,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum DelCommands {
    /// Remove a dependency
    DependsOn {
        /// Task ID
        task_id: String,
        /// Dependency task ID to remove
        depends_on: String,
    },
    /// Remove task from an epic
    Epic {
        /// Task ID
        task_id: String,
        /// Epic name to remove
        epic: String,
    },
}



fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    
    let result = match cli.command {
        Some(Commands::List { state, epic, verbose }) => {
            handle_list(&cli.file, state.as_deref(), epic.as_deref(), verbose, cli.json_out)
                .map(|output| print!("{}", output))
        }
        Some(Commands::Show { task_id, verbose }) => {
            let task_id = normalize_task_id(&task_id);
            handle_show(&cli.file, &task_id, verbose, cli.json_out)
                .map(|output| print!("{}", output))
        }
        Some(Commands::New { content }) => {
            handle_new(&cli.file, &content, cli.json_out)
                .map(|output| print!("{}", output))
        }
        Some(Commands::Edit { edit_command }) => {
            match edit_command {
                EditCommands::State { task_id, state } => {
                    let task_id = normalize_task_id(&task_id);
                    handle_edit_state(&cli.file, &task_id, &state, cli.json_out)
                        .map(|output| print!("{}", output))
                }
                EditCommands::Content { task_id, content } => {
                    let task_id = normalize_task_id(&task_id);
                    handle_edit_content(&cli.file, &task_id, &content, cli.json_out)
                        .map(|output| print!("{}", output))
                }
            }
        }
        Some(Commands::Add { add_command }) => {
            match add_command {
                AddCommands::Content { task_id, content } => {
                    let task_id = normalize_task_id(&task_id);
                    handle_add_content(&cli.file, &task_id, &content, cli.json_out)
                        .map(|output| print!("{}", output))
                }
                AddCommands::DependsOn { task_id, depends_on } => {
                    let task_id = normalize_task_id(&task_id);
                    let depends_on = normalize_task_id(&depends_on);
                    handle_add_depends_on(&cli.file, &task_id, &depends_on, cli.json_out)
                        .map(|output| print!("{}", output))
                }
                AddCommands::Epic { task_id, epic } => {
                    let task_id = normalize_task_id(&task_id);
                    handle_add_epic(&cli.file, &task_id, &epic, cli.json_out)
                        .map(|output| print!("{}", output))
                }
            }
        }
        Some(Commands::Del { del_command }) => {
            match del_command {
                DelCommands::DependsOn { task_id, depends_on } => {
                    let task_id = normalize_task_id(&task_id);
                    let depends_on = normalize_task_id(&depends_on);
                    handle_del_depends_on(&cli.file, &task_id, &depends_on, cli.json_out)
                        .map(|output| print!("{}", output))
                }
                DelCommands::Epic { task_id, epic } => {
                    let task_id = normalize_task_id(&task_id);
                    handle_del_epic(&cli.file, &task_id, &epic, cli.json_out)
                        .map(|output| print!("{}", output))
                }
            }
        }
        Some(Commands::Claim { new_state, state, epic }) => {
            handle_claim(&cli.file, &new_state, &state, epic.as_deref(), cli.json_out)
                .map(|output| print!("{}", output))
        }
        Some(Commands::Serve { transport }) => {
            let transport = transport.as_deref().unwrap_or("stdio");
            handle_serve(transport).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
        _ => {
            error!("Command not yet implemented");
            std::process::exit(1);
        }
    };
    
    if let Err(e) = result {
        error!(error = %e, "Command execution failed");
        std::process::exit(1);
    }
}


/// Handles the serve command - starts MCP server
///
/// # Arguments
/// * `transport` - Transport type (currently only "stdio" is supported)
///
/// # Returns
/// * `Ok(())` if transport is valid
/// * `Err` if transport type is unsupported
///
/// # Current Limitations
/// This is a placeholder implementation. Actual MCP server logic not yet implemented.
///
/// # TODO for Production
/// - Implement JSON-RPC 2.0 message handling
/// - Add stdin/stdout I/O loop
/// - Implement MCP protocol handlers
/// - Add graceful shutdown on EOF/SIGTERM
fn handle_serve(transport: &str) -> Result<(), Box<dyn std::error::Error>> {
    const SUPPORTED_TRANSPORTS: &[&str] = &["stdio"];
    
    if !SUPPORTED_TRANSPORTS.contains(&transport) {
        return Err(format!(
            "Unsupported transport type: {}. Only 'stdio' is currently supported.",
            transport
        ).into());
    }
    
    info!(transport = %transport, "MCP server starting");
    
    let mut server = mcp_server::McpServer::new();
    server.run_stdio_sync()?;
    
    Ok(())
}

use minitask::mcp_server;
