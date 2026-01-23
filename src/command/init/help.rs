pub fn display_init_help() {
    println!("{}", init_help_message);
}

const init_help_message: &str = r#"
Usage: 
      ion init [-c|-cpp] <path|""|.> [--force] [--custom-structure="<structure|clean|""">] | [--help]
Options:
  --path <PATH>               Specify the path to initialize the project
    --lang <LANGUAGE>           Specify the programming language (e.g., c, cpp
    --force                     Force initialization even if the directory is not empty
    --custom-structure <STRUCTURE>  Define a custom project structure
    --interactive               Run in interactive mode to set up the project
    --help                      Display this help message
Example:

    cppbuild init --path ./my_project --lang cpp --force

    --custom-structure="<structure|clean|>"

    structure : file structure as : 

    {
        "name": "my_project",
        folders : [
            "src",
            "include",
            "build",
            "tests" : {
                "name}
        ],
        files : [
            "README.md",
            "LICENSE"
            ]




    command : ion init [-c|-cpp] <path|""|.> [--force] [--custom-structure="<structure|clean|""">] | [--help] 
"#;
