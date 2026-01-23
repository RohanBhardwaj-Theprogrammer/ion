use crate::config::Configs;
use crate::state::ProjectStructure;

pub struct WatcherArgs {
    pub cmd: String,
}

pub fn watcher_parser(args: &crate::cmd_parser::parser::ParsedCommand) -> WatcherArgs {
    if args.command_type != crate::cmd_parser::cmd::Type::Watcher {
        panic!("Invalid command type for watcher_parser");
    }

    let mut watcher_args = WatcherArgs { cmd: String::new() };

    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        let arg = arg.trim();

        // For watcher, we expect a single command argument
        if watcher_args.cmd.is_empty() {
            watcher_args.cmd = arg.to_string();
        }
        // Ignore any additional arguments for now
    }

    watcher_args
}

fn execute(
    watcher_args: &WatcherArgs,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    todo!(
        " Watcher! : will watch later , not today ! :  \t {}",
        watcher_args.cmd
    );
}

pub fn watcher(
    parsed_command: &crate::cmd_parser::parser::ParsedCommand,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    let watcher_args = watcher_parser(parsed_command);
    execute(&watcher_args, configs, project_structure)
}
