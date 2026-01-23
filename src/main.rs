use std::path::PathBuf;

use ion::cmd_parser::parser::parse_args;
use ion::command;
use ion::config::Configs;
use ion::state;
use ion::state::ProjectStructure;
use ion::utils;

fn main() {
    //first intialize the cofigs  and //FIXME: all these two to be done in the async or parallel way

    let project_root = utils::find_cbuild(".");
    let mut project_structure: ProjectStructure;
    let mut configs: Configs;

    if project_root.is_none() {
        configs = Configs::default(PathBuf::from("."));
        project_structure = state::structure::ProjectStructure::none();
    } else {
        let root = project_root.as_deref().unwrap_or(".");
        configs = Configs::default(PathBuf::from(root));
        project_structure = state::structure::ProjectStructure::new(&configs);
    }

    // takes the command line arguments and parses them
    let cli_args = std::env::args().collect::<Vec<String>>();
    let parsed_command = parse_args(cli_args);

    let result = command::execute(parsed_command, &mut configs, &mut project_structure);

    match result {
        Ok(message) => {
            println!("{}", message);
        }
        Err(error) => {
            eprintln!("Error: {}", error);
        }
    }
}
