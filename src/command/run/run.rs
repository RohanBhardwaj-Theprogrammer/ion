use std::path::PathBuf;

use crate::cmd_parser::parser::ParsedCommand;
use crate::compiler;
use crate::compiler::trailt::FromBuildConfigs;
use crate::config::Configs;
use crate::state::ProjectStructure;
use crate::utils::is_numeric;
/*
run "" | <path> [args] [buildProfile] | <--help | -h | -help> | --config < { [-std <int>] , [. <lastExe | project.exe>] ,
 [-in <lastIn | default>] , [-out <lastOut | default>] } > | <-interactive | -i >

*/

#[derive(Clone, Debug)]
pub enum InputSource {
    IString,
    File(String),
    LastIn(String),
    Default,
}
#[derive(Clone, Debug)]
pub enum OutputDestination {
    OString,
    File(String),
    LastOut(String),
    Default,
}

#[derive(Clone, Debug)]
pub struct RunArgs {
    pub file_name: PathBuf,
    pub std: Option<u8>,
    pub build_profile: Option<String>,
    pub help_flag: bool,
    pub interactive_flag: bool,
    pub input: InputSource,
    pub output: OutputDestination,
    pub program_args: Vec<String>,
}

// REVIEW: need to review the code of the run parser and make it more robust and consistent with the check parser, also need to review the file parsing logic and make it more robust and consistent across the codebase
pub(crate) fn parse_file_name(
    file_arg: &str,
    configs: &crate::config::Configs,
    project_structure: &crate::state::ProjectStructure,
) -> PathBuf {
    let trimmed = file_arg.trim();
    match trimmed {
        "" | "." => {
            // No explicit file was provided.
            // Prefer a discovered entry file, then common main filenames, then fall back.
            if let Some(entry) = project_structure.get_entry_files().first() {
                return entry.get_path().clone();
            }
            if let Some(main_cpp) = project_structure.get_file_by_name("main.cpp") {
                return main_cpp.get_path().clone();
            }
            if let Some(main_c) = project_structure.get_file_by_name("main.c") {
                return main_c.get_path().clone();
            }

            let candidate_src_cpp = configs.get_root_path().join("src").join("main.cpp");
            if candidate_src_cpp.exists() {
                return std::fs::canonicalize(&candidate_src_cpp).unwrap_or(candidate_src_cpp);
            }

            let candidate_src_c = configs.get_root_path().join("src").join("main.c");
            if candidate_src_c.exists() {
                return std::fs::canonicalize(&candidate_src_c).unwrap_or(candidate_src_c);
            }

            // Legacy/user preference fallback.
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
                .unwrap_or_else(|| {
                    // Also try resolving as a root-relative path.
                    let candidate = configs.get_root_path().join(idx);
                    if candidate.exists() {
                        std::fs::canonicalize(&candidate).unwrap_or(candidate)
                    } else {
                        // Do not panic on user input; let the caller validate existence and return a friendly error.
                        candidate
                    }
                })
        }
        name_or_path => {
            // Try to resolve by name first, then by path
            project_structure
                .get_file_by_name(name_or_path)
                .map(|f| f.get_path().clone())
                .unwrap_or_else(|| {
                    let p = PathBuf::from(name_or_path);
                    if p.is_absolute() {
                        return p;
                    }

                    // If the provided path exists as-is, canonicalize it.
                    if p.exists() {
                        return std::fs::canonicalize(&p).unwrap_or(p);
                    }

                    // Otherwise, interpret it as root-relative.
                    let candidate = configs.get_root_path().join(&p);
                    if candidate.exists() {
                        return std::fs::canonicalize(&candidate).unwrap_or(candidate);
                    }

                    // Last resort: return as given.
                    p
                })
        }
    }
}
//ISSUE: the parsing logic having inconsistency and poor file_parsing logic
pub fn run_parser(
    args: &ParsedCommand,
    configs: &Configs,
    project_structure: &crate::state::ProjectStructure,
) -> RunArgs {
    if args.command_type != crate::cmd_parser::cmd::Type::Run {
        panic!("Invalid command type for run_parser");
    }
    //ALERT:: Need to parse the args properly and set the values accordingly , big refactor needed
    /*
          arguments Grammer : run
    "" |
    <fileName | int | "" | .> [(-in | <<) <filename>] [(-out | >>) <filename>] [-std <int>] [-- <exe-args>] |
    <fileName | int | "" | .> [inputFileName] [outputFileName] [-- <exe-args>] |
    [-help | --help | -h] |
    --config < { [-std <int>] , [. <lastExe | project.exe>] , [-in <lastIn | default>] , [-out <lastOut | default>] } >
     */
    let mut run_args = RunArgs {
        file_name: PathBuf::new(),
        std: None,
        build_profile: Some("fast".to_string()), // as default profile to speed up the run
        help_flag: false,
        interactive_flag: false,
        input: InputSource::Default,
        output: OutputDestination::Default,
        program_args: Vec::new(),
    };

    // Store a positional filename token (if any) and resolve it after parsing.
    // This ensures `run` with no args still attempts a sensible default.
    let mut file_token: Option<String> = None;

    let mut iter = args.args.iter().peekable();
    while let Some(arg) = iter.next() {
        let raw_arg = arg.trim();
        let lowered = raw_arg.to_lowercase();

        match lowered.as_str() {
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
                    match val.trim().to_lowercase().as_str() {
                        "lastin" => run_args.input = InputSource::LastIn("lastIn".to_string()),
                        "default" => run_args.input = InputSource::Default,
                        _ => run_args.input = InputSource::File(val.clone()),
                    }
                }
            }
            "-out" | ">>" => {
                if let Some(val) = iter.next() {
                    match val.trim().to_lowercase().as_str() {
                        "lastout" => {
                            run_args.output = OutputDestination::LastOut("lastOut".to_string())
                        }
                        "default" => run_args.output = OutputDestination::Default,
                        _ => run_args.output = OutputDestination::File(val.clone()),
                    }
                }
            }
            "--" => {
                // All remaining args are program args
                run_args.program_args.extend(iter.map(|s| s.to_string()));
                break;
            }
            profile_name if profile_name.starts_with("--") && profile_name.len() > 2 => {
                let profile_name = &profile_name[2..].to_string();
                #[cfg(any(test, debug_assertions))]
                {
                    println!("Detected build profile from arg: {}", profile_name);
                }
                run_args.build_profile = Some(profile_name.to_string());
            }
            _ => {
                if !raw_arg.starts_with('-') {
                    if file_token.is_none() {
                        file_token = Some(raw_arg.to_string());
                    } else {
                        // Optional second positional token: treat as build profile.
                        run_args.build_profile = Some(raw_arg.to_string());
                    }
                }
            }
        }
    }

    let file_arg = file_token.as_deref().unwrap_or("");
    run_args.file_name = parse_file_name(file_arg, configs, project_structure);

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
    let file_name_str = file_name.to_string_lossy();
    let file_name_str = file_name_str.as_ref();

    let p = std::path::Path::new(file_name_str);
    if file_name_str.trim().is_empty() || file_name_str == "." || !p.is_file() {
        return Err(format!(
            "No valid source file selected for run. Got: '{}'",
            file_name_str
        ));
    }
    let std = run_args.std;
    let build_profile = &run_args.build_profile;
    let input = &run_args.input;
    let output = &run_args.output;
    let program_args = &run_args.program_args;

    let mut build_settings = crate::compiler::build_settings::build_profile_as_per(&build_profile);
    if build_profile.is_some() {
        if let Some(user_build_configs) = configs.get_build_profile(&build_profile) {
            build_settings.from_build_config(user_build_configs);
        }
    }

    if let Some(_) = std {
        eprintln!("Note: Standard setting from CLI is currently not implemented.\n\tUse a build profile or set it in your configuration file.");
    }

    // Build a dependency graph for the requested main file using the provided project structure
    let deps = crate::deps::DependencyGraph::new(file_name_str, project_structure);
    let include_files =
        compiler::includes::IncludeFiles::new(&deps, configs, project_structure, file_name_str);

    #[cfg(any(test, debug_assertions))]
    {
        println!("Run -> execute");
        println!("---------------- given run args ----------------");
        dbg!(&run_args);
        dbg!(file_name_str);
        dbg!(std);
        dbg!(input);
        dbg!(output);
        dbg!(program_args);

        println!(" [build settings] : ");
        dbg!(&build_settings);

        println!(" [dependency graph] : ");
        dbg!(&deps);
        println!(" [include files] : ");
        dbg!(&include_files);
    }

    // Construct the Compiler via its constructor which will use `configs` to resolve the compiler path
    let compiler = compiler::Compiler::new(
        file_name_str,
        include_files,
        build_settings,
        configs,
        project_structure,
    );

    let result = compiler.compile();
    #[cfg(any(test, debug_assertions))]
    {
        println!("\t [Run : execute] : Compilation result : ");
        dbg!(&result);
    }

    match result {
        Ok(binary_path) => {
            use std::process::Command;
            let mut cmd = Command::new(binary_path);
            #[cfg(any(test, debug_assertions))]
            {
                dbg!(&cmd);
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

            let child_process = match cmd.spawn() {
                Ok(p) => p,
                Err(e) => {
                    return Err(format!("Failed to spawn program: {}", e));
                }
            };
            #[cfg(any(test, debug_assertions))]
            {
                println!("\t [Run : execute] : Spawned child process : ");
                dbg!(&child_process);
            }

            let output = child_process.wait_with_output();
            // Actually run the command
            match output {
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
    #[cfg(any(test, debug_assertions))]
    {
        println!("\t [Run] : ");
        let args_clone = args.clone();
        dbg!(args_clone);
    }
    let run_args = run_parser(&args, configs, project_structure);

    #[cfg(any(test, debug_assertions))]
    {
        let run_args_clone = run_args.clone();
        dbg!(run_args_clone);
    }

    if run_args.interactive_flag {
        return Err("Interactive mode not implemented yet".to_string());
    }
    execute(run_args, configs, project_structure)
}

//______________________________________TEST ____________________________________

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn run_parser_test() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("main.cpp")
            .write_str("int main(){return 0;}")
            .unwrap();

        let input_path = temp.child("input.txt");
        input_path.write_str("hello\n").unwrap();
        let output_path = temp.child("output.txt");

        let args = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Run,
            args: vec![
                "main.cpp".to_string(),
                "-std".to_string(),
                "17".to_string(),
                "--release".to_string(),
                "-in".to_string(),
                input_path.path().to_string_lossy().to_string(),
                "-out".to_string(),
                output_path.path().to_string_lossy().to_string(),
                "--".to_string(),
                "arg1".to_string(),
                "arg2".to_string(),
            ],
        };

        let configs = Configs::default(root_path.clone());
        let project_structure = ProjectStructure::new(&configs);
        let run_args = run_parser(&args, &configs, &project_structure);

        let expected = std::fs::canonicalize(root_path.join("main.cpp")).unwrap();
        let actual = std::fs::canonicalize(&run_args.file_name).unwrap_or(run_args.file_name);
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual.to_string_lossy()),
            crate::utils::canonicalize_path_separators(&expected.to_string_lossy())
        );
        assert_eq!(run_args.std, Some(17));
        assert_eq!(run_args.build_profile, Some("release".to_string()));
        match run_args.input {
            InputSource::File(ref fname) => {
                assert_eq!(
                    crate::utils::canonicalize_path_separators(fname),
                    crate::utils::canonicalize_path_separators(
                        &input_path.path().to_string_lossy()
                    )
                )
            }
            _ => panic!("Input source should be File"),
        }
        match run_args.output {
            OutputDestination::File(ref fname) => {
                assert_eq!(
                    crate::utils::canonicalize_path_separators(fname),
                    crate::utils::canonicalize_path_separators(
                        &output_path.path().to_string_lossy()
                    )
                )
            }
            _ => panic!("Output destination should be File"),
        }
        assert_eq!(
            run_args.program_args,
            vec!["arg1".to_string(), "arg2".to_string()]
        );
    }

    #[test]
    fn run_parser_no_args_resolves_default_file() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        // Prefer the canonical project layout.
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/main.cpp")
            .write_str("int main(){return 0;}")
            .unwrap();

        let args = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Run,
            args: vec![],
        };

        let configs = Configs::default(root_path.clone());
        let project_structure = ProjectStructure::new(&configs);
        let run_args = run_parser(&args, &configs, &project_structure);

        let expected = std::fs::canonicalize(root_path.join("src").join("main.cpp")).unwrap();
        let actual = std::fs::canonicalize(&run_args.file_name).unwrap_or(run_args.file_name);
        assert_eq!(
            crate::utils::canonicalize_path_separators(&actual.to_string_lossy()),
            crate::utils::canonicalize_path_separators(&expected.to_string_lossy())
        );
    }

    #[test]
    fn execute_run_test() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        // Tiny program that echoes first line of stdin.
        temp.child("main.cpp")
            .write_str(
                r#"#include <iostream>
#include <string>
int main(){ std::string s; std::getline(std::cin, s); std::cout << s; return 0; }"#,
            )
            .unwrap();

        let input_path = temp.child("input.txt");
        input_path.write_str("hello\n").unwrap();
        let output_path = temp.child("output.txt");

        let configs = Configs::default(root_path.clone());

        // Skip if the configured compiler isn't available.
        let compiler = configs.compiler_path().unwrap_or_else(|| "g++".to_string());
        if std::process::Command::new(&compiler)
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("Skipping execute_run_test: {} not found on PATH", compiler);
            return;
        }

        let args = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Run,
            args: vec![
                "main.cpp".to_string(),
                "-in".to_string(),
                input_path.path().to_string_lossy().to_string(),
                "-out".to_string(),
                output_path.path().to_string_lossy().to_string(),
            ],
        };

        let project_structure = ProjectStructure::new(&configs);
        let run_args = run_parser(&args, &configs, &project_structure);
        let mut project_structure = ProjectStructure::new(&configs);

        let result = execute(run_args, &configs, &mut project_structure);
        assert!(result.is_ok(), "run execute should succeed: {:?}", result);

        let contents = std::fs::read_to_string(output_path.path())
            .expect("output file should be created and readable");
        assert_eq!(contents, "hello");
    }
}
