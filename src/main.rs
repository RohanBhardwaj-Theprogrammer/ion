// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Rohan Bhardwaj
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ion::cmd_parser::parser::parse_args;
use ion::command;
use ion::config::Configs;
use ion::state::ProjectStructure;
use ion::utils;

//REVIEW: comprehensive testing remaining, and non-.ion configs handling to be made , configs and project structure,

fn main() {
    let mut configs: Configs = match utils::find_cbuild(".") {
        Some(path) => Configs::init(path)
            .expect("[unexepcted error] : encountered unexpeted error, while parsing the configs "),
        None => {
            eprintln!("[warning] :\t Could not find the .{} file in the current or parent directories. defaulting to cwd.", ion::constants::PROGRAM_NAME);
            let current_dir =
                std::env::current_dir().expect("[Path Erros]: Unable to resolve Paths");
            Configs::default(current_dir)
        }
    };

    let mut project_structure = ProjectStructure::new(&configs);

    #[cfg(any(test, debug_assertions))]
    project_structure.debug_print();

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
