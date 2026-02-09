pub fn ion_help() {
    const ION_HELP_MESSAGE: &str = r#"
    ION - C/cpp build system.
    Usage:
        ion <command> [options]"
    Commands:
        init        Initialize a new C/C++ project
        build       Build the project
        clean       Clean build artifacts
        check       Check source files for issues
        env         Display environment information
        version     Display version information
    Use "ion <command> --help" for more information on a specific command.
    
        "#;

    println!("{}", ION_HELP_MESSAGE);
}
