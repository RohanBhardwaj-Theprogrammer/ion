use toml::ser;

use crate::cmd_parser::{cmd::Type, parser::ParsedCommand};
use crate::config::Configs;
use crate::utils::parse_path_arg;
use std::path::Path;

use super::create_init_fs;

pub enum InitArgsType {
    Lang,
    Path,
    CustomStructure,
    Force,
    Help,
    Interactive,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LangType {
    // Auto,
    C,
    Cpp,
}

#[derive(Clone, Debug)]
pub struct CustomStructure {
    pub path: Option<String>,
    pub clean: bool,
}

#[derive(Clone)]
pub struct InitArgs {
    pub lang: LangType,
    pub path: String,
    pub custom_structure: Option<CustomStructure>,
    pub force: bool,
    pub help: bool,
    pub interactive: bool,
}

impl InitArgs {
    pub fn new() -> Self {
        // initialize with default values
        InitArgs {
            lang: LangType::Cpp,
            path: String::from("."),
            custom_structure: None,
            force: false,
            help: false,
            interactive: false,
        }
    }

    pub fn set_lang(&mut self, lang: &str) {
        // expected lowercase letter only
        let mut lang_arg = lang;
        if lang_arg.starts_with('-') {
            lang_arg = &lang[1..];
        }
        match lang_arg {
            "c" => self.lang = LangType::C,
            "cpp" | "c++" | "cxx" => self.lang = LangType::Cpp,
            _ => self.lang = LangType::Cpp, // default to cpp
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn set_force_flag(&mut self, force: bool) {
        self.force = force;
    }

    pub fn set_help_flag(&mut self, help: bool) {
        self.help = help;
    }

    pub fn set_interactive_flag(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    pub fn set_custom_structure(&mut self, path: String) {
        if path == "clean" {
            self.custom_structure = Some(CustomStructure {
                path: None,
                clean: true,
            });
        } else {
            self.custom_structure = Some(CustomStructure {
                path: Some(path),
                clean: false,
            });
        }
    }
}

pub fn parse_init_args(args: ParsedCommand) -> InitArgs {
    let mut init_args = InitArgs::new();

    for arg in args.args.iter() {
        let arg_str = arg.to_lowercase();

        match arg_str.as_str() {
            "-c" | "-cpp" | "--cpp" | "--c++" | "--cxx" => {
                init_args.set_lang(arg_str.as_str());
            }
            "-f" | "--force" => {
                init_args.set_force_flag(true);
            }
            "-h" | "--help" | "-help" => {
                init_args.set_help_flag(true);
            }
            "-i" | "--interactive" => {
                init_args.set_interactive_flag(true);
            }
            arg_path if arg_path.starts_with("--custom-structure=") => {
                let raw_value = arg_path.splitn(2, '=').nth(1).unwrap_or("");
                let structure_value = crate::utils::trim::quotes(raw_value.to_string())
                    .map_err(|e| {
                        println!("Warning: {}", e);
                        e
                    })
                    .unwrap_or(raw_value.to_string());

                init_args.set_custom_structure(structure_value);
            }
            _ => {
                init_args.set_path(parse_path_arg(arg));
            }
        }
    }

    init_args
}

// (parser-related unit tests were moved to the bottom with the rest of the init tests)

pub fn execute(init_args: InitArgs, _configs: &Configs) -> Result<String, String> {
    if init_args.help {
        super::help::display_init_help();
        return Ok("Displayed help".to_string());
    }

    if crate::utils::is_cbuild_project(&init_args.path) && !init_args.force {
        #[cfg(any(debug_assertions, test))]
        {
            println!(
                "Debug: init , Detected existing cbuild project at {} and --force not set ",
                init_args.path
            );
        }

        return Err(format!(
            "A cbuild project already exists at {}",
            init_args.path
        ));
    }

    #[cfg(any(debug_assertions, test))]
    {
        println!(
            "Debug: init , Creating cbuild project at {} ",
            init_args.path
        );
    }

    let dot_folder = create_init_fs::generate_dotfile(&init_args.path, init_args.force)?;

    let skip_default_structure = init_args
        .custom_structure
        .as_ref()
        .map(|custom| custom.clean)
        .unwrap_or(false);

    let project_path = Path::new(&init_args.path);

    //Review:  here we pass lang by reference to avoid ownership issues
    if !skip_default_structure {
        if let Some(custom) = init_args.custom_structure.as_ref() {
            if let Some(structure_path) = custom.path.as_ref() {
                let custom_structure_path = Path::new(structure_path);
                create_init_fs::generate_project_structure(
                    project_path,
                    Some(custom_structure_path),
                    &init_args.lang,
                    init_args.force,
                )?;

                // moving the custom folder inside the dot folder

                let custom_structure_file_cp =
                    project_path.join(custom_structure_path.file_name().unwrap());
                match std::fs::copy(custom_structure_path, custom_structure_file_cp) {
                    Ok(_) => (),
                    Err(e) => {
                        println!(
                            "Failed to copy custom structure file to project .{} folder: {}",
                            crate::constants::PROGRAM_NAME,
                            e
                        );
                    }
                };
            } else {
                create_init_fs::generate_project_structure(
                    project_path,
                    None,
                    &init_args.lang,
                    init_args.force,
                )?;
            }
        } else {
            create_init_fs::generate_project_structure(
                project_path,
                None,
                &init_args.lang.clone(),
                init_args.force,
            )?;
        }
    }

    Configs::new(
        Path::new(&init_args.path),
        Some(&dot_folder),
        init_args.lang,
    )?;

    println!("Initialized cbuild project at {}", init_args.path);
    Ok("Project initialized successfully".to_string())
}

//normalize wrapper around all

pub fn init(args: ParsedCommand, configs: &Configs) -> Result<String, String> {
    #[cfg(any(debug_assertions, test))]
    {
        let printable_args = {
            let mut s = String::new();
            for arg in args.args.iter() {
                s.push_str(&format!("{} \n ", arg));
            }
            s
        };

        println!(
            "Debug: init , Initiating with Args : \n \t {}",
            printable_args
        );
    }

    if args.command_type != Type::Init {
        return Err("Invalid command type for init".to_string());
    }

    let mut init_args = parse_init_args(args);
    #[cfg(any(debug_assertions, test))]
    {
        let init_args_debug = init_args.clone();

        let printable_init_args = format!( "Lang : {:?} \n Path : {} \n Custom Structure : {:?} \n Force : {} \n Help : {} \n Interactive : {} " ,
            init_args_debug.lang ,
            init_args_debug.path ,
            init_args_debug.custom_structure ,
            init_args_debug.force ,
            init_args_debug.help ,
            init_args_debug.interactive,
        ) ;

        println!(
            "Debug: init , Parsed Init Args : \n \t {}",
            printable_init_args
        );
    }

    if init_args.interactive {
        init_args = crate::command::init::init_interactive::init_interactive()?;
    }

    execute(init_args, configs)
}

//__________________________________TESTS____________________________________
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_parser::cmd::Type;
    use crate::command::init::LangType;
    use crate::constants::PROGRAM_NAME;
    use std::fs;
    use std::path::Path;

    fn parsed_vec(args: Vec<String>, command_type: Type) -> ParsedCommand {
        ParsedCommand { command_type, args }
    }

    // helper used by the parser unit tests (slice-based)
    fn parsed_slice(args: &[&str]) -> ParsedCommand {
        ParsedCommand {
            command_type: Type::Init,
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn cleanup(path: &str) {
        if Path::new(path).exists() {
            fs::remove_dir_all(path).unwrap();
        }
        let dot_dir = format!("{}/.{}", path, PROGRAM_NAME);
        if Path::new(&dot_dir).exists() {
            fs::remove_dir_all(dot_dir).unwrap();
        }
    }

    fn temp_dir(name: &str) -> String {
        let path = format!("test_init_cmd_{}", name);
        cleanup(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn rejects_non_init_commands() {
        let args = parsed_vec(vec!["--help".into()], Type::Run);
        let configs = Configs::test_config(None);
        let result = init(args, &configs);
        assert!(result.is_err());
    }

    #[test]
    fn help_flag_short_circuits() {
        let args = parsed_vec(vec!["--help".into()], Type::Init);
        let configs = Configs::test_config(None);
        let result = init(args, &configs);
        assert!(result.is_ok());
    }

    #[test]
    fn errors_when_project_exists_without_force() {
        let dir = temp_dir("existing");
        let dot_dir = format!("{}/.{}", &dir, PROGRAM_NAME);
        fs::create_dir_all(&dot_dir).unwrap();

        let args = parsed_vec(vec![dir.clone()], Type::Init);
        let configs = Configs::test_config(None);
        let result = init(args, &configs);

        assert!(result.is_err());
        cleanup(&dir);
    }

    #[test]
    fn creates_project_structure_by_default() {
        let dir = temp_dir("structure");
        let args = parsed_vec(vec!["--force".into(), dir.clone()], Type::Init);
        let configs = Configs::test_config(None);
        let result = init(args, &configs);
        assert!(result.is_ok());

        let dot_dir = format!("{}/.{}", dir, PROGRAM_NAME);
        assert!(Path::new(&dot_dir).exists());
        assert!(Path::new(&format!("{}/src", dir)).exists());
        assert!(Path::new(&format!("{}/include", dir)).exists());
        assert!(Path::new(&format!("{}/build", dir)).exists());

        cleanup(&dir);
    }

    #[test]
    fn force_allows_overwriting_existing_project() {
        let dir = temp_dir("force_overwrite");
        // create a marker so project is considered existing
        let dot_dir = format!("{}/.{}", &dir, PROGRAM_NAME);
        fs::create_dir_all(&dot_dir).unwrap();

        let args = parsed_vec(vec!["--force".into(), dir.clone()], Type::Init);
        let configs = Configs::test_config(None);
        let result = init(args, &configs);

        assert!(result.is_ok());
        // structure should be recreated
        assert!(Path::new(&format!("{}/src", dir)).exists());
        assert!(Path::new(&format!("{}/include", dir)).exists());
        assert!(Path::new(&format!("{}/build", dir)).exists());

        cleanup(&dir);
    }

    #[test]
    fn default_structure_created_even_with_custom_path() {
        let dir = temp_dir("custom_path_structure");
        let args = parsed_vec(
            vec![
                "--force".into(),
                "--custom-structure=some/alt".into(),
                dir.clone(),
            ],
            Type::Init,
        );
        let configs = Configs::test_config(None);
        let result = init(args, &configs);

        assert!(result.is_ok());
        // clean flag is false, so default structure should still be created
        assert!(Path::new(&format!("{}/src", dir)).exists());
        assert!(Path::new(&format!("{}/include", dir)).exists());
        assert!(Path::new(&format!("{}/build", dir)).exists());

        cleanup(&dir);
    }

    #[test]
    fn skips_default_structure_for_clean_custom_structure() {
        let dir = temp_dir("clean");
        let args = parsed_vec(
            vec![
                "--force".into(),
                "--custom-structure=clean".into(),
                dir.clone(),
            ],
            Type::Init,
        );
        let configs = Configs::test_config(None);
        let result = init(args, &configs);
        assert!(result.is_ok());

        let dot_dir = format!("{}/.{}", dir, PROGRAM_NAME);
        assert!(Path::new(&dot_dir).exists());
        assert!(!Path::new(&format!("{}/src", dir)).exists());
        assert!(!Path::new(&format!("{}/include", dir)).exists());
        assert!(!Path::new(&format!("{}/build", dir)).exists());

        cleanup(&dir);
    }

    // --------- parser-related tests moved here ---------
    #[test]
    fn uses_defaults_when_no_args() {
        let result = parse_init_args(parsed_slice(&[]));

        assert!(matches!(result.lang, LangType::Cpp));
        assert_eq!(result.path, ".");
        assert!(!result.force);
        assert!(!result.help);
        assert!(!result.interactive);
        assert!(result.custom_structure.is_none());
    }

    #[test]
    // ignore the unrecognizable variables
    fn parses_flags_and_path() {
        let result = parse_init_args(parsed_slice(&["-cpp", "--force", "-h", "-i", "my_project"]));

        assert!(matches!(result.lang, LangType::Cpp));
        assert!(result.force);
        assert!(result.help);
        assert!(result.interactive);
        assert_eq!(result.path, "my_project");
    }

    #[test]
    fn parses_custom_structure_path() {
        let result = parse_init_args(parsed_slice(&["--custom-structure=\"src/,include/\""]));
        let custom = result.custom_structure.expect("custom structure expected");

        assert_eq!(custom.path, Some("src/,include/".to_string()));
        assert!(!custom.clean);
    }

    #[test]
    fn parses_custom_structure_clean_flag() {
        let result = parse_init_args(parsed_slice(&["--custom-structure=clean"]));
        let custom = result.custom_structure.expect("custom structure expected");

        assert!(custom.path.is_none());
        assert!(custom.clean);
    }

    #[test]
    fn parses_path_arg_tests() {
        // Current directory cases
        assert_eq!(
            parse_path_arg(""),
            std::env::current_dir().unwrap().to_string_lossy()
        );
        assert_eq!(
            parse_path_arg("."),
            std::env::current_dir().unwrap().to_string_lossy()
        );
        println!(
            "Current Dir: {}",
            std::env::current_dir().unwrap().to_string_lossy()
        );
        println!("Parsed Arg: {}", parse_path_arg("."));
        // Parent directory cases
        let parent = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        assert_eq!(parse_path_arg(".."), parent.to_string_lossy());
        assert_eq!(parse_path_arg("../"), parent.to_string_lossy());

        let grandparent = parent.parent().unwrap().to_path_buf();
        assert_eq!(parse_path_arg("../../"), grandparent.to_string_lossy());
        assert_eq!(
            parse_path_arg("../../some/path"),
            grandparent.join("some/path").to_string_lossy()
        );

        // Normal path case
        assert_eq!(parse_path_arg("some/path"), "some/path");
    }
}
