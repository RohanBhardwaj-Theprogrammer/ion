use crate::cmd_parser::cmd::Type;
use crate::cmd_parser::parser::ParsedCommand;
use crate::config::Configs;
use crate::constants;
use crate::state::structure::ProjectStructure;

// Single API entry for executing parsed commands. Routes to subcommands and
// normalizes return types to `Result<String, String>` where possible.
pub fn execute(
    parsed_command: ParsedCommand,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    match parsed_command.command_type {
        Type::Build => crate::command::build::build(parsed_command, configs, project_structure),
        Type::Run => crate::command::run::run(parsed_command, configs, project_structure),
        Type::Init => {
            // `init` returns `Result<(), String>` — map success to a string.
            crate::command::init::init(parsed_command, configs).map(|_| "Init executed".to_string())
        }
        Type::Check => crate::command::check::check(parsed_command, configs, project_structure),
        Type::Clean => {
            crate::command::clean::clean::clean(&parsed_command, configs, project_structure)
        }
        Type::Env => crate::command::env::env(parsed_command, configs, project_structure),
        Type::Version => {
            // Print version info and return a simple message
            let msg = format!(
                "{} version: {}",
                constants::PROGRAM_NAME,
                constants::PROGRAM_VERSION
            );
            println!("{}", msg);
            Ok(msg)
        }
        Type::Help => {
            // Basic help: delegate to init/gen help if available or print a short message
            println!("cbuild: help - available commands: build, run, check, clean, init, env");
            Ok("help displayed".to_string())
        }
        // Not implemented command types return a clear error
        Type::Log | Type::Config | Type::Previous | Type::Watcher => {
            Err("Command not implemented yet".to_string())
        }
        _ => Err("Command not implemented yet".to_string()),
    }
}
