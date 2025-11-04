use std::process::Command;
use std::error::Error;
use crate::cmd_parser::CmdType;
use super::file_ops::file_exists;
use super::list_file::single_file_selection;

macro_rules! compiler {
    () => {
        "g++"
    };
}

/// Handles the `check` command type.
pub fn check_cmd(root_dir: &str, args: &CmdType) -> Result<String, Box<dyn Error>> {
    match args {
        CmdType::Check(file_path) => {
            // Auto-select if no file provided
            let file = if file_path.is_empty() || !file_exists(file_path) {
                single_file_selection(root_dir, vec![".c", ".cpp"])?
            } else {
                file_path.to_string()
            };
            
            check_internal_cmd(&file)
        },
        _ => Err("Invalid command type for check_cmd".into()),
    }
}

/// Performs syntax-only checking on a given C/C++ source file.
fn check_internal_cmd(file_path: &str) -> Result<String, Box<dyn Error>> {
    if !file_exists(file_path) {
        return Err("Source file does not exist".into());
    }

    let status = Command::new(compiler!())
        .arg("-std=c++20")
        .arg("-fsyntax-only")
        .arg(file_path)
        .status()?;

    if status.success() {
        Ok(format!("✅ No syntax errors found in {}", file_path))
    } else {
        Err(format!("❌ Syntax errors found in {}", file_path).into())
    }
}
