use std::path::PathBuf;

use crate::cmd_parser::{cmd::Type, parser::ParsedCommand};
use crate::command::run::run::parse_file_name;
use crate::compiler::trailt::FromBuildConfigs;
use crate::compiler::Compiler;
use crate::config::Configs;

#[derive(Clone, Debug)]
pub struct BuildArgs {
    file_name: PathBuf,
    opt_level: Option<u8>,
    std: Option<u8>,
    i_extra: Vec<String>,
    //object_files : Vec<String>, // future use
    build_profile: Option<String>,
    help_flag: bool,
    interactive_flag: bool,
}

/// parse build command from the ParsedCommand struct
/// Grammer: `build [previousCommand] | <fileName | int | "" | .> [-opt <int>] [-std <int>] [-I <includePath> ...] [-objects <objectPath> ...] | <--profileName> | <--help | -help | -h> | <-interactive | -i>`
/// # Arguments
/// * `args` - ParsedCommand struct containing command type and arguments
/// * `configs` - Configs struct containing configuration settings
/// * `project_structure` - mutable reference to ProjectStructure struct
/// # Returns
/// * `BuildArgs` - struct containing parsed build arguments
/// # Panics
/// Panics if the command type is not `Build`.
pub fn build_parser(
    args: &ParsedCommand,
    configs: &Configs,
    project_structure: &mut crate::state::structure::ProjectStructure,
) -> BuildArgs {
    if args.command_type != Type::Build {
        panic!("Invalid command type for build_parser");
    }
    let mut build_args = BuildArgs {
        file_name: PathBuf::new(),
        opt_level: None,
        std: None,
        i_extra: Vec::new(),
        build_profile: None,
        help_flag: false,
        interactive_flag: false,
    };
    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "-help" => {
                build_args.help_flag = true;
            }
            "-i" | "--interactive" => {
                build_args.interactive_flag = true;
            }
            "-std" => {
                if let Some(val) = iter.next() {
                    build_args.std = Some(val.parse::<u8>().unwrap_or(11));
                }
            }
            "-opt" | "-o" => {
                if let Some(val) = iter.next() {
                    build_args.opt_level = Some(val.parse::<u8>().unwrap_or(0));
                }
            }
            "-I" | "--include" => {
                if let Some(val) = iter.next() {
                    build_args.i_extra.push(val.to_string());
                }
            }
            arg if arg.starts_with("-I") && arg.len() > 2 => {
                // e.g. -IC:/include
                build_args.i_extra.push(arg[2..].to_string());
            }
            arg if arg.starts_with("--") && arg.len() > 2 => {
                // e.g. --release, --profileName
                let profile = &arg[2..];

                build_args.build_profile = Some(profile.to_string());
            }
            first => {
                if !first.starts_with('-') && build_args.file_name.as_os_str().is_empty() {
                    build_args.file_name = parse_file_name(first, configs, project_structure);
                }
            }
        }
    }

    build_args
}

fn execute(
    build_args: &BuildArgs,
    configs: &Configs,
    project_structure: &mut crate::state::structure::ProjectStructure,
) -> Result<String, String> {
    if build_args.help_flag {
        crate::command::build::help::help();
        return Ok("Help displayed".to_string());
    }

    let file_name = &build_args.file_name;
    let entry_file_name = file_name
        .to_str()
        .expect("[Entry File]: Unable to converts the Types");
    let opt_level = build_args.opt_level;
    let std = build_args.std;
    let i_extra = &build_args.i_extra;

    #[cfg(any(test, debug_assertions))]
    let build_profile = &build_args.build_profile;

    let mut build_settings =
        crate::compiler::build_settings::build_profile_as_per(&build_args.build_profile);
    if build_args.build_profile.is_some() {
        if let Some(user_build_configs) = configs.get_build_profile(&build_args.build_profile) {
            build_settings.from_build_config(user_build_configs);
        }
    }

    //TODO: implement these CLI args properly
    if let Some(_) = opt_level {
        eprintln!("Note: Optimization level from CLI is currently not implemented.\n\tUse a build profile or set it in your configuration file.");
    }
    if let Some(_) = std {
        eprintln!("Note: Standard setting from CLI is currently not implemented.\n\tUse a build profile or set it in your configuration file.");
    }
    if !i_extra.is_empty() {
        eprintln!("Note: Extra include paths (-I/--include) from CLI are currently not implemented.\n\tAdd include paths to your build profile configuration instead.");
    }

    let deps = crate::deps::DependencyGraph::new(entry_file_name, project_structure);
    let include_files = crate::compiler::includes::IncludeFiles::new(
        &deps,
        &configs,
        project_structure,
        entry_file_name,
    );

    #[cfg(any(test, debug_assertions))]
    {
        println!("Build -> execute");
        println!("---------------- given build args ----------------");
        dbg!(build_args);
        dbg!(entry_file_name);
        dbg!(opt_level);
        dbg!(std);
        dbg!(i_extra);
        dbg!(build_profile);
        println!();
        println!("\t [Build Settings]");
        dbg!(&build_settings);
        println!("\t [Dependency Graph]");
        dbg!(&deps);
        println!("\t [Include Files]");
        dbg!(&include_files);
    }

    let compiler = Compiler::new(
        entry_file_name,
        include_files,
        build_settings,
        &configs,
        project_structure,
    );

    let compile_result = compiler.compile();

    match compile_result {
        Ok(output) => Ok(format!(
            "Build Process Completed. Binary at {} : {}",
            compiler.build_name, output
        )),
        Err(err) => {
            eprintln!("Build failed. Error: {}", err);
            Err(err)
        }
    }
}

pub fn build(
    args: ParsedCommand,
    configs: &Configs,
    project_structure: &mut crate::state::structure::ProjectStructure,
) -> Result<String, String> {
    #[cfg(any(test, debug_assertions))]
    {
        let args_clone = args.clone();
        dbg!(args_clone);
    }

    let build_args = build_parser(&args, configs, project_structure);
    if build_args.interactive_flag {
        return Err("Interactive mode not implemented yet".to_string());
    }

    #[cfg(any(test, debug_assertions))]
    {
        let args_clone = build_args.clone();
        dbg!(args_clone);
    }

    execute(&build_args, configs, project_structure)
}

//________________________________TESTS________________________________

#[cfg(test)]
mod test {
    use super::*;
    use crate::cmd_parser::parser::parse_args;
    use crate::config::Configs;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn parser_test() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        // Create a minimal file so parse_file_name can resolve root-relative paths.
        temp.child("main.cpp")
            .write_str("int main(){return 0;}")
            .unwrap();

        let build_args_cli = "program build -std 17 -opt 2 -I ./include --release main.cpp";
        let args_vec: Vec<String> = build_args_cli.split(' ').map(|s| s.to_string()).collect();
        let parsed_command = parse_args(args_vec);

        let configs = Configs::default(root_path.clone());
        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let build_args = build_parser(&parsed_command, &configs, &mut project_structure);

        let expected = std::fs::canonicalize(root_path.join("main.cpp")).unwrap();
        let actual = std::fs::canonicalize(&build_args.file_name).unwrap_or(build_args.file_name);
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual.to_string_lossy()),
            crate::utils::canonicalize_path_separators(&expected.to_string_lossy())
        );
        assert_eq!(build_args.std, Some(17));
        assert_eq!(build_args.opt_level, Some(2));
        assert_eq!(build_args.i_extra, vec!["./include".to_string()]);
        assert_eq!(build_args.build_profile, Some("release".to_string()));
    }

    #[test]
    fn parser_test_supports_dash_i_prefix_form() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("main.cpp")
            .write_str("int main(){return 0;}")
            .unwrap();

        let build_args_cli = "program build -I./inc --fast main.cpp";
        let args_vec: Vec<String> = build_args_cli.split(' ').map(|s| s.to_string()).collect();
        let parsed_command = parse_args(args_vec);

        let configs = Configs::default(root_path);
        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let build_args = build_parser(&parsed_command, &configs, &mut project_structure);

        assert_eq!(build_args.i_extra, vec!["./inc".to_string()]);
        assert_eq!(build_args.build_profile, Some("fast".to_string()));
    }

    #[test]
    fn parser_execute_test() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        let configs = Configs::default(root_path.clone());

        // Integration-style test: actually compiles a tiny program.
        // Skip when a C++ compiler isn't available.
        let compiler = configs.compiler_path().unwrap_or_else(|| "g++".to_string());
        if std::process::Command::new(&compiler)
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!(
                "Skipping build execute test: {} not found on PATH",
                compiler
            );
            return;
        }

        temp.child("main.cpp")
            .write_str(
                r#"#include <iostream>
int main(){ std::cout << "ok"; return 0; }"#,
            )
            .unwrap();

        let build_args_cli = "program build --release main.cpp";
        let args_vec: Vec<String> = build_args_cli.split(' ').map(|s| s.to_string()).collect();
        let parsed_command = parse_args(args_vec);

        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let build_args = build_parser(&parsed_command, &configs, &mut project_structure);
        let result = execute(&build_args, &configs, &mut project_structure);
        assert!(result.is_ok(), "Build should succeed: {:?}", result.err());

        let exe_path = root_path.join("build/main.exe");
        assert!(
            exe_path.exists(),
            "Executable should be created at {:?}",
            exe_path
        );

        let metadata =
            std::fs::metadata(&exe_path).expect("executable metadata should be readable");
        assert!(metadata.len() > 0, "Executable should be non-empty");
    }

    #[test]
    fn parser_____() {}
}
