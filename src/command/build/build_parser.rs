use std::path::PathBuf;

use crate::cmd_parser::{cmd::Type, parser::ParsedCommand};
use crate::command::run::run_parser::parse_file_name;
use crate::compiler::Compiler;
use crate::config::Configs;
use crate::utils::is_numeric; // REVIEW: is this needed anywhere ?

#[derive(Clone, Debug)]
pub struct BuildArgs {
    file_name: PathBuf,
    opt_level: u8,
    std: u8,
    i_extra: Vec<String>,
    //object_files : Vec<String>, // future use
    build_profile: String,
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
        opt_level: 0,
        std: 11,
        i_extra: Vec::new(),
        build_profile: "debug".to_string(),
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
                    build_args.std = val.parse::<u8>().unwrap_or(11);
                }
            }
            "-opt" | "-o" => {
                if let Some(val) = iter.next() {
                    build_args.opt_level = val.parse::<u8>().unwrap_or(0);
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
                build_args.build_profile = arg[2..].to_string();
            }
            first => {
                if !first.starts_with('-') && !first.starts_with("--") {
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
    let entry_file_name = file_name.to_str().unwrap_or("");
    let opt_level = build_args.opt_level;
    let std = build_args.std;
    let i_extra = &build_args.i_extra;
    #[cfg(any(test, debug_assertions))]
    let build_profile = &build_args.build_profile;
    
    let build_settings = crate::compiler::build_settings::BuildSettings::release();
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
        Ok(output) => {
            println!("Build succeeded. Output: {}", output);
            Ok(format!(
                "Build Process Completed. Binary at {}",
                compiler.build_name
            ))
        }
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

#[cfg(test_)]
mod test {
    use super::*;
    use crate::cmd_parser::parser::parse_args;
    use crate::config::Configs;

    #[test]
    fn parser_test() {
        let build_args_cli = "program build -std 17 -opt 2 -I ./include --release main.cpp";
        let args_vec: Vec<String> = build_args_cli.split(' ').map(|s| s.to_string()).collect();
        let parsed_command = parse_args(args_vec);
        let root_path = "./tests/test_project";
        let configs = Configs::default(root_path);
        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let build_args = build_parser(&parsed_command, &configs, &mut project_structure);
        let expected = std::fs::canonicalize(format!("{}/main.cpp", root_path))
            .unwrap()
            .to_string_lossy();
        let actual = build_args.file_name.clone();
        // Normalize separators for comparison across platforms
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual.to_string_lossy()),
            crate::utils::canonicalize_path_separators(&expected)
        );
        assert_eq!(build_args.std, 17);
        assert_eq!(build_args.opt_level, 2);
        assert_eq!(build_args.i_extra, vec!["./include".to_string()]);
        assert_eq!(build_args.build_profile, "release".to_string());
    }

    #[test]
    fn parser_execute_test() {
        let build_args_cli = "program build -std 17 -opt 2 -I ./include --release main.cpp";
        let args_vec: Vec<String> = build_args_cli.split(' ').map(|s| s.to_string()).collect();
        let parsed_command = parse_args(args_vec);
        let root_path = "./tests/test_project";
        let configs = Configs::default(root_path);

        // Skip if a C++ compiler isn't available on PATH.
        if std::process::Command::new("g++")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("Skipping build execute test: g++ not found on PATH");
            return;
        }

        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let build_args = build_parser(&parsed_command, &configs, &mut project_structure);
        let mut project_structure = crate::state::structure::ProjectStructure::new(&configs);
        let _ = execute(&build_args, &configs, &mut project_structure);
        let exe_path = format!("{}/build/main.exe", root_path);
        assert!(
            std::path::Path::new(&exe_path).exists(),
            "Executable should be created at {}",
            exe_path
        );
    }

    #[test]
    fn run_exe_test() {
        // Skip if the executable doesn't exist (e.g., compiler not available).
        let exe_path = "./tests/test_project/build/main.exe";
        if !std::path::Path::new(exe_path).exists() {
            eprintln!("Skipping run_exe_test: {} does not exist", exe_path);
            return;
        }
        std::process::Command::new(exe_path)
            .stdin(std::process::Stdio::inherit())
            .output()
            .expect("Failed to execute test executable");
    }
}
