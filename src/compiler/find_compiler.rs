pub fn find_compiler() -> Option<String> {
    // Try to find g++ in the system PATH
    if let Ok(output) = std::process::Command::new("g++").arg("--version").output() {
        if output.status.success() {
            return Some("g++".to_string());
        }
    }

    // If g++ is not found, try clang++
    if let Ok(output) = std::process::Command::new("clang++")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            return Some("clang++".to_string());
        }
    }

    // check at common locations for g++ and clang++
    let common_paths = if cfg!(target_os = "windows") {
        vec![
            "C:\\MinGW\\bin\\g++.exe",
            "C:\\MinGW\\bin\\clang++.exe",
            "C:\\Program Files\\LLVM\\bin\\clang++.exe",
        ]
    } else {
        vec![
            "/usr/bin/g++",
            "/usr/bin/clang++",
            "/usr/local/bin/g++",
            "/usr/local/bin/clang++",
        ]
    };

    for path in common_paths {
        if let Ok(output) = std::process::Command::new(path).arg("--version").output() {
            if output.status.success() {
                return Some(path.to_string());
            }
        }
    }
    // If neither compiler is found, return None
    None
}

#[test]
pub fn test_find_compiler() {
    let compiler = find_compiler();
    assert!(compiler.is_some(), "No C++ compiler found on the system.");
    println!("Found C++ compiler: {}", compiler.unwrap());
}
