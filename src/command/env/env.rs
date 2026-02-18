//! `env` command — manage per-command environment variables.
//!
//! Stores environment variables inside the project's tool directory
//! (`.cbuild/env/`) as plain `.env` files, one per command scope.
//!
//! # Storage layout
//! ```text
//! .cbuild/
//! └── env/
//!     ├── run.env    ← vars injected when `run` executes the binary
//!     └── build.env  ← vars injected during `build` compilation
//! ```
//!
//! # File format
//! Standard `.env` syntax — one `KEY=value` pair per line.
//! Blank lines and lines starting with `#` are ignored.
//! Quoted values have their outer quotes stripped.
//!
//! # CLI usage
//! ```text
//! env [--cmd <run|build>] [KEY=VALUE ...] [-path <file>] [--clear]
//! ```
//!
//! | Flag / Arg       | Meaning                                                              |
//! |------------------|----------------------------------------------------------------------|
//! | `--cmd <scope>`  | Scope to target: `run`, `build`, or omit for both (default).        |
//! | `KEY=VALUE`      | Inline variable to set. Multiple pairs are accepted.                 |
//! | `-path <file>`   | Path to an external `.env` file to import. Inline vars override it. |
//! | `--clear`        | Wipe the scope file(s) before writing new vars.                     |
//! | `-h / --help`    | Print usage information.                                             |

use crate::cmd_parser::parser::ParsedCommand;
use crate::config::Configs;
use crate::state::ProjectStructure;
use crate::utils::trim_quotes;
use std::fs;

/// Which command scope the env vars are attached to.
///
/// Determines which `.env` file(s) under `.cbuild/env/` are read and written.
/// Defaults to [`Both`](CmdScope::Both) when `--cmd` is not supplied.
#[derive(Debug, Clone, PartialEq)]
enum CmdScope {
    /// Apply only to the `run` command (`run.env`).
    Run,
    /// Apply only to the `build` command (`build.env`).
    Build,
    /// Apply to both commands (`run.env` **and** `build.env`). Default.
    Both,
}

impl CmdScope {
    /// Parse a scope from the value that follows `--cmd` on the CLI.
    ///
    /// Accepts `"run"` or `"build"`; anything else (including an absent value)
    /// maps to [`Both`](CmdScope::Both).
    fn from_str(s: &str) -> Self {
        match s {
            "run" => Self::Run,
            "build" => Self::Build,
            _ => Self::Both,
        }
    }

    /// Returns the file name(s) under `.cbuild/env/` that this scope targets.
    fn file_names(&self) -> &'static [&'static str] {
        match self {
            Self::Run => &["run.env"],
            Self::Build => &["build.env"],
            Self::Both => &["run.env", "build.env"],
        }
    }
}

/// Parsed arguments for the `env` command.
#[derive(Debug, Clone)]
struct EnvArgs {
    /// `true` when `-h` / `--help` is present; all other fields are ignored.
    pub help_flag: bool,
    /// Target command scope (`--cmd run|build`; defaults to [`CmdScope::Both`]).
    pub scope: CmdScope,
    /// Inline `KEY=value` pairs collected from positional CLI arguments.
    pub env_vars: Vec<(String, String)>,
    /// Path to an external `.env` file supplied via `-path <file>`.
    /// Its contents are imported before inline vars (inline vars win on conflict).
    pub env_file_path: Option<String>,
    /// When `true` (`--clear`), the scope file(s) are wiped before writing.
    pub clear_args: bool,
}

/// Parse a [`ParsedCommand`] into [`EnvArgs`].
///
/// # Grammar
/// ```text
/// env_command  := "env" arg*
///
/// arg          := flag
///               | kv_pair
///               | <ignored>
///
/// flag         := "-h" | "--help" | "-help"   → help_flag = true
///               | "--clear"                   → clear_args = true
///               | "-path"  <token>            → env_file_path = <token>
///               | "--cmd"  <scope_value>      → scope = from_str(<scope_value>)
///
/// scope_value  := "run"           → CmdScope::Run
///               | "build"         → CmdScope::Build
///               | <anything_else> → CmdScope::Both  (default when --cmd absent)
///
/// kv_pair      := <key> "=" <rest>    (split on the FIRST '=' only)
///               ─ <key>  is trimmed and lowercased
///               ─ <rest> has outer quotes stripped (via trim_quotes)
///
/// <token>      := the next whitespace-delimited argument (consumed greedily)
/// <ignored>    := any argument that contains no '=' and matches no flag
/// ```
///
/// # Panics
/// Panics if `args.command_type` is not [`Type::Env`].
fn env_parser(args: &ParsedCommand, _configs: &Configs) -> EnvArgs {
    if args.command_type != crate::cmd_parser::cmd::Type::Env {
        panic!("Invalid command type for env_parser");
    }

    let mut env_args = EnvArgs {
        help_flag: false,
        scope: CmdScope::Both,
        env_vars: Vec::new(),
        env_file_path: None,
        clear_args: false,
    };

    let mut iter = args.args.iter().peekable();

    while let Some(arg) = iter.next() {
        let arg = arg.trim().to_lowercase();

        match arg.as_str() {
            "-h" | "--help" | "-help" => env_args.help_flag = true,
            "--clear" => env_args.clear_args = true,
            "-path" => {
                if let Some(val) = iter.next() {
                    env_args.env_file_path = Some(val.to_string());
                }
            }
            // --cmd <run|build>  — takes the next token as the scope value
            "--cmd" => {
                if let Some(val) = iter.next() {
                    env_args.scope = CmdScope::from_str(val.trim().to_lowercase().as_str());
                }
            }
            _ if arg.contains('=') => {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() == 2 {
                    env_args.env_vars.push((parts[0].to_string(), trim_quotes(parts[1])));
                }
            }
            _ => {} // ignore unrecognised
        }
    }

    env_args
}

/// Execute the `env` command.
///
/// # Behaviour
/// 1. Resolves the storage directory: `.cbuild/env/` (created on first use).
/// 2. If `-path` was given, reads and parses that `.env` file.
/// 3. For each file in the target scope:
///    - Loads existing `KEY=value` pairs (unless `--clear` was set).
///    - Merges imported vars, then inline CLI vars (inline wins on conflict).
///    - Writes the result back as plain `KEY=value` lines.
///
/// Returns a human-readable summary or an error message.
fn execute(
    env_args: &EnvArgs,
    _configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    if env_args.help_flag {
        println!("Env command help:");
        println!(
            "Usage: c_cpp_build_system_n_pkg_manager env [options] [KEY=VALUE ...] [--command]"
        );
        return Ok("help displayed".to_string());
    }

    #[cfg(any(test, debug_assertions))]
    dbg!(&env_args);

    // ── Resolve the env storage directory  (.cbuild/env/) ───────────────────
    let env_dir = project_structure
        .get_env_file_path()                    // returns .cbuild/env.json – reuse parent
        .map(|p| p.parent().unwrap().join("env")) // → .cbuild/env/
        .ok_or("No project root available".to_string())?;

    fs::create_dir_all(&env_dir).map_err(|e| format!("Failed to create env dir: {e}"))?;

    // ── Parse vars from -path file (standard KEY=value lines) ───────────────
    let mut imported: Vec<(String, String)> = Vec::new();
    if let Some(ref path_str) = env_args.env_file_path {
        let raw = fs::read_to_string(path_str)
            .map_err(|e| format!("Failed to read -path file '{path_str}': {e}"))?;
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((k, v)) = line.split_once('=') {
                // Lowercase keys for consistency with CLI arg keys.
                imported.push((k.trim().to_lowercase(), trim_quotes(v.trim())));
            }
        }
    }

    // ── Merge & persist for each scope file ─────────────────────────────────
    // Format: plain KEY=value lines (standard .env), one file per scope.
    let mut total_vars = 0usize;
    for fname in env_args.scope.file_names() {
        let file_path = env_dir.join(fname);

        // Load existing (skip blank/comment lines, parse KEY=value)
        let mut store: Vec<(String, String)> = if !env_args.clear_args && file_path.exists() {
            fs::read_to_string(&file_path)
                .unwrap_or_default()
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                .filter_map(|l| l.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
                .collect()
        } else {
            Vec::new()
        };

        // Merge: imported from -path file, then inline vars (later wins)
        for (k, v) in imported.iter().chain(env_args.env_vars.iter()) {
            if let Some(entry) = store.iter_mut().find(|(ek, _)| ek == k) {
                entry.1 = v.clone(); // update existing key
            } else {
                store.push((k.clone(), v.clone()));
            }
        }

        // Serialise as KEY=value lines
        let content: String = store
            .iter()
            .map(|(k, v)| format!("{k}={v}\n"))
            .collect();

        fs::write(&file_path, &content)
            .map_err(|e| format!("Failed to write {fname}: {e}"))?;

        total_vars = total_vars.max(store.len());

        #[cfg(any(test, debug_assertions))]
        println!("DEBUG: wrote {} ({} vars)", file_path.display(), store.len());
    }

    Ok(format!(
        "env: {} variable(s) saved (scope: {:?})",
        total_vars, env_args.scope
    ))
}

/// Entry point for the `env` command.
///
/// Parses `parsed_command` and delegates to [`execute`].
pub fn env(
    parsed_command: ParsedCommand,
    configs: &Configs,
    project_structure: &mut ProjectStructure,
) -> Result<String, String> {
    let env_args = env_parser(&parsed_command, configs);
    execute(&env_args, configs, project_structure)
}

//_________________________TEST__________________________________

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_parser::cmd::Type;
    use crate::config::Configs;
    use crate::state::ProjectStructure;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Build a minimal temp project and return (TempDir, Configs, ProjectStructure).
    /// TempDir must stay alive for the test to keep the directory on disk.
    fn make_project() -> (TempDir, Configs, ProjectStructure) {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let configs = Configs::default(root);
        let ps = ProjectStructure::new(&configs);
        (temp, configs, ps)
    }

    /// Read a scope file from `.cbuild/env/<fname>` relative to the temp root.
    fn read_scope_file(temp: &TempDir, fname: &str) -> String {
        let path = temp.path().join(".cbuild").join("env").join(fname);
        std::fs::read_to_string(path).unwrap_or_default()
    }

    /// Parse a `KEY=value` env file content into a sorted vec for stable assertions.
    fn parse_env_content(s: &str) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = s
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
            .filter_map(|l| {
                l.split_once('=')
                    .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            })
            .collect();
        pairs.sort();
        pairs
    }

    // ── parser unit tests ────────────────────────────────────────────────────

    /// Covers: all flags together, scope variants (run/build/both), help flag,
    /// empty args defaults, and key lowercasing.
    #[test]
    fn parser_flags_and_scopes() {
        let configs = Configs::default(std::path::PathBuf::from("./some/path"));

        // All flags at once — scope=run, vars, -path, --clear
        let full = env_parser(
            &ParsedCommand {
                command_type: Type::Env,
                args: vec![
                    "VAR1=value1".to_string(),
                    "VAR2=\"value with spaces\"".to_string(),
                    "--cmd".to_string(), "run".to_string(),
                    "-path".to_string(), "env_file.env".to_string(),
                    "--clear".to_string(),
                ],
            },
            &configs,
        );
        assert!(!full.help_flag);
        assert_eq!(full.scope, CmdScope::Run);
        assert_eq!(full.env_vars, [("var1".to_string(), "value1".to_string()), ("var2".to_string(), "value with spaces".to_string())]);
        assert_eq!(full.env_file_path, Some("env_file.env".to_string()));
        assert!(full.clear_args);

        // --cmd build + key lowercasing
        let build = env_parser(
            &ParsedCommand {
                command_type: Type::Env,
                args: vec!["--cmd".to_string(), "build".to_string(), "CC=gcc".to_string()],
            },
            &configs,
        );
        assert_eq!(build.scope, CmdScope::Build);
        assert_eq!(build.env_vars[0].0, "cc");

        // no --cmd → Both; no args → all defaults
        let both = env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["KEY=val".to_string()] }, &configs);
        assert_eq!(both.scope, CmdScope::Both);

        let empty = env_parser(&ParsedCommand { command_type: Type::Env, args: vec![] }, &configs);
        assert!(!empty.help_flag);
        assert_eq!(empty.scope, CmdScope::Both);
        assert!(empty.env_vars.is_empty());
        assert!(empty.env_file_path.is_none());
        assert!(!empty.clear_args);

        // --help flag
        let help = env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--help".to_string()] }, &configs);
        assert!(help.help_flag);
    }

    /// Covers: value containing '=' is split only on the first '='.
    #[test]
    fn parser_edge_cases() {
        let configs = Configs::default(std::path::PathBuf::from("./some/path"));

        let env_args = env_parser(
            &ParsedCommand {
                command_type: Type::Env,
                args: vec!["url=http://host?a=1".to_string()],
            },
            &configs,
        );
        assert_eq!(env_args.env_vars[0], ("url".to_string(), "http://host?a=1".to_string()));
    }

    // ── execute integration tests (file I/O, no compiler) ───────────────────

    /// Covers: Both scope writes both files; run-only leaves build.env untouched;
    /// build-only leaves run.env untouched.
    #[test]
    fn execute_scope_routing() {
        // Both scope → both files written
        let (temp, configs, mut ps) = make_project();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["CC=gcc".to_string(), "OPT=-O2".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        for fname in &["run.env", "build.env"] {
            let pairs = parse_env_content(&read_scope_file(&temp, fname));
            assert!(pairs.iter().any(|(k, v)| k == "cc" && v == "gcc"), "{fname}: {pairs:?}");
            assert!(pairs.iter().any(|(k, v)| k == "opt" && v == "-o2"), "{fname}: {pairs:?}");
        }

        // run only → build.env untouched
        let (temp, configs, mut ps) = make_project();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--cmd".to_string(), "run".to_string(), "PORT=8080".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        assert!(parse_env_content(&read_scope_file(&temp, "run.env")).iter().any(|(k, _)| k == "port"));
        assert!(read_scope_file(&temp, "build.env").is_empty(), "build.env should be untouched");

        // build only → run.env untouched
        let (temp, configs, mut ps) = make_project();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--cmd".to_string(), "build".to_string(), "CXX=clang++".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        assert!(parse_env_content(&read_scope_file(&temp, "build.env")).iter().any(|(k, v)| k == "cxx" && v == "clang++"));
        assert!(read_scope_file(&temp, "run.env").is_empty(), "run.env should be untouched");
    }

    /// Covers: second write updates an existing key without duplicating it;
    /// --clear wipes all old vars before writing.
    #[test]
    fn execute_merge_and_clear() {
        // Merge: update existing key, preserve others, no duplicates
        let (temp, configs, mut ps) = make_project();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--cmd".to_string(), "build".to_string(), "CC=gcc".to_string(), "EXTRA=yes".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--cmd".to_string(), "build".to_string(), "CC=clang".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        let pairs = parse_env_content(&read_scope_file(&temp, "build.env"));
        assert!(pairs.iter().any(|(k, v)| k == "cc" && v == "clang"), "CC should be updated: {pairs:?}");
        assert!(pairs.iter().any(|(k, v)| k == "extra" && v == "yes"), "EXTRA should survive: {pairs:?}");
        assert_eq!(pairs.len(), 2, "no duplicates: {pairs:?}");

        // Clear: only the new var survives
        let (temp, configs, mut ps) = make_project();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--cmd".to_string(), "run".to_string(), "OLD=1".to_string(), "ALSO=2".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--cmd".to_string(), "run".to_string(), "--clear".to_string(), "NEW=3".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        let pairs = parse_env_content(&read_scope_file(&temp, "run.env"));
        assert_eq!(pairs, [("new".to_string(), "3".to_string())], "only NEW after --clear: {pairs:?}");
    }

    /// Covers: -path file imports vars; inline CLI wins over file on conflict;
    /// comments and blank lines in the -path file are ignored.
    #[test]
    fn execute_path_file_import() {
        // Import + CLI override
        let (temp, configs, mut ps) = make_project();
        let ext = temp.child("external.env");
        ext.write_str("FROM_FILE=imported\nSHARED=from_file\n").unwrap();
        execute(
            &env_parser(&ParsedCommand {
                command_type: Type::Env,
                args: vec!["--cmd".to_string(), "run".to_string(), "-path".to_string(), ext.path().to_string_lossy().to_string(), "SHARED=from_cli".to_string()],
            }, &configs),
            &configs, &mut ps,
        ).unwrap();
        let pairs = parse_env_content(&read_scope_file(&temp, "run.env"));
        assert!(pairs.iter().any(|(k, v)| k == "from_file" && v == "imported"), "{pairs:?}");
        let shared: Vec<_> = pairs.iter().filter(|(k, _)| k == "shared").collect();
        assert_eq!(shared.len(), 1, "one 'shared' key: {pairs:?}");
        assert_eq!(shared[0].1, "from_cli", "inline wins: {pairs:?}");

        // Comments and blank lines are skipped
        let (temp, configs, mut ps) = make_project();
        let ext = temp.child("commented.env");
        ext.write_str("# comment\n\nVALID=yes\n# another\n").unwrap();
        execute(
            &env_parser(&ParsedCommand {
                command_type: Type::Env,
                args: vec!["--cmd".to_string(), "build".to_string(), "-path".to_string(), ext.path().to_string_lossy().to_string()],
            }, &configs),
            &configs, &mut ps,
        ).unwrap();
        let pairs = parse_env_content(&read_scope_file(&temp, "build.env"));
        assert_eq!(pairs, [("valid".to_string(), "yes".to_string())], "{pairs:?}");
    }

    /// Covers: --help exits early without creating any files;
    /// first run auto-creates .cbuild/env/.
    #[test]
    fn execute_side_effects() {
        // --help must not create .cbuild/env/
        let (temp, configs, mut ps) = make_project();
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["--help".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        assert!(!temp.path().join(".cbuild").join("env").exists(), "--help must not create env dir");

        // first normal call must create .cbuild/env/
        let (temp, configs, mut ps) = make_project();
        assert!(!temp.path().join(".cbuild").join("env").exists());
        execute(
            &env_parser(&ParsedCommand { command_type: Type::Env, args: vec!["KEY=val".to_string()] }, &configs),
            &configs, &mut ps,
        ).unwrap();
        assert!(temp.path().join(".cbuild").join("env").is_dir(), ".cbuild/env/ must be created");
    }
}
