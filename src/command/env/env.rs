use crate::cmd_parser::parser::ParsedCommand;
use crate::config::Configs;
use crate::state::ProjectStructure;
use crate::utils::trim_quotes;

#[derive(Debug, Clone)]
struct EnvArgs {
    pub help_flag: bool,
    pub cmd: String,
    pub env_vars: Vec<(String, String)>,
    pub env_file_path: Option<String>,
    pub clear_args: bool,
}

fn env_parser(args: &ParsedCommand, configs: &Configs) -> EnvArgs {
    if args.command_type != crate::cmd_parser::cmd::Type::Env {
        panic!("Invalid command type for env_parser");
    }

    let mut env_args = EnvArgs {
        help_flag: false,
        cmd: String::new(),
        env_vars: Vec::new(),
        env_file_path: None,
        clear_args: false,
    };

    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        let arg = arg.trim().to_lowercase();

        match arg.as_str() {
            "-h" | "--help" | "-help" => {
                env_args.help_flag = true;
            }
            "--clear" => {
                env_args.clear_args = true;
            }

            _ if arg.contains('=') => {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = trim_quotes(parts[1]);
                    env_args.env_vars.push((key, value));
                }
            }
            _ if arg.starts_with("--") => {
                // Assume it's the command to run
                env_args.cmd = arg[2..].to_string();
            }
            "-path" => {
                if let Some(val) = iter.next() {
                    env_args.env_file_path = Some(val.to_string());
                }
            }
            _ => {
                // Ignore unrecognized args for now
            }
        }
    }

    env_args
}

fn execute(
    env_args: &EnvArgs,
    _configs: &Configs,
    _project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    if env_args.help_flag {
        println!("Env command help:");
        println!(
            "Usage: c_cpp_build_system_n_pkg_manager env [options] [KEY=VALUE ...] [--command]"
        );
        return Ok("help displayed".to_string());
    }

    #[cfg(any(test, debug_assertions))]
    dbg!(&env_args);

    Err(format!(
        "Implement the env functionality for command: {} with vars: {:?}, file_path: {:?}, clear_args: {}",
        env_args.cmd,
        env_args.env_vars,
        env_args.env_file_path,
        env_args.clear_args
    ))
}

pub fn env(
    parsed_command: ParsedCommand,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    let env_args = env_parser(&parsed_command, configs);
    execute(&env_args, configs, project_structure)
}

//_________________________TEST__________________________________

#[cfg(test_)]
mod tests {
    use super::*;
    use crate::cmd_parser::cmd::Type;
    use crate::config::Configs;
    // use crate::command::build::stage::IncludeFiles;

    #[test]
    fn test_env_parser() {
        let args: ParsedCommand = ParsedCommand {
            command_type: Type::Env,
            args: vec![
                "VAR1=value1".to_string(),
                "VAR2=\"value with spaces\"".to_string(),
                "--my_command".to_string(),
                "-path".to_string(),
                "env_file.env".to_string(),
                "--clear".to_string(),
            ],
        };
        let configs = Configs::default("./some/path");
        let env_args = env_parser(&args, &configs);

        assert!(env_args.help_flag == false);
        assert!(env_args.cmd == "my_command");
        assert!(env_args.env_vars.len() == 2);
        assert!(env_args.env_vars[0] == ("var1".to_string(), "value1".to_string()));
        assert!(env_args.env_vars[1] == ("var2".to_string(), "value with spaces".to_string()));
        assert!(env_args.env_file_path == Some("env_file.env".to_string()));
        assert!(env_args.clear_args == true);
    }
}
