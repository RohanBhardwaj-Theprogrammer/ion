use std::process::Command;
use std::io;

use super::file_ops::{file_exists, bin_name, binary_exists, remove_binary};
use super::list_file::single_file_selection;
use crate::cmd_parser::CmdType;

macro_rules! compiler {
    () => { "g++" };
}

fn cstd_version_flag() -> &'static str {
    "-std=c++20"
}

/// Return the compiler optimization flag for a given shorthand
fn optimization_flag(opt_level: &str) -> &'static str {
    match opt_level.trim().to_ascii_lowercase().as_str() {
        "" | "none" | "0" => "-O0",
        "reg" | "1" | "o1" => "-O1",
        "opt" | "2" | "o2" => "-O2",
        "release" | "3" | "o3" => "-O3",
        "s" | "size" => "-Os",
        "z" | "oz" => "-Oz",
        _ => "-O0",
    }
}

/// Compile a single file into a binary
pub fn build_internal_cmd(file: &str, optimization_lvl: &str, force: bool) -> io::Result<String> {
    let bin_path = bin_name(file);

    if binary_exists(&bin_path) && force {
        remove_binary(file);
    }

    let output = Command::new(compiler!())
        .arg(cstd_version_flag())
        .arg(file)
        .arg(optimization_lvl)
        .arg("-o")
        .arg(&bin_path)
        .output()?;

    if output.status.success() {
        Ok(bin_path)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::new(io::ErrorKind::Other, format!("Build failed: {}", error_msg)))
    }
}

pub fn build_cmd(root_dir: &str, args: &CmdType) -> io::Result<String> {
    match args {
        CmdType::Build(file_path, opt_lvl, force) => {
            let build_cmd_type = vec![file_path.clone(), opt_lvl.clone(), force.to_string()];
            build_cmd_internal(&build_cmd_type, root_dir)
        },
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid command type for build_cmd"))
    }
}

/// `build_cmd_type` is expected to be `[source_file, optimization_flag, force_flag]`
fn build_cmd_internal(build_cmd_type: &[String], root_dir: &str) -> io::Result<String> {
    if build_cmd_type.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No build arguments provided"));
    }

    // Determine the source file
    let file = if build_cmd_type[0].is_empty() || !file_exists(&build_cmd_type[0]) {
        single_file_selection(root_dir, vec![".c", ".cpp"])?
    } else {
        build_cmd_type[0].clone()
    };

    if !file_exists(&file) {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Source file does not exist"));
    }

    // Determine optimization level
    let optimization_lvl = if build_cmd_type.len() > 1 {
        optimization_flag(&build_cmd_type[1]).to_string()
    } else {
        optimization_flag("").to_string()
    };

    // Determine force flag
    let force = if build_cmd_type.len() > 2 {
        build_cmd_type[2] == "true"
    } else {
        false
    };

    build_internal_cmd(&file, &optimization_lvl, force)
}
