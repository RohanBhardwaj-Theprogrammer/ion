use crate::cmd_parser::{cmd::Type, parser::ParsedCommand};
use crate::config::Configs;
use crate::utils::parse_path_arg;
use std::path::Path;

use super::init_fs_initializer;

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

pub fn execute(init_args: InitArgs) -> Result<String, String> {
    if init_args.help {
        super::help::display_init_help();
        return Ok("Displayed help".to_string());
    }

    if crate::utils::is_initialized_project(&init_args.path) && !init_args.force {
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

    let dot_folder = init_fs_initializer::generate_dotfile(&init_args.path, init_args.force)?;

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
                init_fs_initializer::generate_project_structure(
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
                init_fs_initializer::generate_project_structure(
                    project_path,
                    None,
                    &init_args.lang,
                    init_args.force,
                )?;
            }
        } else {
            init_fs_initializer::generate_project_structure(
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
        None, // std version
        None, // compiler path
    )?;

    println!("Initialized cbuild project at {}", init_args.path);
    Ok("Project initialized successfully".to_string())
}

//normalize wrapper around all

pub fn init(args: ParsedCommand) -> Result<String, String> {
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

    execute(init_args)
}

//__________________________________TESTS____________________________________
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_parser::cmd::Type;
    use crate::cmd_parser::parser::ParsedCommand;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    fn pc(command_type: Type, args: &[&str]) -> ParsedCommand {
        ParsedCommand {
            command_type,
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn init_args_for_dir(dir: &std::path::Path) -> InitArgs {
        InitArgs {
            lang: LangType::Cpp,
            path: dir.to_string_lossy().to_string(),
            custom_structure: None,
            force: false,
            help: false,
            interactive: false,
        }
    }

    // ---------------- CLI-level init() behavior ----------------

    #[test]
    fn init_rejects_non_init_commands() {
        let args = pc(Type::Run, &["--help"]);
        let result = init(args);
        assert!(result.is_err());
    }

    #[test]
    fn init_help_flag_short_circuits() {
        let args = pc(Type::Init, &["--help"]);
        let result = init(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Displayed help");
    }

    // ---------------- execute() behavior (filesystem effects) ----------------

    #[test]
    fn execute_errors_when_project_exists_without_force() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join(format!(".{}", crate::constants::PROGRAM_NAME))).unwrap();

        let result = execute(init_args_for_dir(root));
        assert!(result.is_err());
    }

    #[test]
    fn execute_creates_default_structure_and_configs() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let result = execute(init_args_for_dir(root));
        assert!(result.is_ok());

        let dot = root.join(format!(".{}", crate::constants::PROGRAM_NAME));
        assert!(dot.is_dir());
        assert!(dot.join("project_config.json").is_file());
        assert!(dot.join("config.toml").is_file());

        assert!(root.join("src").is_dir());
        assert!(root.join("include").is_dir());
        assert!(root.join("build").is_dir());
        assert!(root.join("src").join("main.cpp").is_file());
    }

    #[test]
    fn execute_force_overwrites_existing_project_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create an initialized marker and some existing files.
        let dot = root.join(format!(".{}", crate::constants::PROGRAM_NAME));
        std::fs::create_dir_all(&dot).unwrap();
        std::fs::write(dot.join("old.txt"), "old").unwrap();

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src").join("main.cpp"), "old content").unwrap();

        let mut args = init_args_for_dir(root);
        args.force = true;

        let result = execute(args);
        assert!(result.is_ok());

        // Dotfolder contents should be truncated.
        assert!(!dot.join("old.txt").exists());

        // main.cpp should be overwritten with template content.
        let main = std::fs::read_to_string(root.join("src").join("main.cpp")).unwrap();
        assert!(main.contains("Hello, World"));
    }

    #[test]
    fn execute_clean_custom_structure_skips_default_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let mut args = init_args_for_dir(root);
        args.custom_structure = Some(CustomStructure {
            path: None,
            clean: true,
        });

        let result = execute(args);
        assert!(result.is_ok());

        let dot = root.join(format!(".{}", crate::constants::PROGRAM_NAME));
        assert!(dot.is_dir());

        assert!(!root.join("src").exists());
        assert!(!root.join("include").exists());
        assert!(!root.join("build").exists());
    }

    #[test]
    fn execute_custom_structure_path_creates_custom_tree_and_copies_format_file() {
        let project = TempDir::new().unwrap();
        let root = project.path();

        // Create format JSON outside the project root to avoid path collisions.
        let format_dir = TempDir::new().unwrap();
        let format = format_dir.child("format.json");
        format
            .write_str(
                r#"{
                "name": "my_project",
                "folders": [
                    { "name": "src", "files": ["main.c"] }
                ]
                }"#,
            )
            .unwrap();

        let mut args = init_args_for_dir(root);
        args.force = true; // required because generate_custom_structure errors if target exists
        args.custom_structure = Some(CustomStructure {
            path: Some(format.path().to_string_lossy().to_string()),
            clean: false,
        });

        let result = execute(args);
        assert!(result.is_ok());

        // Custom structure should exist under root/my_project/...
        assert!(root.join("my_project").is_dir());
        assert!(root.join("my_project").join("src").join("main.c").is_file());

        // init.rs currently copies the structure file into the project root.
        assert!(root.join("format.json").is_file());
    }

    // ---------------- parser behavior ----------------

    #[test]
    fn parse_init_args_defaults() {
        let parsed = pc(Type::Init, &[]);
        let result = parse_init_args(parsed);
        assert!(matches!(result.lang, LangType::Cpp));
        assert_eq!(result.path, ".");
        assert!(!result.force);
        assert!(!result.help);
        assert!(!result.interactive);
        assert!(result.custom_structure.is_none());
    }

    #[test]
    fn parse_init_args_parses_flags_and_path() {
        let parsed = pc(Type::Init, &["-cpp", "--force", "-h", "-i", "my_project"]);
        let result = parse_init_args(parsed);
        assert!(matches!(result.lang, LangType::Cpp));
        assert!(result.force);
        assert!(result.help);
        assert!(result.interactive);
        assert_eq!(result.path, "my_project");
    }

    #[test]
    fn parse_init_args_parses_custom_structure_clean() {
        let parsed = pc(Type::Init, &["--custom-structure=clean"]);
        let result = parse_init_args(parsed);
        let custom = result.custom_structure.expect("custom structure expected");
        assert!(custom.path.is_none());
        assert!(custom.clean);
    }
}
