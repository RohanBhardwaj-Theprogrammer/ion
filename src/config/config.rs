use std::{collections::HashSet, fs, path::PathBuf};

use crate::command::init::LangType;

pub struct BuildConfig {
    pub std: usize,
    pub lang: LangType,
    pub optimization_level: usize,
    pub is_dot_main_file: bool,
    pub last_file: String,
    pub main_file: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        BuildConfig {
            std: 11,
            lang: LangType::Cpp,
            optimization_level: 0,
            is_dot_main_file: false,
            last_file: String::new(),
            main_file: String::new(),
        }
    }
}

impl BuildConfig {
    pub fn new(main_file: &str) -> Self {
        BuildConfig {
            std: 11,
            lang: LangType::Cpp,
            optimization_level: 0,
            is_dot_main_file: true,
            last_file: String::new(),
            main_file: main_file.to_string(),
        }
    }
}

pub struct RunConfig {
    pub std: usize,
    pub file: String,
    pub optimization_level: usize,
    pub is_dot_main_file: bool,
    pub last_run_file: String,
    pub last_args: Vec<String>,
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig {
            file: "src/main.cpp".to_string(),
            std: 11,
            optimization_level: 0,
            is_dot_main_file: false,
            last_run_file: String::new(),
            last_args: Vec::new(),
        }
    }
}

impl RunConfig {
    pub fn new(file_path: &str) -> Self {
        RunConfig {
            file: file_path.to_string(),
            std: 11,
            optimization_level: 0,
            is_dot_main_file: true,
            last_run_file: String::new(),
            last_args: Vec::new(),
        }
    }
}

//XXX: implement these
// If you need EnvConfigs, LogConfigs, CheckConfigs, define them here. Otherwise, remove these empty structs.

pub struct Configs {
    root_path: PathBuf,
    allowed_extensions: Vec<String>,
    pub run: RunConfig,
    pub build: BuildConfig,
    excluded_dirs: HashSet<String>,
}

impl Configs {
    pub fn new(root_path: PathBuf, allowed_extensions: Vec<String>) -> Self {
        let build = BuildConfig::new("");
        let run = RunConfig::default();

        Configs {
            root_path: fs::canonicalize(&root_path).unwrap_or_else(|_| root_path.clone()),
            allowed_extensions,
            run,
            build,
            excluded_dirs: HashSet::new(),
        }
    }

    pub fn default(root_path: PathBuf) -> Self {
        let allowed_extensions = vec![
            ".cpp".to_string(),
            ".c".to_string(),
            ".hpp".to_string(),
            ".h".to_string(),
            ".cc".to_string(),
            ".cxx".to_string(),
        ];

        let excluded_dirs = [
            format!(".{}", crate::constants::PROGRAM_NAME),
            "node_modules".to_string(),
            ".git".to_string(),
            ".vscode".to_string(),
            "docs".to_string(),
        ]
        .iter()
        .cloned()
        .collect::<HashSet<String>>();

        Configs {
            root_path: root_path,
            allowed_extensions,
            run: RunConfig::default(),
            build: BuildConfig::default(),
            excluded_dirs,
        }
    }
    #[cfg(test)]
    pub fn test_config(path: Option<&str>) -> Self {
        use crate::utils::path_utils;
        use std::path::{Path, PathBuf};

        let root_path = match path {
            None => PathBuf::from("./tests/test_project"),
            Some(p) => {
                let raw = p.trim();
                let raw_path = Path::new(raw);

                // Avoid creating "./tests/tests/..." when caller already included "tests/".
                let candidate = if path_utils::starts_with_dot_relative(raw) {
                    PathBuf::from(raw)
                } else if raw.starts_with("tests/") || raw.starts_with("tests\\") {
                    PathBuf::from("./").join(raw)
                } else {
                    PathBuf::from("./tests").join(raw)
                };

                // If it's a file path (e.g. config.toml), use its parent directory as root.
                if candidate.extension().is_some() {
                    candidate
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| PathBuf::from("./tests/test_project"))
                } else {
                    candidate
                }
            }
        };

        let root_path = fs::canonicalize(&root_path).unwrap_or_else(|_| root_path);

        println!("Using the test the Path as : {}", root_path.display());
        let allowed_extensions = vec![
            ".cpp".to_string(),
            ".c".to_string(),
            ".hpp".to_string(),
            ".h".to_string(),
            ".cc".to_string(),
            ".cxx".to_string(),
        ];

        Configs {
            root_path: PathBuf::from(root_path),
            allowed_extensions,
            run: RunConfig::default(),
            build: BuildConfig::default(),
            excluded_dirs: HashSet::new(),
        }
    }

    pub fn get_root_path(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn get_project_root(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn get_allowed_extensions(&self) -> &Vec<String> {
        &self.allowed_extensions
    }

    pub fn compiler_path(&self) -> Option<String> {
        // Placeholder logic to get compiler path from configs
        Some("g++".to_string())
    }

    pub fn build_name(&self) -> Option<String> {
        // Placeholder logic to get build name from configs
        Some("test_builds".to_string())
    }

    pub fn project_name(&self) -> Option<String> {
        // Placeholder logic to get project name from configs
        Some("sample_project".to_string())
    }

    pub fn is_excluded_dir(&self, dir_path: &std::path::Path) -> bool {
        self.excluded_dirs
            .contains(dir_path.to_string_lossy().as_ref())
    }

    // Optionally, you can add a wrapper for build.get_file if needed
    pub fn get_file(&self, file_arg: &str) -> PathBuf {
        match file_arg.trim() {
            "." | "" | " " => {
                if self.build.is_dot_main_file {
                    if !self.build.main_file.is_empty() {
                        PathBuf::from(&self.build.main_file)
                    } else {
                        PathBuf::from(&self.build.last_file)
                    }
                } else {
                    PathBuf::from(&self.build.last_file)
                }
            }
            other => PathBuf::from(other),
        }
    }
}
