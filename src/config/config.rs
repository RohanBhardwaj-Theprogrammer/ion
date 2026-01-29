use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::command::init::LangType;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ProjectConfigs {
    pub name: Option<String>,
    pub root: PathBuf,
    pub lang: Option<LangType>,
    pub std: Option<usize>,
    pub main: Option<PathBuf>,
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub macros: Option<Vec<String>>,
}
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
    pub configs: ProjectConfigs,
    pub root_path: PathBuf,
    allowed_extensions: Vec<String>,
    pub run: RunConfig,
    pub build: BuildConfig,
    excluded_dirs: HashSet<String>,
}

impl Configs {
    pub fn Init(root_path: PathBuf) -> Result<Self, String> {
        let config_path = root_path
            .join(format!(".{}", crate::constants::PROGRAM_NAME))
            .join("config.toml");

        let project_config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let configs: ProjectConfigs = toml::from_str(&project_config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(Configs {
            configs,
            root_path,
            allowed_extensions: vec![
                ".cpp".to_string(),
                ".c".to_string(),
                ".hpp".to_string(),
                ".h".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
            ],
            run: RunConfig::default(),
            build: BuildConfig::default(),
            excluded_dirs: HashSet::new(),
        })
    }

    pub fn new(root_path: &Path, configs_at: Option<&Path>, lang: LangType) -> Result<(), String> {
        let configs_at_path = match configs_at {
            Some(path) => fs::canonicalize(path)
                .map_err(|e| format!("Failed to canonicalize config path: {}", e))?,
            None => root_path.join(format!(".{}", crate::constants::PROGRAM_NAME)),
        };

        let project_config = ProjectConfigs {
            name: root_path
                .file_stem()
                .unwrap()
                .to_str()
                .map(|s| s.to_string()),
            root: root_path.to_path_buf(),
            lang: Some(lang.clone()),
            std: None,
            main: None,
            exclude: None,
            include: None,
            macros: None,
        };

        let toml_string = toml::to_string_pretty(&project_config)
            .map_err(|e| format!("Failed to serialize config to TOML: {}", e))?;

        match fs::create_dir_all(&configs_at_path) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "Failed to create config directory at {}: {}",
                    configs_at_path.display(),
                    e
                ));
            }
        }

        let config_file_path = configs_at_path.join("config.toml");

        fs::write(&config_file_path, toml_string)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        #[cfg(any(test, debug_assertions))]
        {
            println!("Creating config file at: {}", configs_at_path.display());
        }
        Ok(())
    }

    //XXX: currently a stub to iplemented with the best defaults
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
            configs: ProjectConfigs {
                name: match root_path.file_stem() {
                    Some(name) => name.to_str().map(|s| s.to_string()),
                    None => None,
                },
                root: root_path.clone(),
                lang: Some(LangType::Cpp),
                std: Some(11),
                main: None,
                exclude: None,
                include: None,
                macros: None,
            },
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
            configs: ProjectConfigs {
                name: Some("test_project".to_string()),
                root: PathBuf::from(&root_path),
                lang: Some(LangType::Cpp),
                std: Some(11),
                main: None,
                exclude: None,
                include: None,
                macros: None,
            },
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
