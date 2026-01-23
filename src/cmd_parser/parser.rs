use crate::cmd_parser::cmd::Type;

pub struct ParsedCommand {
    pub command_type: Type,
    pub args: Vec<String>, // no ref , to avoid lifetime issues
}

pub fn parse_args(mut args: Vec<String>) -> ParsedCommand {
    // args[0] = binary name, args[1] = command (if present)
    let command_str = args.get(1).map(String::as_str).unwrap_or("help"); // args[2]

    let command_type = match command_str.to_lowercase().as_str() {
        "build" => Type::Build,
        "run" => Type::Run,
        "check" => Type::Check,
        "init" => Type::Init,
        "clean" => Type::Clean,
        "log" => Type::Log,
        "env" => Type::Env,
        "config" => Type::Config,
        "help" => Type::Help,
        cmd if cmd.starts_with('/') => Type::Previous,
        "-v" | "--version" | "version" => Type::Version,
        _ => Type::Help,
    };

    if !args.is_empty() {
        args.remove(0); // remove binary name
    }
    if !args.is_empty()
        && match command_type {
            Type::Previous => false,
            _ => true,
        }
    {
        args.remove(0); // remove command name
    }

    ParsedCommand {
        command_type,
        // we expose args after the command itself
        args,
    }
}

// _________________________________ TESTS _________________________________________________

/// Tests for the command parser
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_parser::cmd::Type;

    #[test]
    fn test_parse_build_with_arg() {
        let args = vec![
            "cbuild".to_string(),
            "build".to_string(),
            "src/main.c".to_string(),
        ];

        let parsed = parse_args(args);

        assert!(matches!(parsed.command_type, Type::Build));
        assert_eq!(parsed.args, vec!["src/main.c".to_string()]);
    }

    #[test]
    fn test_all_type() {
        let commands = vec![
            ("build", Type::Build),
            ("run", Type::Run),
            ("check", Type::Check),
            ("init", Type::Init),
            ("clean", Type::Clean),
            ("log", Type::Log),
            ("env", Type::Env),
            ("config", Type::Config),
            ("help", Type::Help),
            ("version", Type::Version),
            ("/2", Type::Previous),
        ];

        for (cmd_str, expected_type) in commands {
            let args = vec!["cbuild".to_string(), cmd_str.to_string()];
            let parsed = parse_args(args);
            assert_eq!(parsed.command_type as u8, expected_type as u8);
        }
    }

    #[test]
    fn test_sub_args() {
        let args = vec![
            "cbuild".to_string(),
            "build".to_string(),
            "--release".to_string(),
            "--target=x86".to_string(),
        ];

        let parsed = parse_args(args);

        assert!(matches!(parsed.command_type, Type::Build));
        assert_eq!(
            parsed.args,
            vec!["--release".to_string(), "--target=x86".to_string()]
        );
    }

    #[test]
    fn test_no_command_defaults_to_help() {
        let args = vec!["cbuild".to_string()];

        let parsed = parse_args(args);

        assert!(matches!(parsed.command_type, Type::Help));
        assert!(parsed.args.is_empty());
    }

    #[test]
    fn test_previous_args() {
        let args = vec!["cbuild".to_string(), "/2".to_string()];

        let parsed = parse_args(args);

        assert_eq!(parsed.args, vec!["/2".to_string()]);
    }
}
