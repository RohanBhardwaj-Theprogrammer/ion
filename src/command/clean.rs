use crate::cmd_parser::CmdType;

pub fn clean_cmd(_root_dir: &str, args: &CmdType) -> Result<String, Box<dyn std::error::Error>> {
    match args {
        CmdType::Clean(project_name) => {
            Ok(format!("Cleaned project: {}", project_name))
        },
        _ => Err("Invalid command type for clean_cmd".into())
    }
}