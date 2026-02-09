use std::path::PathBuf;

use crate::cmd_parser::cmd::Type;
use crate::cmd_parser::parser::ParsedCommand;
use crate::compiler::build_settings;
use crate::compiler::trailt::FromBuildConfigs;
use crate::config::Configs;

#[derive(Clone, Debug)]
pub struct CheckArgs {
    pub file_path: PathBuf,
    pub std: Option<usize>,
    pub build_profile: Option<String>,
    pub verboase_flag: bool,
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
        build_profile: None,
        verboase_flag: false,
        help_flag: false,
    };
    let mut file_path_arg = String::new();
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
            "-v" | "--verbose" => {
                check_args.verboase_flag = true;
            }
            profile_name if profile_name.starts_with("--") && profile_name.len() > 2 => {
                let profile = profile_name[2..].to_string();
                check_args.build_profile = Some(profile);
            }
            file_name => {
                if !file_name.starts_with("-") && check_args.file_path.as_os_str().is_empty() {
                    file_path_arg = file_name.to_string();
                }
            }
        }
    }

    check_args.file_path =
        super::super::run::run::parse_file_name(&file_path_arg, configs, project_structure);

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
    let build_profile = &check_args.build_profile;

    use crate::compiler::build_settings::CompilationMode;
    use crate::compiler::compiler::Compiler;
    use crate::compiler::includes::IncludeFiles;
    use crate::deps::DependencyGraph;

    let mut settings = build_settings::build_profile_as_per(build_profile);
    if build_profile.is_some() {
        if let Some(profile_settings) = configs.get_build_profile(build_profile) {
            settings.from_build_config(profile_settings);
        }
    }
    settings.control.control.compilation_mode = CompilationMode::SyntaxCheck;

    if check_args.verboase_flag {
        settings.control.logging.verbose = true;
    }
    if check_args.std.is_some() {
        eprintln!("not supported std change for check command, use default or profile std");
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_parser::parser::ParsedCommand;
    use crate::config::Configs;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn check_parser_test() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("main.cpp")
            .write_str("int main(){return 0;}")
            .unwrap();

        let args = ParsedCommand {
            command_type: Type::Check,
            args: vec!["-std".to_string(), "17".to_string(), "main.cpp".to_string()],
        };

        let configs = Configs::default(root_path.clone());
        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let check_args = check_parser(&args, &configs, &mut project_structure);

        let expected = std::fs::canonicalize(root_path.join("main.cpp")).unwrap();
        let actual = std::fs::canonicalize(&check_args.file_path).unwrap_or(check_args.file_path);
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual.to_string_lossy()),
            crate::utils::canonicalize_path_separators(&expected.to_string_lossy())
        );
        assert_eq!(check_args.std, Some(17));
        assert!(!check_args.help_flag);
    }

    #[test]
    fn check_execute_syntax_check_smoke_test() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("main.cpp")
            .write_str("int main(){return 0;}")
            .unwrap();

        let configs = Configs::default(root_path.clone());

        // Skip if the configured compiler isn't available.
        let compiler = configs.compiler_path().unwrap_or_else(|| "g++".to_string());
        if std::process::Command::new(&compiler)
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!(
                "Skipping check execute test: {} not found on PATH",
                compiler
            );
            return;
        }

        let args = ParsedCommand {
            command_type: Type::Check,
            args: vec!["--verbose".to_string(), "main.cpp".to_string()],
        };

        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let check_args = check_parser(&args, &configs, &mut project_structure);

        let result = super::execute(&check_args, &configs, &mut project_structure);
        assert!(result.is_ok(), "check should succeed: {:?}", result);
    }
}
