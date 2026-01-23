pub fn help() {
    println!("C/C++ Build System and Package Manager");
    println!();
    println!("Usage: c_cpp_build_system_n_pkg_manager build [options] <file>");
    println!();
    println!("Options:");
    println!("  -h, --help               Show this help message and exit");
    println!("  -i, --interactive        Run in interactive mode");
    println!("  -std <version>           Specify C/C++ standard version (default: 11)");
    println!("  -opt <level>             Set optimization level (0-3, default: 0)");
    println!("  -I, -i, --include <dir>  Add extra include directory");
    println!("  --<profileName>          Use specified build profile");
    println!();
    println!("Example:");
    println!("  c_cpp_build_system_n_pkg_manager build -std 17 -opt 2 -I ./include main.cpp");
}
