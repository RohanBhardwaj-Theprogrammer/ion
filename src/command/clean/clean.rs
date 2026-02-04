// Import or define ParsedCommand and Configs as needed
use std::path::{Path, PathBuf};

use crate::cmd_parser::parser::ParsedCommand;
use crate::config::Configs;
use crate::state::ProjectStructure;
use crate::utils::trim_quotes;

#[derive(Debug)]
struct CleanArgs {
    pub exceptions: Vec<String>,
    pub ext: Vec<String>,
    pub all: bool,
    pub force: bool,
    pub help: bool,
}

pub fn clean_parser(args: &ParsedCommand, configs: &Configs) -> CleanArgs {
    if args.command_type != crate::cmd_parser::cmd::Type::Clean {
        panic!("Invalid command type for clean_parser");
    }

    let mut clean_args = CleanArgs {
        exceptions: Vec::new(),
        ext: Vec::new(),
        all: false,
        force: false,
        help: false,
    };

    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        let arg_trimmed = arg.trim();
        let arg_lower = arg_trimmed.to_lowercase();

        match arg_lower.as_str() {
            "-h" | "--help" => {
                clean_args.help = true;
            }
            "--all" => {
                clean_args.all = true;
            }
            "--force" => {
                clean_args.force = true;
            }
            _ => {
                if arg_trimmed.starts_with("-e") {
                    let exception = arg_trimmed.trim_start_matches("-e").trim().to_string();
                    let exception = trim_quotes(&exception);
                    clean_args.exceptions.push(exception);
                } else if arg_trimmed.starts_with('.') {
                    clean_args.ext.push(arg_trimmed.to_string());
                }
                // Ignore unknown arguments for now
            }
        }
    }

    clean_args
}

fn execute(
    clean_args: &CleanArgs,
    _configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    if clean_args.help {
        clean_help();
        return Ok("Displayed help information.".to_string());
    }

    if clean_args.all && clean_args.force {
        for ext in &clean_args.ext {
            let ext_files = project_structure.get_files_with_extension(ext);
            let file_paths: Vec<PathBuf> = ext_files.iter().map(|f| f.get_path().clone()).collect();
            for file_path in file_paths {
                let file_path_str = file_path.to_string_lossy().to_string();
                if clean_args.exceptions.contains(&file_path_str) {
                    continue;
                }
                if let Err(e) = project_structure.remove_file(&file_path) {
                    eprintln!("Failed to remove file {}: {}", file_path.display(), e);
                }
            }
        }
    } else if clean_args.all {
        eprintln!("Use --force to delete files with specific extension. No files were deleted.");
    }
    // Always try to remove bin directory, but ignore error if it doesn't exist
    let _ = project_structure.remove_dir_all(Path::new("build"));
    Ok("Clean operation completed.".to_string())
}

fn clean_help() {
    println!("CBuild Clean Command Help:");
    println!("Usage: cbuild clean [options] [extensions]");
    println!();
    println!("Options:");
    println!("  -h, --help          Show this help message and exit");
    println!("  --all               Clean all build artifacts (not supported yet)");
    println!("  --force            Force deletion of files even if they are exceptions");
    println!("  -e<file_path>      Specify a file to exclude from cleaning");
    println!();
    println!("Extensions:");
    println!("  Specify file extensions to clean (e.g., .o, .tmp).");
    println!("  Only extensions allowed by the configuration will be considered.");
    println!();
    println!("Examples:");
    println!("  cbuild clean --all");
    println!("  cbuild clean .o .tmp -e\"src/main.o\" --force");
}

pub fn clean(
    args: &ParsedCommand,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    #[cfg(any(test, debug_assertions))]
    dbg!(&args);
    let clean_args = clean_parser(args, configs);
    #[cfg(any(test, debug_assertions))]
    dbg!(&clean_args);

    execute(&clean_args, configs, project_structure)
}

//_________________________________TEST___________________________

#[cfg(test)]
mod tests {

    use super::*;
    use crate::cmd_parser::parser::ParsedCommand;

    #[test]
    fn clean_parser_test() {
        let parsed_cmd = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Clean,
            args: vec![
                "--all".to_string(),
                "--force".to_string(),
                ".o".to_string(),
                ".tmp".to_string(),
            ],
        };

        let configs = Configs::test_config(None);

        let clean_args = clean_parser(&parsed_cmd, &configs);

        assert!(clean_args.all);
        assert!(clean_args.force);
        assert_eq!(clean_args.ext.len(), 2);
        assert!(clean_args.ext.contains(&".o".to_string()));
        assert!(clean_args.ext.contains(&".tmp".to_string()));
    }
}
