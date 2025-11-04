
use super::cmd_type::CmdType;


pub fn parser_cmd(args: Vec<String>) -> CmdType {
    if args.len() < 2 {
        return CmdType::Help;
    }
    
    match args[1].as_str() {
        "-h" | "--help" | "help" => CmdType::Help,
        "run" => CmdType::Run(
            args.get(2).cloned().unwrap_or_default(),
            args.get(3).cloned().unwrap_or_default(),
            args.get(4).cloned().unwrap_or_default()
        ),
        "build" => CmdType::Build(
            args.get(2).cloned().unwrap_or_default(),
            String::new(),
            false
        ),
        "clean" => CmdType::Clean(args.get(2).cloned().unwrap_or_default()),
        "check" => CmdType::Check(args.get(2).cloned().unwrap_or_default()),
        "init" => CmdType::Init(args.get(2).cloned().unwrap_or_default()),
        _ => CmdType::Help,
    }
}