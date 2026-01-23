use chrono::Local;
use std::fs;
use std::path::Path;

pub fn is_cbuild_project(path: &str) -> bool {
    // let project_path = format!("{}/.{}", path, crate::constants::PROGRAM_NAME); skipped  in favor of Path operations
    let project_path = Path::new(path).join(crate::constants::PROGRAM_NAME);
    project_path.exists() && project_path.is_dir()
}

pub fn truncate(path: &str) -> Result<bool, std::io::Error> {
    if !Path::new(path).exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?.path();

        if entry.is_dir() {
            fs::remove_dir_all(entry)?;
        } else if entry.is_file() {
            fs::remove_file(entry)?;
        }
    }

    Ok(true)
}

/// Trims matching quotes from the start and end of a string.
/// If the string does not start and end with matching quotes, it is returned unchanged.
/// # Examples
/// ```rust
/// use c_cpp_build_system_n_pkg_manager::utils::trim_quotes;
///
/// let s = r#""Hello, World!""#;
/// let trimmed = trim_quotes(s);
/// assert_eq!(trimmed, "Hello, World!");
/// ```
pub fn trim_quotes(s: &str) -> String {
    let mut chars = s.chars();
    if let (Some(first), Some(last)) = (chars.next(), chars.next_back()) {
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return chars.as_str().to_string();
        }
    }
    s.to_string()
}

pub fn quotes(string: String) -> Result<String, String> {
    let mut trimmed = string.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed = &trimmed[1..trimmed.len() - 1];
    } else if trimmed.starts_with('"')
        || trimmed.ends_with('"')
        || trimmed.starts_with('\'')
        || trimmed.ends_with('\'')
    {
        return Err(format!("Malformed quoted string: {}", string));
    }

    Ok(trimmed.to_string())
}

pub fn input(key: &str) -> Option<String> {
    print!("{} >", key);
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

pub fn time_stamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

pub fn local_time() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn find_cbuild(start_path: &str) -> Option<String> {
    let mut current_path = Path::new(start_path).to_path_buf();

    while let Some(parent) = current_path.parent() {
        let dot_cbuild = current_path.join(format!(".{}", crate::constants::PROGRAM_NAME));
        if dot_cbuild.exists() {
            return current_path.to_str().map(|s| s.to_string());
        }
        current_path = parent.to_path_buf();
    }

    None
}

pub fn is_numeric(s: &str) -> bool {
    s.chars().all(|e| e >= '0' && e <= '9')
}

// FIXME: need the configs here to resolve relative paths correctly
pub fn parse_path_arg(arg: &str) -> String {
    use std::path::PathBuf;

    // empty or "." means current directory
    if arg.is_empty() || arg == "." {
        return std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .into_owned();
    }

    // handle any number of "../" prefixes (../, ../../, ../../../, etc.)
    if arg.starts_with("../") || arg == ".." {
        let mut cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut remainder = arg;

        // Count and apply each "../"
        while remainder.starts_with("../") {
            // Move cwd up one level if possible
            cwd = cwd.parent().unwrap_or(&cwd).to_path_buf();

            // Strip one "../"
            remainder = &remainder[3..];
        }

        // Handle the case where the whole string is exactly ".."
        if remainder == ".." {
            cwd = cwd.parent().unwrap_or(&cwd).to_path_buf();
            remainder = "";
        }

        // If there's leftover path (e.g. "path/file")
        if !remainder.is_empty() {
            cwd = cwd.join(remainder);
        }

        return cwd.to_string_lossy().into_owned();
    }

    // default: return as-is
    arg.to_string()
}

//__________________________________TEST____________________________________

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_cbuild() {
        let test_path = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let dot_ion = format!("{}/.{}", test_path, crate::constants::PROGRAM_NAME);
        std::fs::create_dir(&dot_ion).ok();

        let result = find_cbuild(&test_path);
        assert!(result.is_some());

        std::fs::remove_dir(&dot_ion).ok();
    }

    #[test]
    fn test_timestamp() {
        let ts = time_stamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_local_time() {
        let lt = local_time();
        assert!(lt.len() > 0);
    }

    #[test]
    fn test_truncate_nonexistent_path() {
        let result = truncate("non_existent_path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}
