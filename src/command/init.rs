use std::fs;
use std::path::Path;
use std::error::Error;
use crate::cmd_parser::CmdType;

pub fn init_cmd(_root_dir: &str, args: &CmdType) -> Result<String, Box<dyn Error>> {
    match args {
        CmdType::Init(project_name) => {
            init_cmd_internal(project_name)
        },
        _ => Err("Invalid command type for init_cmd".into())
    }
}

fn init_cmd_internal(root_path: &str) -> Result<String, Box<dyn Error>> {
    // Initializes a hidden-like folder named `.ccbpkg` in the given root path.
    // Note: Actually making the folder hidden is OS-specific and not handled here.
    let project_path = Path::new(root_path);

    let init_path = project_path.join(".ccbpkg");
    if init_path.exists() {
        return Err("Project already initialized".into());
    }

    fs::create_dir_all(&init_path)?;
    
    Ok(format!("Initialized project at: {}", init_path.display()))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_cmd() {
        // Use a workspace-relative test directory to avoid hard-coding absolute paths
        let test_project = Path::new("tests").join("test_project");

        // Ensure a clean slate for the test
        let init_path = test_project.join(".ccbpkg");
        if init_path.exists() {
            fs::remove_dir_all(&init_path).unwrap();
        }

        // Create the CmdType::Init argument and call init_cmd with both parameters
        let cmd = CmdType::Init(test_project.to_str().unwrap().to_string());
        let result = init_cmd(test_project.to_str().unwrap(), &cmd);
        assert!(result.is_ok());

        let expected_msg = format!(
            "Initialized project at: {}",
            init_path.display()
        );
        assert_eq!(result.unwrap(), expected_msg);

        assert!(init_path.exists(), ".ccbpkg directory should exist after init_cmd");

        // Cleanup
        fs::remove_dir_all(&init_path).unwrap();
    }
}