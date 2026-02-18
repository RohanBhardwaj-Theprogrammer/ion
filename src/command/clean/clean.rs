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

fn clean_parser(args: &ParsedCommand, _configs: &Configs) -> CleanArgs {
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
                if arg_lower == "-e" {
                    if let Some(val) = iter.next() {
                        let exception = trim_quotes(val.trim());
                        if !exception.is_empty() {
                            clean_args.exceptions.push(exception);
                        }
                    }
                } else if arg_lower.starts_with("-e") {
                    let exception = arg_trimmed[2..].trim().to_string();
                    let exception = trim_quotes(&exception);
                    if !exception.is_empty() {
                        clean_args.exceptions.push(exception);
                    }
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
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    if clean_args.help {
        clean_help();
        return Ok("Displayed help information.".to_string());
    }

    // Canonicalize exception paths for comparison - use HashSet for O(1) lookup
    let exception_paths: std::collections::HashSet<PathBuf> = clean_args.exceptions.iter()
        .filter_map(|e| {
            let path = configs.get_project_root().join(e);
            std::fs::canonicalize(&path).ok()
        })
        .collect();

    if clean_args.all && clean_args.force {
        for ext in &clean_args.ext {
            let ext_files = project_structure.get_files_with_extension(ext);
            let file_paths: Vec<PathBuf> = ext_files.iter().map(|f| f.get_path().clone()).collect();
            for file_path in file_paths {
                let canonical = std::fs::canonicalize(&file_path).unwrap_or(file_path.clone());
                if exception_paths.contains(&canonical) {
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
    println!("  --force            Enable deletion for extension-based cleaning");
    println!("  -e<file_path>      Exclude a file from cleaning (repeatable)");
    println!("  -e <file_path>     Same as above (space-separated form)");
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
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

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

    #[test]
    fn clean_parser_parses_exceptions_both_forms() {
        let parsed_cmd = ParsedCommand {
            command_type: crate::cmd_parser::cmd::Type::Clean,
            args: vec![
                "--all".to_string(),
                "--force".to_string(),
                ".o".to_string(),
                "-e".to_string(),
                "src/main.o".to_string(),
                "-e\"keep.o\"".to_string(),
            ],
        };

        let configs = Configs::test_config(None);
        let clean_args = clean_parser(&parsed_cmd, &configs);

        assert!(clean_args.exceptions.contains(&"src/main.o".to_string()));
        assert!(clean_args.exceptions.contains(&"keep.o".to_string()));
    }

    #[test]
    fn clean_execute_removes_build_dir_even_without_all() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("build").create_dir_all().unwrap();
        temp.child("build/old.bin").write_str("x").unwrap();

        let configs = Configs::default(root_path);
        let mut structure = ProjectStructure::new(&configs);

        let clean_args = CleanArgs {
            exceptions: vec![],
            ext: vec![".o".to_string()],
            all: false,
            force: false,
            help: false,
        };

        let result = super::execute(&clean_args, &configs, &mut structure);
        assert!(result.is_ok());
        assert!(!temp.child("build").path().exists());
    }

    #[test]
    fn clean_execute_does_not_delete_ext_files_without_force() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("build").create_dir_all().unwrap();
        temp.child("build/old.bin").write_str("x").unwrap();
        temp.child("delete.o").write_str("x").unwrap();

        let configs = Configs::default(root_path);
        let mut structure = ProjectStructure::new(&configs);

        let clean_args = CleanArgs {
            exceptions: vec![],
            ext: vec![".o".to_string()],
            all: true,
            force: false,
            help: false,
        };

        let result = super::execute(&clean_args, &configs, &mut structure);
        assert!(result.is_ok());

        assert!(temp.child("delete.o").path().exists());
        assert!(!temp.child("build").path().exists());
    }

    #[test]
    fn clean_execute_deletes_ext_files_with_force_except_exceptions() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();

        temp.child("build").create_dir_all().unwrap();
        temp.child("build/old.bin").write_str("x").unwrap();

        temp.child("delete.o").write_str("x").unwrap();
        temp.child("keep.o").write_str("x").unwrap();

        let configs = Configs::default(root_path);
        let mut structure = ProjectStructure::new(&configs);

        let clean_args = CleanArgs {
            exceptions: vec!["keep.o".to_string()],
            ext: vec![".o".to_string()],
            all: true,
            force: true,
            help: false,
        };

        let result = super::execute(&clean_args, &configs, &mut structure);
        assert!(result.is_ok());

        assert!(!temp.child("delete.o").path().exists());
        assert!(temp.child("keep.o").path().exists());
        assert!(!temp.child("build").path().exists());
    }
}

