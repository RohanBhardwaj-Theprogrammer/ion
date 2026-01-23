use std::fs;

pub fn get_header(header_file_path: &str) -> Vec<String> {
    let headers = read_file_headers_quote(header_file_path);

    if headers.is_ok() {
        return headers.unwrap();
    }
    println!(" Failed to read headers from file : {} ", header_file_path);
    return Vec::new();
}

pub fn read_file_headers_quote(file_path: &str) -> Result<Vec<String>, String> {
    // let mut  max_include_miss :u128 = u128::MAX  ;
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut headers = Vec::new();
    for line in content.lines() {
        if line.trim_start().starts_with("#include") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let header = &line[start + 1..start + 1 + end];
                    headers.push(header.to_string());
                }
            } else if let Some(start) = line.find('<') {
                if let Some(end) = line[start + 1..].find('>') {
                    let header = &line[start + 1..start + 1 + end];
                    headers.push(header.to_string());
                }
            }
        }
        // else {
        //     // stop parsing after certain lines to avoid unnecessary processing
        //     if  max_include_miss == 0 {
        //         break;
        //     }
        //     max_include_miss -= 1;
        // }
    }

    return Ok(headers);
}

fn read_headers_from(file_path: &str) -> Result<(Vec<String>, Vec<String>), String> {
    if std::path::Path::new(file_path).is_dir() {
        return Err(" Provided path is a directory , expected a file path ".to_string());
    }

    let mut quote_headers = Vec::new();
    let mut angle_bracket_headers = Vec::new();
    let content = fs::read_to_string(file_path).unwrap_or("".to_string());

    for line in content.lines() {
        if line.trim_start().starts_with("#include") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let header = &line[start + 1..start + 1 + end];
                    quote_headers.push(header.to_string());
                }
            } else if let Some(start) = line.find('<') {
                if let Some(end) = line[start + 1..].find('>') {
                    let header = &line[start + 1..start + 1 + end];
                    angle_bracket_headers.push(header.to_string());
                }
            }
        }
    }

    return Ok((quote_headers, angle_bracket_headers));
}

//FIXME: to be improved with more headers and O(1) lookup and std_headers creations logic to be more robusts and static memory
pub fn is_std_header(header_name: &str) -> bool {
    //FIXME: to be read from a static file or a better way
    let std_headers: std::collections::HashSet<&str> = [
        "iostream",
        "vector",
        "string",
        "map",
        "set",
        "algorithm",
        "memory",
        "thread",
        "mutex",
        "functional",
        "utility",
        "cmath",
        "cstdio",
        "cstdlib",
        "cstring",
        "cassert",
        "fstream",
        "sstream",
        "iomanip",
        "limits",
        "typeinfo",
        "exception",
        "stdexcept",
        "cstddef",
        "cstdint",
        "array",
        "bitset",
        "deque",
        "forward_list",
        "list",
        "queue",
        "stack",
        "unordered_map",
        "unordered_set",
        "tuple",
        "chrono",
        "random",
        "regex",
        "condition_variable",
        "atomic",
        "future",
        "initializer_list",
        "new",
        "scoped_allocator",
        "system_error",
        "valarray",
        "cwchar",
        "cwctype",
        "locale",
        "codecvt",
        "clocale",
        "cctype",
        "cstdlib",
        "cfloat",
        "ciso646",
        "ccomplex",
        "ctgmath",
        "cuchar",
        "cfenv",
        "cinttypes",
        "cstdbool",
        "cstddef",
        "cstdint",
        "cstdalign",
        "cstdarg",
        "cstdbool",
        "ctime",
    ]
    .into_iter()
    .collect();

    for std_header in std_headers {
        if header_name == std_header || header_name == format!("{}.h", std_header) {
            return true;
        }
    }

    return false;
}

/// Check if a given file contains the main function
/// # Arguments
/// * `file_path` - A reference to a Path that holds the path of the file to be checked
/// # Returns
/// * `bool` - Returns true if the file contains the main function, false otherwise
//FIXME: improve to handle comments and different main function signatures
pub fn is_main_file(file_path: &std::path::Path) -> bool {
    use std::fs;

    let content = fs::read_to_string(file_path);
    if content.is_err() {
        return false;
    }

    let content = content.unwrap();
    let mut skip_comments = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        if skip_comments {
            if line.trim_end().ends_with("*/") {
                skip_comments = false;
            }
            continue;
        }

        if line.contains("int main(") || line.contains("int main ") {
            return true;
        }
    }

    false
}
