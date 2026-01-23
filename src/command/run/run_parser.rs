use std::path::PathBuf;

use crate::cmd_parser::parser::ParsedCommand;
use crate::compiler;
use crate::config::Configs;
use crate::state::ProjectStructure;
use crate::utils::is_numeric;
/*
run "" | <path> [args] [buildProfile] | <--help | -h | -help> | --config < { [-std <int>] , [. <lastExe | project.exe>] ,
 [-in <lastIn | default>] , [-out <lastOut | default>] } > | <-interactive | -i >

*/

enum InputSource {
    IString,
    File(String),
    LastIn(String),
    Default,
}

enum OutputDestination {
    OString,
    File(String),
    LastOut(String),
    Default,
}

struct RunArgs {
    pub file_name: PathBuf,
    pub std: Option<u8>,
    pub build_profile: Option<String>,
    pub help_flag: bool,
    pub interactive_flag: bool,
    pub input: InputSource,
    pub output: OutputDestination,
    pub program_args: Vec<String>,
}

pub(crate) fn parse_file_name(
    file_arg: &str,
    configs: &crate::config::Configs,
    project_structure: &crate::state::ProjectStructure,
) -> PathBuf {
    let trimmed = file_arg.trim();
    match trimmed {
        "" | "." => {
            // Use configs for default file selection (legacy or user preference)
            configs.get_file("")
        }
        idx if is_numeric(idx) => {
            if let Ok(idx_num) = idx.parse::<usize>() {
                if let Some(file) = project_structure.get_by_index(idx_num) {
                    return file.get_path().clone();
                }
            }
            // Fallback to name-based resolution
            project_structure
                .get_file_by_name(idx)
                .map(|f| f.get_path().clone())
                .unwrap_or_else(|| panic!("File not found by index or name: {}", idx))
        }
        name_or_path => {
            // Try to resolve by name first, then by path
            project_structure
                .get_file_by_name(name_or_path)
                .map(|f| f.get_path().clone())
                .unwrap_or_else(|| PathBuf::from(name_or_path))
        }
    }
}

pub fn run_parser(
    args: &ParsedCommand,
    configs: &Configs,
    project_structure: &crate::state::ProjectStructure,
) -> RunArgs {
    if args.command_type != crate::cmd_parser::cmd::Type::Run {
        panic!("Invalid command type for run_parser");
    }

    let mut run_args = RunArgs {
        file_name: PathBuf::new(),
        std: None,
        build_profile: None,
        help_flag: false,
        interactive_flag: false,
        input: InputSource::Default,
        output: OutputDestination::Default,
        program_args: Vec::new(),
    };

    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        let arg = arg.trim().to_lowercase();

        match arg.as_str() {
            "-h" | "--help" | "-help" => {
                run_args.help_flag = true;
            }
            "-i" | "--interactive" => {
                run_args.interactive_flag = true;
            }
            "-std" => {
                if let Some(val) = iter.next() {
                    run_args.std = Some(val.parse::<u8>().unwrap_or(11));
                }
            }
            "-in" | "<<" => {
                if let Some(val) = iter.next() {
                    match val.as_str() {
                        "lastin" => run_args.input = InputSource::LastIn("lastIn".to_string()),
                        "default" => run_args.input = InputSource::Default,
                        _ => run_args.input = InputSource::File(val.clone()),
                    }
                }
            }
            "-out" | ">>" => {
                if let Some(val) = iter.next() {
                    match val.as_str() {
                        "lastout" => {
                            run_args.output = OutputDestination::LastOut("lastOut".to_string())
                        }
                        "default" => run_args.output = OutputDestination::Default,
                        _ => run_args.output = OutputDestination::File(val.clone()),
                    }
                }
            }
            profile if !profile.starts_with('-') => {
                // Assume it's the file name or build profile
                if run_args.file_name.as_os_str().is_empty() {
                    run_args.file_name = parse_file_name(profile, configs, project_structure);
                } else {
                    run_args.build_profile = Some(profile.to_string());
                }
            }
            "--" => {
                // All remaining args are program args
                run_args.program_args.extend(iter.map(|s| s.to_string()));
                break;
            }
            _ => {
                run_args.file_name = parse_file_name(&arg, configs, project_structure);
            }
        }
    }

    run_args
}

// #[allow(unused)]
fn execute(
    run_args: RunArgs,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    if run_args.help_flag {
        crate::command::run::help::help();
        return Ok("Help displayed".to_string());
    }

    let file_name = &run_args.file_name;
    let file_name_str = file_name.to_str().unwrap_or("");
    let std = run_args.std;
    let build_profile = &run_args.build_profile;
    let input = &run_args.input;
    let output = &run_args.output;
    let program_args = &run_args.program_args;

    let build_settings = crate::compiler::build_settings::BuildSettings::fast();

    // Build a dependency graph for the requested main file using the provided project structure
    let deps = crate::deps::DependencyGraph::new(file_name_str, project_structure);
    let include_files =
        compiler::includes::IncludeFiles::new(&deps, configs, project_structure, file_name_str);

    // Construct the Compiler via its constructor which will use `configs` to resolve the compiler path
    let compiler = compiler::Compiler::new(
        file_name_str,
        include_files,
        build_settings,
        configs,
        project_structure,
    );

    let result = compiler.compile();

    match result {
        Ok(binary_path) => {
            use std::process::Command;
            let mut cmd = Command::new(binary_path);

            if !configs.get_root_path().as_os_str().is_empty() {
                cmd.current_dir(configs.get_root_path());
            }
            // Handle input redirection
            match input {
                InputSource::File(fname) => {
                    use std::fs::File;
                    use std::process::Stdio;
                    match File::open(fname) {
                        Ok(file) => {
                            cmd.stdin(Stdio::from(file));
                        }
                        Err(e) => {
                            eprintln!("Failed to open input file '{}': {}", fname, e);
                        }
                    }
                }
                _ => {
                    println!("Using default input source");
                }
            }

            match output {
                OutputDestination::File(fname) => {
                    use std::fs::File;
                    use std::process::Stdio;
                    match File::create(fname) {
                        Ok(file) => {
                            cmd.stdout(Stdio::from(file));
                        }
                        Err(e) => {
                            eprintln!("Failed to create output file '{}': {}", fname, e);
                        }
                    }
                }
                _ => {
                    println!("Using default output destination");
                }
            }

            match program_args.len() {
                0 => {}
                _ => {
                    cmd.args(program_args);
                }
            }

            // Actually run the command
            match cmd.output() {
                Ok(output) => {
                    println!("Program exited with status: {}", output.status);
                    Ok(format!("Program exited with status: {}", output.status))
                }
                Err(e) => {
                    eprintln!("Failed to execute program: {}", e);
                    Err(format!("Failed to execute program: {}", e))
                }
            }
        }

        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            Err(e)
        }
    }
}

pub fn run(
    args: ParsedCommand,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    #[allow(unused_mut)]
    let mut run_args = run_parser(&args, configs, project_structure);

    if run_args.interactive_flag {
        return Err("Interactive mode not implemented yet".to_string());
    }
    execute(run_args, configs, project_structure)
}

//______________________________________TEST ____________________________________

#[cfg(test_)]
mod tests {
    use super::*;
    #[test]
    fn run_parser_test() {
        let args = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Run,
            args: vec![
                "main.cpp".to_string(),
                "-std".to_string(),
                "17".to_string(),
                "-in".to_string(),
                "input.txt".to_string(),
                "-out".to_string(),
                "output.txt".to_string(),
                "--".to_string(),
                "arg1".to_string(),
                "arg2".to_string(),
            ],
        };
        let configs = Configs::test_config(None);
        let mut project_structure = ProjectStructure::new(&configs);
        let run_args = run_parser(&args, &configs, &project_structure);
        let expected = std::fs::canonicalize(format!("{}/main.cpp", configs.get_root_path()))
            .unwrap()
            .to_string_lossy();
        let actual = run_args.file_name.to_string_lossy();
        // Normalize separators for comparison across platforms
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual),
            crate::utils::canonicalize_path_separators(&expected)
        );
        assert_eq!(run_args.std, Some(17));
        match run_args.input {
            InputSource::File(ref fname) => assert_eq!(fname, "input.txt"),
            _ => panic!("Input source should be File"),
        }
        match run_args.output {
            OutputDestination::File(ref fname) => assert_eq!(fname, "output.txt"),
            _ => panic!("Output destination should be File"),
        }
        assert_eq!(
            run_args.program_args,
            vec!["arg1".to_string(), "arg2".to_string()]
        );
    }

    #[test]
    fn execute_run_test() {
        // Skip if a C++ compiler isn't available on PATH.
        if std::process::Command::new("g++")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("Skipping execute_run_test: g++ not found on PATH");
            return;
        }

        let args = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Run,
            args: vec![
                "main.cpp".to_string(),
                "-std".to_string(),
                "17".to_string(),
                "-in".to_string(),
                format!(
                    "tests{}test_project{}input.txt",
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR
                ),
                "-out".to_string(),
                format!(
                    "tests{}test_project{}output.txt",
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR
                ),
                "--".to_string(),
                "arg1".to_string(),
                "arg2".to_string(),
            ],
        };
        let configs = Configs::test_config(Some("tests/test_project/config.toml"));
        let mut project_structure = ProjectStructure::test_new(&configs);
        let run_args = run_parser(&args, &configs, &project_structure);

        let mut project_structure = ProjectStructure::new(&configs);
        let _ = execute(run_args, &configs, &mut project_structure);

        match std::fs::File::open("tests/test_project/output.txt") {
            Ok(mut file) => {
                use std::io::Read;
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
            }
            Err(e) => {
                panic!("Failed to open output file: {}", e);
            }
        }
    }
}
