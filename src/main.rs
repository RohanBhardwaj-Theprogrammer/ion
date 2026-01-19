
use std::env;

pub mod cmd_parser;
pub mod command ;  

use command::{build_cmd, init_cmd, run_cmd, clean_cmd, check_cmd, help_cmd};
use cmd_parser::CmdType;

fn main(){ 
    let args: Vec<String> = env::args().collect();
    let parsed_cmd = cmd_parser::parser_cmd(args);

    let root_dir = env::current_dir().unwrap().to_str().unwrap().to_string(); 
    
    match &parsed_cmd {
        CmdType::Help => { 
            help_cmd(); 
        },
        CmdType::Init(_) => { 
            match init_cmd(&root_dir, &parsed_cmd) {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Error: {}", e),
            }
        },
        CmdType::Build(_, _, _) => { 
            match build_cmd(&root_dir, &parsed_cmd) {
                Ok(msg) => println!("✅ Build successful: {}", msg),
                Err(e) => eprintln!("❌ Build failed: {}", e),
            }
        },
        CmdType::Run(_, _, _) => { 
            match run_cmd(&root_dir, &parsed_cmd) {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Error: {}", e),
            }
        },
        CmdType::Clean(_) => { 
            match clean_cmd(&root_dir, &parsed_cmd) {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Error: {}", e),
            }
        },
        CmdType::Check(_) => { 
            match check_cmd(&root_dir, &parsed_cmd) {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Error: {}", e),
            }
        },
    }
}