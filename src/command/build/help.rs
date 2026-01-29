pub fn help() {
    println!("C / C++ Build System and Package Manager\n");
    println!("Build Command\n=============");
    println!("Usage:");
    println!("  build [previousCommand] | <fileName | int | \"\" | .> [-opt <int>] [-std <int>] [-I <includePath> ...] [-objects <objectPath> ...] | <--profileName> | <--help | -help | -h> | <-interactive | -i>\n");
    println!("Arguments / Flags:");
    println!("  /<int>                   Reuse a previous command/target by index (e.g., /1, /2)");
    println!("  <fileName>               File or target to build");
    println!("  <int>                    Index of a previously known target");
    println!("  \"\"                      Special value, usually 'last used' or configured default target");
    println!("  .                        Special value, usually 'current project' or configured main target");
    println!("  -opt <int>               Optimization level/profile ID");
    println!(
        "  -std <int>               Standard/profile ID (e.g., C/C++ standard, build profile)"
    );
    println!("  -I <includePath> ...     Additional include directories for compilation (can be repeated)");
    println!(
        "  -objects <objectPath> ... Extra object files to link into the build (can be repeated)"
    );
    println!("  --<profileName>          Use a named build profile (e.g., --debug, --release)");
    println!("  -h, --help, -help        Show this help message and exit");
    println!("  -i, --interactive        Run build in interactive mode\n");
    println!("Notes:");
    println!("  - /<int> is written without a space, e.g., /1, /2");
    println!("  - -I and -objects can be repeated or accept multiple paths");
    println!("  - \"\" and . follow the same 'special target' semantics as in 'run'\n");
    println!("Examples:");
    println!("  build main.cpp -std 17 -opt 2 -I ./include");
    println!("  build /1 --debug");
    println!("  build . -objects obj1.o obj2.o");
    println!("  build --release\n");
    println!("For more details, see the documentation or use 'build --help'.");
}
