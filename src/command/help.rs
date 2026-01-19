pub fn help_cmd() { 
    println!(r#"
╔══════════════════════════════════════════════════════════════════════════════╗
║                    C/C++ Build System & Package Manager                      ║
╚══════════════════════════════════════════════════════════════════════════════╝

USAGE:
    cbuild <COMMAND> [ARGUMENTS] [OPTIONS]

COMMANDS:
    help                           Display this help message
    init    <project_name>         Initialize a new C/C++ project
    build   <file_path>            Compile a C/C++ source file
    run     <file_path> [in] [out] Run a compiled program with optional I/O redirection
    check   <file_path>            Check syntax without compiling
    clean   <project_name>         Remove build artifacts

EXAMPLES:
    cbuild init my_project
    cbuild build main.cpp
    cbuild build main.cpp -O2
    cbuild run main.cpp
    cbuild run main.cpp input.txt output.txt
    cbuild check main.cpp
    cbuild clean my_project

OPTIONS:
    -h, --help                     Show this help message
    -O0, -O1, -O2, -O3            Set optimization level (for build command)

For more information, visit: https://github.com/your-repo
"#);
}
