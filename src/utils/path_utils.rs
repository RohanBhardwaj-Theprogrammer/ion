/// Cross-platform path utilities for normalized path handling.
use std::path::{Path, MAIN_SEPARATOR};

/// Normalize path separators for display (converts backslashes to forward slashes for canonical form)
pub fn canonicalize_path_separators(s: &str) -> String {
    s.replace('\\', "/")
}

/// Build path string from components using current platform's separator
pub fn build_path_string(components: &[&str]) -> String {
    components.join(&MAIN_SEPARATOR.to_string())
}

/// Display path - removes Windows UNC prefixes for readability
/// Handles Windows extended-length paths (\\?\) and UNC paths (\\?\UNC\)
pub fn normalize_path_for_display(path: &Path) -> String {
    let s = path.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{}", rest);
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return rest.to_string();
        }
    }
    s.to_string()
}

/// Check if a path starts with a given prefix, handling both "/" and "\" separators
pub fn starts_with_any_sep(path: &str, prefix: &str) -> bool {
    // Try with forward slash
    if path.starts_with(&format!("{}/", prefix.trim_end_matches('/'))) {
        return true;
    }
    // Try with backslash
    if path.starts_with(&format!(
        "{}\\",
        prefix.trim_end_matches('\\').replace("/", "\\")
    )) {
        return true;
    }
    false
}

/// Check if a path starts with a dot relative prefix ("./" or ".\\")
pub fn starts_with_dot_relative(path: &str) -> bool {
    path.starts_with("./") || path.starts_with(".\\")
}

/// Convert path string to use platform-appropriate separators
#[cfg(target_os = "windows")]
pub fn to_platform_path(path: &str) -> String {
    path.replace('/', "\\")
}

/// Convert path string to use platform-appropriate separators (forward slash on Unix)
#[cfg(not(target_os = "windows"))]
pub fn to_platform_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_path_separators() {
        assert_eq!(
            canonicalize_path_separators("src\\main.cpp"),
            "src/main.cpp"
        );
        assert_eq!(canonicalize_path_separators("src/main.cpp"), "src/main.cpp");
        assert_eq!(
            canonicalize_path_separators("tests\\test_project\\input.txt"),
            "tests/test_project/input.txt"
        );
    }

    #[test]
    fn test_starts_with_dot_relative() {
        assert!(starts_with_dot_relative("./src"));
        assert!(starts_with_dot_relative(".\\src"));
        assert!(!starts_with_dot_relative("src"));
        assert!(!starts_with_dot_relative("../src"));
    }

    #[test]
    fn test_build_path_string() {
        let result = build_path_string(&["src", "main", "cpp"]);
        // Result depends on platform, but should contain all components
        assert!(result.contains("src"));
        assert!(result.contains("main"));
        assert!(result.contains("cpp"));
    }
}
