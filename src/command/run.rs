use std::process::Command;
use super::build::build_internal_cmd;
use super::file_ops::file_exists;
use super::list_file::single_file_selection;
use crate::cmd_parser::CmdType;

pub fn run_cmd(root_dir: &str, args: &CmdType) -> Result<String, Box<dyn std::error::Error>> {
    

    match args {
        CmdType::Run(filepath, input_file, output_file) => {
            // Auto-select file if not provided or doesn't exist
            let file = if filepath.is_empty() || !file_exists(filepath) {
                single_file_selection(root_dir, vec![".c", ".cpp"])
                    .map_err(|e| format!("File selection failed: {}", e))?
            } else {
                filepath.clone()
            };
            
            let full_path = format!("{}/{}", root_dir, file);
            run_internal_cmd(&full_path, Some(input_file.as_str()), Some(output_file.as_str()))
                .ok_or_else(|| "Run command failed".into())
        },
        _ => {
            Err("Invalid command type for run_cmd".into())
        }
    }
}





fn run_internal_cmd(file_path: &str, input_file: Option<&str>, output_file: Option<&str>) -> Option<String> {
    use std::fs::File;
    use std::process::Stdio;

    let executable_file_path = build_internal_cmd(file_path, "-O0", true).ok()?; 

    let mut cmd = Command::new(&executable_file_path);

    // Handle input redirection
    if let Some(input) = input_file {
        if !input.is_empty() && file_exists(input) {
            let file = File::open(input).ok()?;
            cmd.stdin(Stdio::from(file));
        }else { 
            cmd.stdin(Stdio::inherit());
        }
    }

    // Handle output redirection
    if let Some(output) = output_file {
        if !output.is_empty() {
            use std::fs::OpenOptions;
            
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output)
                .ok()?;
            cmd.stdout(Stdio::from(file));
        }
    }

    let output = cmd.output().ok()?;
    
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Some(format!("Error: {}", String::from_utf8_lossy(&output.stderr)))
    }
}
