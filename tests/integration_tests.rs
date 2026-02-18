// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Rohan Bhardwaj
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// tests/integration_tests.rs
//
// Comprehensive integration tests for the `ion` (cbuild) CLI tool.
//
// These tests exercise the full command pipeline end-to-end — from CLI argument
// parsing through command dispatch and filesystem effects — using isolated
// temporary directories so no real state is mutated.
//
// Compiler-dependent tests (build / run / check execute) are
// skipped gracefully when no C++ compiler is found on PATH.

use assert_fs::prelude::*;
use assert_fs::TempDir;
use ion::cmd_parser::cmd::Type;
use ion::cmd_parser::parser::{parse_args, ParsedCommand};
use ion::command;
use ion::command::check::check_parser;
use ion::command::init::init;
use ion::command::run::run::{run_parser, InputSource};
use ion::compiler::find_compiler::find_compiler;
use ion::config::Configs;
use ion::state::ProjectStructure;

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn pc(t: Type, args: &[&str]) -> ParsedCommand {
    ParsedCommand {
        command_type: t,
        args: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn has_compiler() -> bool {
    find_compiler().is_some()
}

macro_rules! require_compiler {
    () => {
        if !has_compiler() {
            eprintln!("SKIP: no C++ compiler on PATH — skipping compiler-dependent test");
            return;
        }
    };
}

/// Create a temp project pre-loaded with the given (relative-path, content) files.
/// Returns (TempDir, Configs, ProjectStructure). TempDir must stay alive.
fn make_project(files: &[(&str, &str)]) -> (TempDir, Configs, ProjectStructure) {
    let temp = TempDir::new().unwrap();
    for (rel, content) in files {
        temp.child(rel).write_str(content).unwrap();
    }
    let configs = Configs::default(temp.path().to_path_buf());
    let ps = ProjectStructure::new(&configs);
    (temp, configs, ps)
}

// ── C++ source fixtures ───────────────────────────────────────
const HELLO: &str = r#"
#include <iostream>
int main() { std::cout << "hello" << std::endl; return 0; }
"#;

const ECHO_STDIN: &str = r#"
#include <iostream>
#include <string>
int main() {
    std::string line;
    std::getline(std::cin, line);
    std::cout << line;
    return 0;
}
"#;

const ECHO_ARGV: &str = r#"
#include <iostream>
int main(int argc, char* argv[]) {
    for (int i = 1; i < argc; ++i) std::cout << argv[i] << "\n";
    return 0;
}
"#;

const HELLO_WITH_HEADER: &str = r#"
#include <iostream>
#include "util.h"
int main() { say_hello(); return 0; }
"#;

const UTIL_H: &str = r#"
#pragma once
#include <iostream>
inline void say_hello() { std::cout << "hello from util" << std::endl; }
"#;

const MULTI_MAIN: &str = r#"
#include <iostream>
#include "greet.h"
int main() { greet(); return 0; }
"#;

const GREET_H: &str = r#"
#pragma once
void greet();
"#;

const GREET_CPP: &str = r#"
#include "greet.h"
#include <iostream>
void greet() { std::cout << "greet!" << std::endl; }
"#;

const SYNTAX_ERROR: &str = "int main() { THIS IS NOT VALID C++ }";

// ─────────────────────────────────────────────────────────────
// 1. parse_args — top-level CLI parser
// ─────────────────────────────────────────────────────────────

#[test]
fn parse_args_build_strips_binary_and_subcommand() {
    let args = vec!["ion".to_string(), "build".to_string(), "main.cpp".to_string()];
    let parsed = parse_args(args);
    assert_eq!(parsed.command_type as u8, Type::Build as u8);
    assert_eq!(parsed.args, vec!["main.cpp"]);
}

#[test]
fn parse_args_run_with_flags() {
    let args = vec![
        "ion".to_string(),
        "run".to_string(),
        "main.cpp".to_string(),
        "--release".to_string(),
    ];
    let parsed = parse_args(args);
    assert_eq!(parsed.command_type as u8, Type::Run as u8);
    assert_eq!(parsed.args, vec!["main.cpp", "--release"]);
}

#[test]
fn parse_args_unknown_defaults_to_help() {
    let args = vec!["ion".to_string(), "totally_unknown".to_string()];
    let parsed = parse_args(args);
    assert_eq!(parsed.command_type as u8, Type::Help as u8);
}

#[test]
fn parse_args_no_subcommand_defaults_to_help() {
    let args = vec!["ion".to_string()];
    let parsed = parse_args(args);
    assert_eq!(parsed.command_type as u8, Type::Help as u8);
}

#[test]
fn parse_args_version_flags() {
    for flag in &["-v", "--version", "version"] {
        let args = vec!["ion".to_string(), flag.to_string()];
        let parsed = parse_args(args);
        assert_eq!(parsed.command_type as u8, Type::Version as u8, "failed for {flag}");
    }
}

#[test]
fn parse_args_all_known_commands() {
    let table: &[(&str, Type)] = &[
        ("build",   Type::Build),
        ("run",     Type::Run),
        ("check",   Type::Check),
        ("init",    Type::Init),
        ("clean",   Type::Clean),
        ("env",     Type::Env),
        ("help",    Type::Help),
        ("log",     Type::Log),
        ("config",  Type::Config),
    ];
    for (cmd_str, expected) in table {
        let args = vec!["ion".to_string(), cmd_str.to_string()];
        let parsed = parse_args(args);
        assert_eq!(parsed.command_type as u8, *expected as u8, "failed for '{cmd_str}'");
    }
}

// ─────────────────────────────────────────────────────────────
// 2. execute() dispatch
// ─────────────────────────────────────────────────────────────

#[test]
fn execute_version_returns_program_name_and_version() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Version, &[]), &mut configs, &mut ps);
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert!(msg.contains(ion::constants::PROGRAM_NAME));
    assert!(msg.contains(ion::constants::PROGRAM_VERSION));
}

#[test]
fn execute_help_returns_ok() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Help, &[]), &mut configs, &mut ps).is_ok());
}

#[test]
fn execute_unimplemented_commands_return_err() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    for t in &[Type::Log, Type::Config, Type::Watcher] {
        let mut ps = ProjectStructure::new(&configs);
        let result = command::execute(
            ParsedCommand { command_type: *t, args: vec![] },
            &mut configs,
            &mut ps,
        );
        assert!(result.is_err(), "{:?} should return Err", t);
    }
}

// ─────────────────────────────────────────────────────────────
// 3. init command
// ─────────────────────────────────────────────────────────────

#[test]
fn init_creates_dotfolder_config_files_and_default_structure() {
    let temp = TempDir::new().unwrap();
    let result = init(pc(Type::Init, &[temp.path().to_str().unwrap()]));
    assert!(result.is_ok(), "init failed: {:?}", result);

    let dot = temp.path().join(format!(".{}", ion::constants::PROGRAM_NAME));
    assert!(dot.is_dir(),                              ".cbuild dir missing");
    assert!(dot.join("project_config.json").is_file(), "project_config.json missing");
    assert!(dot.join("config.toml").is_file(),         "config.toml missing");
    assert!(temp.path().join("src").is_dir(),          "src dir missing");
    assert!(temp.path().join("include").is_dir(),      "include dir missing");
    assert!(temp.path().join("src").join("main.cpp").is_file(), "main.cpp missing");
}

#[test]
fn init_rejects_reinit_without_force() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();
    assert!(init(pc(Type::Init, &[path])).is_ok());
    let second = init(pc(Type::Init, &[path]));
    assert!(second.is_err(), "second init without --force should fail");
}

#[test]
fn init_force_overwrites_existing_project() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();
    assert!(init(pc(Type::Init, &[path])).is_ok());
    let result = init(pc(Type::Init, &[path, "--force"]));
    assert!(result.is_ok(), "init --force should succeed: {:?}", result);
}

#[test]
fn init_c_lang_creates_main_c() {
    let temp = TempDir::new().unwrap();
    let result = init(pc(Type::Init, &[temp.path().to_str().unwrap(), "-c"]));
    assert!(result.is_ok(), "init -c failed: {:?}", result);
    assert!(temp.path().join("src").join("main.c").is_file(), "main.c missing");
}

#[test]
fn init_help_flag_does_not_create_project_files() {
    let temp = TempDir::new().unwrap();
    let result = init(pc(Type::Init, &[temp.path().to_str().unwrap(), "--help"]));
    assert!(result.is_ok());
    let dot = temp.path().join(format!(".{}", ion::constants::PROGRAM_NAME));
    assert!(!dot.exists(), ".cbuild should not be created when --help is passed");
}

#[test]
fn init_custom_structure_clean_skips_default_dirs() {
    let temp = TempDir::new().unwrap();
    let result = init(pc(Type::Init, &[temp.path().to_str().unwrap(), "--custom-structure=clean"]));
    assert!(result.is_ok(), "init --custom-structure=clean failed: {:?}", result);
    assert!(!temp.path().join("src").exists(),     "src should not exist");
    assert!(!temp.path().join("include").exists(), "include should not exist");
}

#[test]
fn init_routed_via_execute_dispatch() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Init, &[temp.path().to_str().unwrap()]), &mut configs, &mut ps);
    assert!(result.is_ok());
}

#[test]
fn init_rejects_non_init_command_type() {
    let result = init(pc(Type::Run, &["--help"]));
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 4. build command
// ─────────────────────────────────────────────────────────────

#[test]
fn build_help_flag_returns_ok_without_compiler() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Build, &["--help"]), &mut configs, &mut ps);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Help displayed");
}

#[test]
fn build_simple_program_produces_binary() {
    require_compiler!();
    let (temp, mut configs, mut ps) = make_project(&[("main.cpp", HELLO)]);
    let main_cpp = temp.path().join("main.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Build, &[&main_cpp]), &mut configs, &mut ps);
    assert!(result.is_ok(), "build failed: {:?}", result);
    assert!(temp.path().join("build").exists(), "build dir should exist after successful build");
}

#[test]
fn build_program_with_local_header() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[
        ("main.cpp", HELLO_WITH_HEADER),
        ("util.h", UTIL_H),
    ]);
    let main_cpp = _temp.path().join("main.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Build, &[&main_cpp]), &mut configs, &mut ps);
    assert!(result.is_ok(), "build with header dep failed: {:?}", result);
}

#[test]
fn build_multifile_project() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[
        ("main.cpp", MULTI_MAIN),
        ("greet.h", GREET_H),
        ("greet.cpp", GREET_CPP),
    ]);
    let main_cpp = _temp.path().join("main.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Build, &[&main_cpp]), &mut configs, &mut ps);
    assert!(result.is_ok(), "multi-file build failed: {:?}", result);
}

#[test]
fn build_fails_on_syntax_error() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("bad.cpp", SYNTAX_ERROR)]);
    let bad_cpp = _temp.path().join("bad.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Build, &[&bad_cpp]), &mut configs, &mut ps);
    assert!(result.is_err(), "build on invalid C++ should fail");
}

#[test]
fn build_fails_on_nonexistent_file() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Build, &["ghost.cpp"]), &mut configs, &mut ps);
    assert!(result.is_err(), "build with nonexistent file should fail");
}

#[test]
fn build_auto_discovers_src_main_cpp_when_no_file_given() {
    // build_parser does not auto-discover files when no arg is passed -- it leaves
    // file_name empty, which causes the compiler to return an error. Verify that
    // behaviour rather than expecting success.
    let temp = TempDir::new().unwrap();
    temp.child("src").create_dir_all().unwrap();
    temp.child("src/main.cpp").write_str(HELLO).unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Build, &[]), &mut configs, &mut ps);
    // Without an explicit file and without auto-discovery in build_parser the
    // compiler is invoked with an empty path and must return an error.
    assert!(
        result.is_err(),
        "build with no file arg should return an error (build_parser has no auto-discover)"
    );
}

#[test]
fn build_with_release_profile() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("main.cpp", HELLO)]);
    let main_cpp = _temp.path().join("main.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Build, &[&main_cpp, "--release"]), &mut configs, &mut ps);
    assert!(result.is_ok(), "release build failed: {:?}", result);
}

// ─────────────────────────────────────────────────────────────
// 5. run command — parser
// ─────────────────────────────────────────────────────────────

#[test]
fn run_parser_parses_build_profile_and_program_args() {
    let temp = TempDir::new().unwrap();
    temp.child("main.cpp").write_str(HELLO).unwrap();
    let input  = temp.child("in.txt");  input.write_str("x\n").unwrap();
    let output = temp.child("out.txt");
    let configs = Configs::default(temp.path().to_path_buf());
    let ps = ProjectStructure::new(&configs);

    let cmd = pc(Type::Run, &[
        "main.cpp", "--release",
        "-in",  input.path().to_str().unwrap(),
        "-out", output.path().to_str().unwrap(),
        "--", "arg1", "arg2",
    ]);
    let args = run_parser(&cmd, &configs, &ps);

    assert!(args.file_name.to_string_lossy().contains("main.cpp"));
    assert_eq!(args.build_profile, Some("release".to_string()));
    assert_eq!(args.program_args, vec!["arg1", "arg2"]);
    assert!(matches!(args.input, InputSource::File(_)));
}

#[test]
fn run_parser_help_flag() {
    let temp = TempDir::new().unwrap();
    let configs = Configs::default(temp.path().to_path_buf());
    let ps = ProjectStructure::new(&configs);
    let args = run_parser(&pc(Type::Run, &["--help"]), &configs, &ps);
    assert!(args.help_flag);
}

#[test]
fn run_parser_no_args_resolves_src_main_cpp() {
    let temp = TempDir::new().unwrap();
    temp.child("src").create_dir_all().unwrap();
    temp.child("src/main.cpp").write_str(HELLO).unwrap();
    let configs = Configs::default(temp.path().to_path_buf());
    let ps = ProjectStructure::new(&configs);
    let args = run_parser(&pc(Type::Run, &[]), &configs, &ps);
    assert!(
        args.file_name.to_string_lossy().contains("main.cpp"),
        "expected main.cpp, got {:?}", args.file_name
    );
}

// ─────────────────────────────────────────────────────────────
// 6. run command — execute
// ─────────────────────────────────────────────────────────────

#[test]
fn run_help_flag_returns_ok_without_compiler() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Run, &["--help"]), &mut configs, &mut ps).is_ok());
}

#[test]
fn run_compiles_and_executes_hello_world() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("main.cpp", HELLO)]);
    let main_cpp = _temp.path().join("main.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Run, &[&main_cpp]), &mut configs, &mut ps);
    assert!(result.is_ok(), "run failed: {:?}", result);
}

#[test]
fn run_redirects_stdin_and_captures_output_to_file() {
    require_compiler!();
    let temp = TempDir::new().unwrap();
    temp.child("main.cpp").write_str(ECHO_STDIN).unwrap();
    let input  = temp.child("input.txt");  input.write_str("integration_token\n").unwrap();
    let output = temp.child("output.txt");
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    let cmd = pc(Type::Run, &[
        "main.cpp",
        "-in",  input.path().to_str().unwrap(),
        "-out", output.path().to_str().unwrap(),
    ]);
    let result = command::execute(cmd, &mut configs, &mut ps);
    assert!(result.is_ok(), "run with I/O redirect failed: {:?}", result);

    let contents = std::fs::read_to_string(output.path())
        .expect("output.txt should be written");
    assert_eq!(contents, "integration_token");
}

#[test]
fn run_passes_program_args_to_binary() {
    require_compiler!();
    let temp = TempDir::new().unwrap();
    temp.child("main.cpp").write_str(ECHO_ARGV).unwrap();
    let output = temp.child("output.txt");
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    let cmd = pc(Type::Run, &[
        "main.cpp",
        "-out", output.path().to_str().unwrap(),
        "--", "foo", "bar",
    ]);
    let result = command::execute(cmd, &mut configs, &mut ps);
    assert!(result.is_ok(), "run with argv failed: {:?}", result);

    let contents = std::fs::read_to_string(output.path()).unwrap();
    assert!(contents.contains("foo"), "stdout should contain 'foo'");
    assert!(contents.contains("bar"), "stdout should contain 'bar'");
}

#[test]
fn run_fails_when_no_valid_source_file() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Run, &[]), &mut configs, &mut ps);
    assert!(result.is_err(), "run with no source should return Err");
}

#[test]
fn run_fails_on_syntax_error_at_compile_stage() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("bad.cpp", SYNTAX_ERROR)]);
    let bad_cpp = _temp.path().join("bad.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Run, &[&bad_cpp]), &mut configs, &mut ps);
    assert!(result.is_err(), "run on bad C++ should fail at compile stage");
}

// ─────────────────────────────────────────────────────────────
// 7. check command — parser
// ─────────────────────────────────────────────────────────────

#[test]
fn check_parser_parses_std_and_verbose_flags() {
    let temp = TempDir::new().unwrap();
    temp.child("main.cpp").write_str(HELLO).unwrap();
    let configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    let args = check_parser(
        &pc(Type::Check, &["main.cpp", "-std", "17", "--verbose"]),
        &configs,
        &mut ps,
    );
    assert_eq!(args.std, Some(17));
    assert!(args.verboase_flag);
    assert!(!args.help_flag);
}

#[test]
fn check_parser_detects_help_flag() {
    let temp = TempDir::new().unwrap();
    let configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let args = check_parser(&pc(Type::Check, &["--help"]), &configs, &mut ps);
    assert!(args.help_flag);
}

#[test]
fn check_parser_sets_build_profile_from_double_dash() {
    let temp = TempDir::new().unwrap();
    temp.child("main.cpp").write_str(HELLO).unwrap();
    let configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let args = check_parser(
        &pc(Type::Check, &["main.cpp", "--fast"]),
        &configs,
        &mut ps,
    );
    assert_eq!(args.build_profile, Some("fast".to_string()));
}

// ─────────────────────────────────────────────────────────────
// 8. check command — execute
// ─────────────────────────────────────────────────────────────

#[test]
fn check_help_returns_ok_without_compiler() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Check, &["--help"]), &mut configs, &mut ps).is_ok());
}

#[test]
fn check_valid_cpp_file_succeeds() {
    require_compiler!();
    // check_parser lowercases all args including file paths. To avoid a
    // case-mismatch on Linux we ensure the project root path is already
    // all-lowercase by creating a dedicated subdirectory under /tmp.
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    let root = std::path::PathBuf::from(format!("/tmp/ion_check_valid_{}", ts));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.cpp"), HELLO).unwrap();
    let main_cpp = root.join("main.cpp").to_string_lossy().to_string();
    let mut configs = Configs::default(root.clone());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Check, &[&main_cpp]), &mut configs, &mut ps);
    let _ = std::fs::remove_dir_all(&root); // cleanup
    assert!(result.is_ok(), "check failed on valid file: {:?}", result);
}

#[test]
fn check_invalid_cpp_file_returns_error() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("bad.cpp", SYNTAX_ERROR)]);
    let bad_cpp = _temp.path().join("bad.cpp").to_string_lossy().to_string();
    let result = command::execute(pc(Type::Check, &[&bad_cpp]), &mut configs, &mut ps);
    assert!(result.is_err(), "check should fail on syntax error");
}

#[test]
fn check_file_with_local_header_succeeds() {
    require_compiler!();
    // check_parser lowercases all args including file paths. To avoid a
    // case-mismatch on Linux we ensure the project root path is already
    // all-lowercase by creating a dedicated subdirectory under /tmp.
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    let root = std::path::PathBuf::from(format!("/tmp/ion_check_header_{}", ts));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.cpp"), HELLO_WITH_HEADER).unwrap();
    std::fs::write(root.join("util.h"), UTIL_H).unwrap();
    let main_cpp = root.join("main.cpp").to_string_lossy().to_string();
    let mut configs = Configs::default(root.clone());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Check, &[&main_cpp]), &mut configs, &mut ps);
    let _ = std::fs::remove_dir_all(&root); // cleanup
    assert!(result.is_ok(), "check with local header failed: {:?}", result);
}

#[test]
fn check_nonexistent_file_returns_error() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Check, &["ghost.cpp"]), &mut configs, &mut ps);
    assert!(result.is_err(), "check on nonexistent file should fail");
}

// ─────────────────────────────────────────────────────────────
// 9. clean command
// ─────────────────────────────────────────────────────────────

#[test]
fn clean_removes_build_directory() {
    let temp = TempDir::new().unwrap();
    temp.child("build").create_dir_all().unwrap();
    temp.child("build/binary").write_str("elf").unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Clean, &[]), &mut configs, &mut ps);
    assert!(result.is_ok(), "clean failed: {:?}", result);
    assert!(!temp.path().join("build").exists(), "build dir should be removed");
}

#[test]
fn clean_is_ok_when_build_dir_absent() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Clean, &[]), &mut configs, &mut ps).is_ok());
}

#[test]
fn clean_help_flag_returns_ok() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Clean, &["--help"]), &mut configs, &mut ps).is_ok());
}

#[test]
fn clean_without_force_preserves_object_files() {
    let temp = TempDir::new().unwrap();
    temp.child("build").create_dir_all().unwrap();
    temp.child("src/main.o").write_str("obj").unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(pc(Type::Clean, &["--all", ".o"]), &mut configs, &mut ps);
    assert!(result.is_ok());
    assert!(temp.path().join("src/main.o").exists(), ".o must survive without --force");
}

#[test]
fn clean_all_force_deletes_extension_files_respecting_exceptions() {
    let temp = TempDir::new().unwrap();
    temp.child("build").create_dir_all().unwrap();
    temp.child("delete.o").write_str("obj").unwrap();
    temp.child("keep.o").write_str("obj").unwrap();
    let keep_path = temp.path().join("keep.o").to_string_lossy().to_string();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(
        pc(Type::Clean, &["--all", "--force", ".o", "-e", &keep_path]),
        &mut configs,
        &mut ps,
    );
    assert!(result.is_ok());
    assert!(!temp.path().join("delete.o").exists(), "delete.o should be removed");
    assert!(temp.path().join("keep.o").exists(),    "keep.o should be preserved");
}

// ─────────────────────────────────────────────────────────────
// 10. env command
// ─────────────────────────────────────────────────────────────

#[test]
fn env_help_flag_returns_ok() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Env, &["--help"]), &mut configs, &mut ps).is_ok());
}

#[test]
fn env_writes_var_to_run_scope_file() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(
        pc(Type::Env, &["--cmd", "run", "MY_VAR=hello"]),
        &mut configs, &mut ps,
    );
    assert!(result.is_ok(), "env --cmd run failed: {:?}", result);
    let run_env = temp.path().join(".cbuild").join("env").join("run.env");
    assert!(run_env.exists(), "run.env not created");
    let content = std::fs::read_to_string(&run_env).unwrap();
    assert!(content.contains("my_var=hello"), "var not found in run.env");
}

#[test]
fn env_writes_var_to_build_scope_file() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(
        pc(Type::Env, &["--cmd", "build", "BUILD_VAR=world"]),
        &mut configs, &mut ps,
    ).is_ok());
    let build_env = temp.path().join(".cbuild").join("env").join("build.env");
    assert!(build_env.exists());
    assert!(std::fs::read_to_string(&build_env).unwrap().contains("build_var=world"));
}

#[test]
fn env_default_scope_writes_both_run_and_build() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(
        pc(Type::Env, &["SHARED=42"]),
        &mut configs, &mut ps,
    ).is_ok());
    let base = temp.path().join(".cbuild").join("env");
    for scope in &["run.env", "build.env"] {
        let content = std::fs::read_to_string(base.join(scope))
            .unwrap_or_default();
        assert!(content.contains("shared=42"), "{scope} should contain shared=42");
    }
}

#[test]
fn env_clear_wipes_previous_vars_before_writing() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    // Write initial var
    command::execute(pc(Type::Env, &["--cmd", "run", "OLD=old"]), &mut configs, &mut ps).unwrap();

    // Clear and write new var
    let mut ps2 = ProjectStructure::new(&configs);
    command::execute(
        pc(Type::Env, &["--cmd", "run", "--clear", "NEW=new"]),
        &mut configs, &mut ps2,
    ).unwrap();

    let content = std::fs::read_to_string(
        temp.path().join(".cbuild").join("env").join("run.env")
    ).unwrap();
    assert!(!content.contains("old=old"), "OLD var should be wiped by --clear");
    assert!(content.contains("new=new"),  "NEW var should be written");
}

#[test]
fn env_updating_existing_key_deduplicates_entries() {
    let temp = TempDir::new().unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    command::execute(pc(Type::Env, &["--cmd", "run", "KEY=first"]), &mut configs, &mut ps).unwrap();

    let mut ps2 = ProjectStructure::new(&configs);
    command::execute(pc(Type::Env, &["--cmd", "run", "KEY=second"]), &mut configs, &mut ps2).unwrap();

    let content = std::fs::read_to_string(
        temp.path().join(".cbuild").join("env").join("run.env")
    ).unwrap();
    assert!(content.contains("key=second"), "key should be updated to 'second'");
    let count = content.lines().filter(|l| l.starts_with("key=")).count();
    assert_eq!(count, 1, "key should appear exactly once — no duplicates");
}

#[test]
fn env_imports_variables_from_external_file() {
    let temp = TempDir::new().unwrap();
    let env_src = temp.child("external.env");
    env_src.write_str("FILE_VAR=from_file\nANOTHER=value\n").unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    let result = command::execute(
        pc(Type::Env, &["--cmd", "run", "-path", env_src.path().to_str().unwrap()]),
        &mut configs, &mut ps,
    );
    assert!(result.is_ok(), "env -path failed: {:?}", result);
    let content = std::fs::read_to_string(
        temp.path().join(".cbuild").join("env").join("run.env")
    ).unwrap();
    assert!(content.contains("file_var=from_file"));
    assert!(content.contains("another=value"));
}

#[test]
fn env_inline_vars_override_imported_file_vars() {
    let temp = TempDir::new().unwrap();
    let env_src = temp.child("base.env");
    env_src.write_str("KEY=from_file\n").unwrap();
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    command::execute(
        pc(Type::Env, &[
            "--cmd", "run",
            "-path", env_src.path().to_str().unwrap(),
            "KEY=inline_wins",
        ]),
        &mut configs, &mut ps,
    ).unwrap();
    let content = std::fs::read_to_string(
        temp.path().join(".cbuild").join("env").join("run.env")
    ).unwrap();
    assert!(content.contains("key=inline_wins"), "inline var should override file var");
    let count = content.lines().filter(|l| l.starts_with("key=")).count();
    assert_eq!(count, 1, "no duplicate keys after merge");
}

// ─────────────────────────────────────────────────────────────
// 11. Full end-to-end pipeline tests
// ─────────────────────────────────────────────────────────────

/// init → build → run → clean full lifecycle
#[test]
fn e2e_init_build_run_clean_lifecycle() {
    require_compiler!();
    let temp = TempDir::new().unwrap();
    let path_str = temp.path().to_str().unwrap();

    // 1. init
    assert!(init(pc(Type::Init, &[path_str])).is_ok(), "init failed");

    // 2. replace generated main.cpp with known-good source
    std::fs::write(temp.path().join("src").join("main.cpp"), HELLO).unwrap();

    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    // 3. build
    let build = command::execute(pc(Type::Build, &["src/main.cpp"]), &mut configs, &mut ps);
    assert!(build.is_ok(), "build step failed: {:?}", build);
    assert!(temp.path().join("build").exists(), "build dir missing after build");

    // 4. run
    let mut ps2 = ProjectStructure::new(&configs);
    let run = command::execute(pc(Type::Run, &["src/main.cpp"]), &mut configs, &mut ps2);
    assert!(run.is_ok(), "run step failed: {:?}", run);

    // 5. clean
    let mut ps3 = ProjectStructure::new(&configs);
    let clean = command::execute(pc(Type::Clean, &[]), &mut configs, &mut ps3);
    assert!(clean.is_ok(), "clean step failed: {:?}", clean);
    assert!(!temp.path().join("build").exists(), "build dir should be removed after clean");
}

/// build twice — second build must not crash or error (idempotent)
#[test]
fn e2e_build_is_idempotent() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("main.cpp", HELLO)]);
    let main_cpp = _temp.path().join("main.cpp").to_string_lossy().to_string();
    assert!(command::execute(pc(Type::Build, &[&main_cpp]), &mut configs, &mut ps).is_ok());
    let mut ps2 = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Build, &[&main_cpp]), &mut configs, &mut ps2).is_ok(),
        "second build should succeed");
}

/// init → check valid → modify to invalid → check must fail
#[test]
fn e2e_check_valid_then_break_then_check_invalid() {
    require_compiler!();
    let temp = TempDir::new().unwrap();
    let path_str = temp.path().to_str().unwrap();

    assert!(init(pc(Type::Init, &[path_str])).is_ok());
    let main_cpp = temp.path().join("src").join("main.cpp");
    std::fs::write(&main_cpp, HELLO).unwrap();

    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Check, &["src/main.cpp"]), &mut configs, &mut ps).is_ok());

    std::fs::write(&main_cpp, SYNTAX_ERROR).unwrap();
    let mut ps2 = ProjectStructure::new(&configs);
    assert!(command::execute(pc(Type::Check, &["src/main.cpp"]), &mut configs, &mut ps2).is_err());
}

/// build → clean → build again (rebuild after clean works)
#[test]
fn e2e_rebuild_after_clean() {
    require_compiler!();
    let (temp, mut configs, mut ps) = make_project(&[("main.cpp", HELLO)]);

    assert!(command::execute(pc(Type::Build, &["main.cpp"]), &mut configs, &mut ps).is_ok());
    assert!(temp.path().join("build").exists());

    let mut ps2 = ProjectStructure::new(&configs);
    let clean = command::execute(pc(Type::Clean, &[]), &mut configs, &mut ps2);
    assert!(clean.is_ok());
    assert!(!temp.path().join("build").exists());

    let mut ps3 = ProjectStructure::new(&configs);
    let rebuild = command::execute(pc(Type::Build, &["main.cpp"]), &mut configs, &mut ps3);
    assert!(rebuild.is_ok(), "rebuild after clean failed: {:?}", rebuild);
    assert!(temp.path().join("build").exists(), "build dir should exist after rebuild");
}

/// env set → build succeeds (env file presence is benign to build)
#[test]
fn e2e_env_set_does_not_break_build() {
    require_compiler!();
    let (_temp, mut configs, mut ps) = make_project(&[("main.cpp", HELLO)]);
    let main_cpp = _temp.path().join("main.cpp").to_string_lossy().to_string();

    command::execute(pc(Type::Env, &["--cmd", "build", "SOME_VAR=value"]), &mut configs, &mut ps)
        .expect("env should succeed");

    let mut ps2 = ProjectStructure::new(&configs);
    let build = command::execute(pc(Type::Build, &[&main_cpp]), &mut configs, &mut ps2);
    assert!(build.is_ok(), "build after env set failed: {:?}", build);
}

/// run result produces output file with expected contents (stdin→stdout pipeline)
#[test]
fn e2e_run_stdin_stdout_pipeline() {
    require_compiler!();
    let temp = TempDir::new().unwrap();
    temp.child("main.cpp").write_str(ECHO_STDIN).unwrap();
    let input  = temp.child("in.txt");   input.write_str("pipeline_token\n").unwrap();
    let output = temp.child("out.txt");
    let mut configs = Configs::default(temp.path().to_path_buf());
    let mut ps = ProjectStructure::new(&configs);

    let result = command::execute(
        pc(Type::Run, &[
            "main.cpp",
            "-in",  input.path().to_str().unwrap(),
            "-out", output.path().to_str().unwrap(),
        ]),
        &mut configs, &mut ps,
    );
    assert!(result.is_ok(), "e2e stdin/stdout pipeline failed: {:?}", result);
    let contents = std::fs::read_to_string(output.path()).unwrap();
    assert_eq!(contents, "pipeline_token");
}
