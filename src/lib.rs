// Minimal library surface for testing specific modules without compiling the entire binary.
// We only include the init module to run its unit tests cleanly.

pub mod cmd_parser;

#[path = "command/init.rs"]
pub mod init;

#[path = "command/help.rs"]
pub mod help;

#[path = "command/build.rs"]
pub mod build;

#[path = "command/run.rs"]
pub mod run;

#[path = "command/clean.rs"]
pub mod clean;

#[path = "command/file_ops.rs"]
pub mod file_ops;

#[path = "command/list_file.rs"]
pub mod list_file;
