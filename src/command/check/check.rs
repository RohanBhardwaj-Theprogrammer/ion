use std::path::PathBuf;

use crate::cmd_parser::cmd::Type;
use crate::cmd_parser::parser::ParsedCommand;
use crate::config::Configs;

#[derive(Clone, Debug)]
pub struct CheckArgs {
    pub file_path: PathBuf,
    pub std: Option<usize>,
    pub help_flag: bool,
}

pub fn check_parser(
    args: &ParsedCommand,
    configs: &Configs,
    project_structure: &mut crate::state::structure::ProjectStructure,
) -> CheckArgs {
    if args.command_type != Type::Check {
        panic!("Invalid command type for check_parser");
    }
    let mut check_args = CheckArgs {
        file_path: PathBuf::new(),
        std: None,
        help_flag: false,
    };
    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        let arg = arg.to_lowercase().trim().to_string();

        match arg.as_str() {
            "-h" | "--help" | "-help" => {
                check_args.help_flag = true;
            }
            "-std" => {
                if let Some(val) = iter.next() {
                    check_args.std = val.parse::<usize>().ok();
                }
            }
            _ => {
                // Assume it's the file path
                check_args.file_path = super::super::run::run_parser::parse_file_name(
                    &arg,
                    configs,
                    project_structure,
                );
            }
        }
    }

    check_args
}

fn execute(
    check_args: &CheckArgs,
    configs: &Configs,
    project_structure: &mut crate::state::structure::ProjectStructure,
) -> Result<String, String> {
    if check_args.help_flag {
        println!("Check command help:");
        println!("Usage: c_cpp_build_system_n_pkg_manager check [options] <file_path>");
        return Ok("help displayed".to_string());
    }
    let file_name = check_args.file_path.clone();
    let file_name_str = file_name.to_str().unwrap_or("");
    let std = check_args.std.unwrap_or(11);
    use crate::compiler::build_settings::{BuildSettings, CompilationMode};
    use crate::compiler::compiler::Compiler;
    use crate::compiler::includes::IncludeFiles;
    use crate::deps::DependencyGraph;

    let mut settings = BuildSettings::default();
    settings.control.control.compilation_mode = CompilationMode::SyntaxCheck;

    let deps = DependencyGraph::new(file_name_str, project_structure);

    let include_files = IncludeFiles::new(&deps, configs, project_structure, file_name_str);

    let compiler = Compiler::new(
        file_name_str,
        include_files,
        settings,
        configs,
        project_structure,
    );
    compiler.compile()
}

pub fn check(
    args: ParsedCommand,
    configs: &Configs,
    project_structure: &mut crate::state::structure::ProjectStructure,
) -> Result<String, String> {
    #[cfg(any(test, debug_assertions))]
    {
        println!("[Check : check ] : ParsedCommand : ");
        dbg!(&args);
    }
    let check_args = check_parser(&args, configs, project_structure);

    #[cfg(any(test, debug_assertions))]
    {
        use crate::command::check;

        println!("[Check : check ] : CheckArgs : ");
        let check_args: check::CheckArgs = check_args.clone();
        dbg!(&check_args);
    }
    execute(&check_args, configs, project_structure)
}

//__________________________TEST____________________________

#[cfg(test_)]
mod tests {
    use super::*;
    use crate::cmd_parser::parser::ParsedCommand;
    use crate::config::Configs;

    #[test]
    fn check_parser_test() {
        let args = ParsedCommand {
            command_type: Type::Check,
            args: vec!["-std".to_string(), "17".to_string(), "main.cpp".to_string()],
        };

        let configs = Configs::test_config(None);
        let check_args = check_parser(
            &args,
            &configs,
            &mut crate::state::structure::ProjectStructure::new(&configs),
        );

        let expected = std::fs::canonicalize(format!("{}/main.cpp", configs.get_root_path()))
            .unwrap()
            .to_string_lossy();
        let actual = check_args.file_path.clone();
        // Normalize separators for comparison across platforms
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual),
            crate::utils::canonicalize_path_separators(&expected)
        );
        assert_eq!(check_args.std, Some(17));
        assert_eq!(check_args.help_flag, false);
    }
}
