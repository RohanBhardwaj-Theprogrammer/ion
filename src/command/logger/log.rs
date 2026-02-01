use crate::cmd_parser::cmd::Type;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Represents a single action or step performed during command execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Action {
    Created(String),      // File or directory created
    Modified(String),     // File or directory modified
    Removed(String),      // File or directory removed
    Compiled(String),     // File compiled
    Linked(String),       // Files linked
    Executed(String),     // Program executed
    Message(String),      // General message
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Created(s) => format!("Created: {}", s),
            Action::Modified(s) => format!("Modified: {}", s),
            Action::Removed(s) => format!("Removed: {}", s),
            Action::Compiled(s) => format!("Compiled: {}", s),
            Action::Linked(s) => format!("Linked: {}", s),
            Action::Executed(s) => format!("Executed: {}", s),
            Action::Message(s) => s.clone(),
        }
    }
}

/// Result of a command execution, including the main result and detailed actions
#[derive(Debug, Clone)]
pub struct CommandResult<T> {
    pub result: T,
    pub actions: Vec<Action>,
}

impl<T> CommandResult<T> {
    pub fn new(result: T) -> Self {
        CommandResult {
            result,
            actions: Vec::new(),
        }
    }

    pub fn with_actions(result: T, actions: Vec<Action>) -> Self {
        CommandResult { result, actions }
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }
}

/// Log entry for a command execution, including command type, arguments, result, and actions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandLog<T> {
    pub command_type: Type,
    pub command: T,
    pub result: Result<String, String>,
    pub actions: Vec<String>, // Serializable version of actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl<T> CommandLog<T> {
    /// Create a new CommandLog from command type, arguments, and result
    pub fn new(command_type: Type, command: T, result: Result<String, String>) -> Self {
        CommandLog {
            command_type,
            command,
            result,
            actions: Vec::new(),
            timestamp: Some(chrono::Local::now().to_rfc3339()),
        }
    }

    /// Create a CommandLog with actions
    pub fn with_actions(
        command_type: Type,
        command: T,
        result: Result<String, String>,
        actions: Vec<Action>,
    ) -> Self {
        CommandLog {
            command_type,
            command,
            result,
            actions: actions.iter().map(|a| a.to_string()).collect(),
            timestamp: Some(chrono::Local::now().to_rfc3339()),
        }
    }

    /// Add an action to the log
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action.to_string());
    }

    /// Check if the command was successful
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }
}

impl<T: serde::Serialize> CommandLog<T> {
    /// Save this log entry to a file
    pub fn save_to_file(&self, file_path: &Path) -> Result<(), String> {
        // Ensure the parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create log directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize log: {}", e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        writeln!(file, "{}", json)
            .map_err(|e| format!("Failed to write log: {}", e))?;

        Ok(())
    }
}

/// Logger for managing command logs
pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    /// Create a new Logger with the specified log directory
    pub fn new(log_dir: PathBuf) -> Self {
        Logger { log_dir }
    }

    /// Get the log file path for a specific command type
    pub fn get_log_file(&self, command_type: &Type) -> PathBuf {
        let filename = match command_type {
            Type::Init => "init.log",
            Type::Run => "run.log",
            Type::Build => "build.log",
            Type::Check => "check.log",
            Type::Clean => "clean.log",
            Type::Env => "env.log",
            Type::Version => "version.log",
            Type::Log => "log.log",
            Type::Unknown => "unknown.log",
        };
        self.log_dir.join(filename)
    }

    /// Save a command log entry
    pub fn log<T: serde::Serialize>(&self, log_entry: &CommandLog<T>) -> Result<(), String> {
        let log_file = self.get_log_file(&log_entry.command_type);
        log_entry.save_to_file(&log_file)
    }

    /// Clear logs for a specific command type
    pub fn clear_logs(&self, command_type: Option<Type>) -> Result<(), String> {
        if let Some(cmd_type) = command_type {
            let log_file = self.get_log_file(&cmd_type);
            if log_file.exists() {
                fs::remove_file(&log_file)
                    .map_err(|e| format!("Failed to clear log file: {}", e))?;
            }
        } else {
            // Clear all logs
            if self.log_dir.exists() {
                fs::remove_dir_all(&self.log_dir)
                    .map_err(|e| format!("Failed to clear log directory: {}", e))?;
                fs::create_dir_all(&self.log_dir)
                    .map_err(|e| format!("Failed to recreate log directory: {}", e))?;
            }
        }
        Ok(())
    }

    /// Read logs for a specific command type
    pub fn read_logs(&self, command_type: Option<Type>) -> Result<String, String> {
        if let Some(cmd_type) = command_type {
            let log_file = self.get_log_file(&cmd_type);
            if log_file.exists() {
                fs::read_to_string(&log_file)
                    .map_err(|e| format!("Failed to read log file: {}", e))
            } else {
                Ok(format!("No logs found for {:?}", cmd_type))
            }
        } else {
            // Read all logs
            let mut all_logs = String::new();
            if self.log_dir.exists() {
                for entry in fs::read_dir(&self.log_dir)
                    .map_err(|e| format!("Failed to read log directory: {}", e))?
                {
                    let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                    let path = entry.path();
                    if path.is_file() {
                        let content = fs::read_to_string(&path)
                            .map_err(|e| format!("Failed to read log file: {}", e))?;
                        all_logs.push_str(&format!("\n=== {} ===\n", path.display()));
                        all_logs.push_str(&content);
                    }
                }
            }
            if all_logs.is_empty() {
                Ok("No logs found".to_string())
            } else {
                Ok(all_logs)
            }
        }
    }
}