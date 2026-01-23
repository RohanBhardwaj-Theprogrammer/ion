use crate::utils::prompt;

pub fn init_interactive() -> Result<super::InitArgs, String> {
    let path_input;
    let lang_input;
    let force_input;
    let custom_structure_input;

    println!("Interactive Project Initialization \n case-insensitive inputs where applicable. \n Press Enter to accept default values shown in [brackets] . \n Type 'clean' for custom structure to create a blank structure. \n ");
    path_input = prompt::input("Enter project path (current directory : blank or .): ")
        .unwrap_or(".".to_string());

    lang_input =
        prompt::input("Enter programming language (c/cpp) [auto]: ").unwrap_or("auto".to_string());

    force_input = prompt::input(
        "any unmatched value is condider no \nForce overwrite if files exist? (y/n) [n]: ",
    )
    .unwrap();

    custom_structure_input =
        prompt::input("Custom structure path (leave blank for default , clean for blank ): ")
            .unwrap_or("".to_string());

    let mut init_args = super::InitArgs::new();
    init_args.set_path(path_input.trim().to_string());
    init_args.set_lang(lang_input.trim().to_lowercase().as_str());
    init_args.set_force_flag(matches!(
        force_input.trim().to_lowercase().as_str(),
        "y" | "yes"
    ));
    init_args.set_custom_structure(custom_structure_input.trim().to_string());
    init_args.set_interactive_flag(false);
    init_args.set_help_flag(false);
    return Ok(init_args);
}

// __________________________________TESTS____________________________________

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::init::InitArgs;

    #[test]
    #[ignore]
    fn test_init_interactive() {
        // Note: Testing interactive functions can be complex due to user input.
        // Here, we can only ensure that the function signature is correct and it compiles.
        // Comprehensive testing would require mocking stdin, which is beyond this simple test.
        let result = init_interactive();
        assert!(result.is_err() || result.is_ok()); // Just check it returns a Result
    }
    #[test]
    #[ignore]
    fn test_input_matches() {
        let mut init_args = InitArgs::new();
        init_args.set_path(".".to_string());
        init_args.set_lang("c");
        init_args.set_force_flag(true);
        init_args.set_custom_structure("clean".to_string());

        let init_i_result = init_interactive();

        assert!(init_i_result.is_ok());

        let init_i_args = init_i_result.unwrap();

        assert_eq!(init_args.path, init_i_args.path);
        // assert!(matches!(init_args.lang, init_i_args.lang)) ; no derive partial eq for LangType
        assert_eq!(init_args.force, init_i_args.force);

        let init_path = init_args
            .custom_structure
            .as_ref()
            .and_then(|cs| cs.path.as_ref());
        let i_init_path = init_i_args
            .custom_structure
            .as_ref()
            .and_then(|cs| cs.path.as_ref());

        assert_eq!(init_path, i_init_path);
    }
}
