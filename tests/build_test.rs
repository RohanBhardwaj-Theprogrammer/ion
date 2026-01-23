use ion::compiler::includes::IncludeFiles;
use ion::compiler::ToCompilerArgs;
use ion::config::Configs;
use std::path::PathBuf;

#[test]
fn test_include_files_for_test_project() {
    // Print current directory for debugging
    println!("Current dir: {:?}", std::env::current_dir().unwrap());
    // Set up config for the test_project directory
    let root_path = "./tests/test_project";
    let main_file = format!("{}/main.cpp", root_path);
    let configs = Configs::default(PathBuf::from(root_path));
    let project_structure = ion::state::ProjectStructure::new(&configs);
    let deps = ion::deps::DependencyGraph::new(&main_file, &project_structure);
    let include_files = IncludeFiles::new(&deps, &configs, &project_structure, &main_file);

    // Check main file path
    assert!(
        include_files.main_file.contains("main.cpp"),
        "main_file should be main.cpp"
    );

    // Check that all expected header search paths are present
    assert!(
        !include_files.header_files.is_empty(),
        "Should find header search paths"
    );

    // Check that all expected source files are present
    let expected_sources = [
        "main.cpp",
        "some_code.cpp",
        "util.cpp",
        "math_utils.cpp",
        "logger.cpp",
    ];
    for src in &expected_sources {
        let found = include_files.source_files.iter().any(|f| f.contains(src));
        assert!(found, "Source file {} should be found", src);
    }

    // Check that to_args produces -I and source args
    let args = include_files.to_args();
    // Check that at least one arg starts with -I
    assert!(
        args.iter().any(|a| a.trim_start().starts_with("-I")),
        "Args should contain -I for include dirs"
    );
    for src in &expected_sources {
        assert!(
            args.iter().any(|a| a.contains(src)),
            "Args should contain {}",
            src
        );
    }
}

use ion::compiler::Compiler;

#[test]
fn test_compiler_compile_for_test_project() {
    use std::path::Path;
    use std::process::Command;

    // Skip this test if a C++ compiler isn't available on PATH.
    if Command::new("g++").arg("--version").output().is_err() {
        eprintln!("Skipping compiler integration test: g++ not found on PATH");
        return;
    }

    let root_path = "./tests/test_project";
    let main_file = format!("{}/main.cpp", root_path);
    let configs = Configs::default(PathBuf::from(root_path));
    let mut project_structure = ion::state::ProjectStructure::new(&configs);
    let deps = ion::deps::DependencyGraph::new(&main_file, &project_structure);
    let include_files = ion::compiler::includes::IncludeFiles::new(
        &deps,
        &configs,
        &mut project_structure,
        &main_file,
    );
    let build_settings = ion::compiler::build_settings::BuildSettings::fast();
    let compiler = Compiler::new(
        &main_file,
        include_files,
        build_settings,
        &configs,
        &mut project_structure,
    );
    let result = compiler.compile();
    let exe_path = &compiler.build_name;
    match result {
        Ok(output) => {
            println!("Compiler output: {}", output);
            // Check that the .exe file exists
            assert!(
                Path::new(exe_path).exists(),
                "Executable should be created at {}",
                exe_path
            );
            // Try to run the .exe file (Windows)
            let run_result = Command::new(exe_path).output();
            match run_result {
                Ok(run_output) => {
                    if !run_output.status.success() {
                        println!("\n\n=== EXECUTABLE RUNTIME OUTPUT ===");
                        println!("Exit code: {:?}", run_output.status.code());
                        println!("Stdout:\n{}", String::from_utf8_lossy(&run_output.stdout));
                        println!("Stderr:\n{}", String::from_utf8_lossy(&run_output.stderr));
                        panic!("Executable did not run successfully");
                    }
                }
                Err(e) => panic!("Failed to run the executable: {}", e),
            }
        }
        Err(e) => {
            // Print the error and try to print the compiler's stdout/stderr if available
            println!("\n\n=== COMPILER ERROR OUTPUT ===\n{}\n", e);
            panic!("Compiler error: {}", e)
        }
    }
}
