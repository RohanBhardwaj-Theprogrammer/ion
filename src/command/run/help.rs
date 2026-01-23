pub(super) fn help() {
    println!("Usage: ccb run [options] <file_name> [build_profile] [-- <program_args>]");
    println!();
    println!("Options:");
    println!("  -h, --help, -help          Show this help message and exit");
    println!("  -i, --interactive          Run in interactive mode (not implemented yet)");
    println!("  -std <version>             Specify the C++ standard version (default: 11)");
    println!(
        "  -in << <source>            Specify input source (file name, 'lastIn', or 'default')"
    );
    println!("  -out >> <destination>      Specify output destination (file name, 'lastOut', or 'default')");
    println!();
    println!("Arguments:");
    println!("  <file_name>                The source file to compile and run");
    println!("  [build_profile]            Optional build profile to use");
    println!("  [-- <program_args>]        Arguments to pass to the program being run");
}
