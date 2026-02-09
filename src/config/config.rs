use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    command::init::LangType,
    compiler::BuildProfileConfig,
    compiler::{find_compiler::find_compiler, BuildProfileConfigs},
};

// will be saved to .<PROGRAM_NAME>/config.toml in the root path ,
// user edited config file will be deserialized to this struct, and then merged with the default config to get the final config used by the program
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct UserDefinedConfig {
    pub project_name: String,
    pub language: LangType,
    pub std: Option<usize>,
    #[serde(alias = "Compiler Path")]
    pub compiler_path: Option<String>,

    // any things ending with / will be considered as directory, otherwise file
    // no pattern matching supported for now , but can be added in the future if needed
    #[serde(flatten)]
    pub include_dirs: Option<HashSet<String>>, // hashet for faster access time
    #[serde(flatten)]
    pub exclude_dirs: Option<HashSet<String>>, // hashset for faster access time
    #[serde(rename = "profile", flatten)]
    pub build_profiles: Option<BuildProfileConfigs>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProjectConfigs {
    pub name: Option<String>,
    pub root: PathBuf,
    pub lang: Option<LangType>,
    pub std: Option<usize>,
    pub main: Option<String>,
    pub is_dot_main_file: bool,
    pub allowed_extensions: Vec<String>,
    pub exclude: HashSet<String>,
    pub compiler_path: Option<String>,
}

impl Default for ProjectConfigs {
    fn default() -> Self {
        ProjectConfigs {
            name: None,
            root: PathBuf::new(),
            lang: None,
            main: None,
            is_dot_main_file: false,
            allowed_extensions: vec![
                ".cpp".to_string(),
                ".c".to_string(),
                ".hpp".to_string(),
                ".h".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
            ],
            exclude: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                "docs".to_string(),
                ".ion".to_string(),
                "docker".to_string(),
            ]
            .iter()
            .cloned()
            .collect::<HashSet<String>>(),
            // as per my own testing and system
            compiler_path: Some("g++".to_string()),
            std: None,
        }
    }
}

//XXX: implement these
// If you need EnvConfigs, LogConfigs, CheckConfigs, define them here. Otherwise, remove these empty structs.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Configs {
    project: ProjectConfigs,
    user: Option<UserDefinedConfig>,
}

impl Configs {
    pub fn init(root_path: PathBuf) -> Result<Self, String> {
        let project_config_path = root_path
            .join(format!(".{}", crate::constants::PROGRAM_NAME))
            .join("project_config.json");
        let user_config_path = root_path
            .join(format!(".{}", crate::constants::PROGRAM_NAME))
            .join("config.toml");

        let project_config_str = fs::read_to_string(&project_config_path)
            .map_err(|e| format!("Failed to read project config file: {}", e))?;
        let user_config_str = fs::read_to_string(&user_config_path)
            .map_err(|e| format!("Failed to read user config file: {}", e))?;

        let mut project_config: ProjectConfigs = serde_json::from_str(&project_config_str)
            .map_err(|e| format!("Failed to parse project config JSON: {}", e))?;
        let user_config: UserDefinedConfig = toml::from_str(&user_config_str)
            .map_err(|e| format!("Failed to parse user config TOML: {}", e))?;

        // Validate user config
        if let Some(std) = user_config.std {
            if std < 11 {
                return Err(format!(
                    "Unsupported C++ standard version: {}. Minimum supported is C++11.",
                    std
                ));
            }
        }

        if let Some(ref compiler) = user_config.compiler_path {
            if compiler.trim().is_empty() {
                return Err("Compiler path cannot be empty.".to_string());
            }
        }

        // Merge user exclude dirs into project config
        if let Some(ref excluded_dirs) = user_config.exclude_dirs {
            for dir in excluded_dirs {
                if !dir.trim().is_empty() {
                    project_config.exclude.insert(dir.clone());
                }
            }
        }

        // Overwrite root_path to ensure it's current
        project_config.root = root_path;

        Ok(Configs {
            project: project_config,
            user: Some(user_config),
        })
    }
    pub fn new(
        root_path: &Path,
        configs_at: Option<&Path>,
        lang: LangType,
        std: Option<usize>,
        compiler: Option<String>,
    ) -> Result<(), String> {
        let configs_at_path = match configs_at {
            Some(path) => fs::canonicalize(path)
                .map_err(|e| format!("Failed to canonicalize config path: {}", e))?,
            None => root_path.join(format!(".{}", crate::constants::PROGRAM_NAME)),
        };

        let project_config = ProjectConfigs {
            name: root_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
            root: root_path.to_path_buf(),
            lang: Some(lang.clone()),
            std,
            main: None,
            is_dot_main_file: false,
            allowed_extensions: vec![
                ".cpp".to_string(),
                ".c".to_string(),
                ".hpp".to_string(),
                ".h".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
            ],
            exclude: vec![
                format!(".{}", crate::constants::PROGRAM_NAME),
                "node_modules".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                "docs".to_string(),
            ]
            .iter()
            .cloned()
            .collect::<HashSet<String>>(),
            compiler_path: compiler.clone().or_else(|| find_compiler()),
        };

        let user_config = UserDefinedConfig {
            project_name: root_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown_project".to_string()),
            language: lang,
            std,
            compiler_path: compiler,
            include_dirs: None,
            exclude_dirs: None,
            build_profiles: None,
        };

        fs::create_dir_all(&configs_at_path).map_err(|e| {
            format!(
                "Failed to create config directory at {}: {}",
                configs_at_path.display(),
                e
            )
        })?;

        let user_config_path = configs_at_path.join("config.toml");
        let project_config_path = configs_at_path.join("project_config.json");

        let project_config_json = serde_json::to_string_pretty(&project_config)
            .map_err(|e| format!("Failed to serialize project config to JSON: {}", e))?;

        let user_config_toml = toml::to_string_pretty(&user_config)
            .map_err(|e| format!("Failed to serialize user config to TOML: {}", e))?;

        #[cfg(any(test, debug_assertions))]
        {
            println!("Creating config files at: {}", configs_at_path.display());
        }

        fs::write(&project_config_path, project_config_json)
            .map_err(|e| format!("Failed to write project config file: {}", e))?;

        fs::write(&user_config_path, user_config_toml)
            .map_err(|e| format!("Failed to write user config file: {}", e))?;

        Ok(())
    }

    //XXX: currently a stub to be implemented with the best defaults
    pub fn default(root_path: PathBuf) -> Self {
        let project_config = ProjectConfigs {
            name: root_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
            root: root_path.clone(),
            lang: Some(LangType::Cpp),
            std: Some(11),
            main: None,
            is_dot_main_file: false,
            allowed_extensions: vec![
                ".cpp".to_string(),
                ".c".to_string(),
                ".hpp".to_string(),
                ".h".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
            ],
            exclude: vec![
                format!(".{}", crate::constants::PROGRAM_NAME),
                "node_modules".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                "docs".to_string(),
            ]
            .iter()
            .cloned()
            .collect::<HashSet<String>>(),
            compiler_path: Some("g++".to_string()),
        };

        Configs {
            project: project_config,
            user: None,
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

        let project_config = ProjectConfigs {
            name: Some("test_project".to_string()),
            root: root_path.clone(),
            lang: Some(LangType::Cpp),
            std: Some(11),
            main: None,
            is_dot_main_file: false,
            allowed_extensions: vec![
                ".cpp".to_string(),
                ".c".to_string(),
                ".hpp".to_string(),
                ".h".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
            ],
            exclude: HashSet::new(),
            compiler_path: Some("g++".to_string()),
        };

        Configs {
            project: project_config,
            user: None,
        }
    }

    pub fn get_root_path(&self) -> &PathBuf {
        &self.project.root
    }

    pub fn get_project_root(&self) -> &PathBuf {
        &self.project.root
    }

    pub fn get_allowed_extensions(&self) -> &Vec<String> {
        &self.project.allowed_extensions
    }

    pub fn compiler_path(&self) -> Option<String> {
        // Return project's compiler path (always has a value: user-provided, auto-detected, or default)
        self.project.compiler_path.clone()
    }

    pub fn build_name(&self) -> Option<String> {
        // Placeholder logic to get build name from configs
        Some("test_builds".to_string())
    }

    pub fn project_name(&self) -> Option<String> {
        // User config overrides project config
        self.user
            .as_ref()
            .map(|u| u.project_name.clone())
            .or_else(|| self.project.name.clone())
    }

    pub fn language(&self) -> Option<LangType> {
        self.user
            .as_ref()
            .map(|u| u.language.clone())
            .or_else(|| self.project.lang.clone())
    }

    pub fn std_version(&self) -> Option<usize> {
        self.user.as_ref().and_then(|u| u.std).or(self.project.std)
    }

    pub fn is_excluded_dir(&self, dir_path: &std::path::Path) -> bool {
        // Check directory name only (works with absolute paths)
        if let Some(dir_name) = dir_path.file_name() {
            let dir_str = dir_name.to_string_lossy();

            // Check project exclude list
            if self.project.exclude.contains(dir_str.as_ref()) {
                return true;
            }

            // Check user-defined excluded dirs
            if let Some(ref user) = self.user {
                if let Some(ref user_exclude) = user.exclude_dirs {
                    if user_exclude.contains(dir_str.as_ref()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn get_file(&self, file_arg: &str) -> PathBuf {
        // Return main file if arg is empty/dot and is_dot_main_file is true
        match file_arg.trim() {
            "." | "" | " " => {
                if self.project.is_dot_main_file {
                    // Use main file if configured
                    self.project
                        .main
                        .as_ref()
                        .map(|m| PathBuf::from(m))
                        .unwrap_or_else(|| PathBuf::from("."))
                } else {
                    // Default to current directory
                    PathBuf::from(".")
                }
            }
            other => PathBuf::from(other),
        }
    }

    pub fn get_build_profile(&self, profile_name: &Option<String>) -> Option<BuildProfileConfig> {
        match profile_name {
            Some(name) => self
                .user
                .as_ref()
                .and_then(|u| u.build_profiles.as_ref())
                .and_then(|profiles| profiles.get(name).cloned()),
            None => None,
        }
    }
}

impl Drop for Configs {
    //ISSUE: this will alway create the file , even if there is not file exists for the current project and where it does't need to do
    // need to resolve this problem so that only save the data when there is alrady a file
    fn drop(&mut self) {
        // Serialize the config, but don't panic if it fails
        let project_config_path = self
            .project
            .root
            .join(format!(".{}", crate::constants::PROGRAM_NAME))
            .join("project_config.json");
        if !project_config_path.exists() {
            return;
        }

        match serde_json::to_string_pretty(&self.project) {
            Ok(project_config_json) => {
                if let Err(e) = fs::write(&project_config_path, project_config_json) {
                    eprintln!("Failed to write project config file on drop: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize project config on drop: {}", e);
            }
        }
    }
}

//________________________ TEST ___________________________
#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::TempDir;

    #[test]
    fn test_config_new() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("test_project");

        let create_result = Configs::new(
            &root_path,
            None,
            LangType::Cpp,
            Some(17),
            Some("g++".to_string()),
        );
        let _project_configs = ProjectConfigs {
            name: Some("test_project".to_string()),
            root: root_path.clone(),
            lang: Some(LangType::Cpp),
            std: Some(17),
            main: None,
            is_dot_main_file: false,
            allowed_extensions: vec![
                ".cpp".to_string(),
                ".c".to_string(),
                ".hpp".to_string(),
                ".h".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
            ],
            exclude: vec![
                format!(".{}", crate::constants::PROGRAM_NAME),
                "node_modules".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                "docs".to_string(),
            ]
            .iter()
            .cloned()
            .collect::<HashSet<String>>(),
            compiler_path: Some("g++".to_string()),
        };
        let _user_config = UserDefinedConfig {
            project_name: "test_project".to_string(),
            language: LangType::Cpp,
            std: Some(17),
            compiler_path: Some("g++".to_string()),
            include_dirs: None,
            exclude_dirs: None,
            build_profiles: None,
        };
        assert!(
            create_result.is_ok(),
            "Failed to create new config: {}",
            create_result.err().unwrap()
        );

        let config_dir = root_path.join(format!(".{}", crate::constants::PROGRAM_NAME));
        assert!(
            config_dir.exists(),
            "Config directory was not created at expected location: {}",
            config_dir.display()
        );

        let new_config = Configs::init(root_path.clone()).expect("Failed to init config");
        let compiler = "g++".to_string();
        let lang = LangType::Cpp;
        let std = Some(17);

        assert_eq!(new_config.get_project_root(), &root_path);
        assert_eq!(new_config.compiler_path(), Some(compiler));
        assert_eq!(
            format!("{:?}", new_config.language().unwrap()),
            format!("{:?}", lang)
        );
        assert_eq!(new_config.std_version(), std);

        let project_config_path = root_path
            .join(format!(".{}", crate::constants::PROGRAM_NAME))
            .join("project_config.json");
        let user_config_path = root_path
            .join(format!(".{}", crate::constants::PROGRAM_NAME))
            .join("config.toml");

        assert!(
            project_config_path.exists(),
            "Project config file was not created at expected location: {}",
            project_config_path.display()
        );
        assert!(
            user_config_path.exists(),
            "User config file was not created at expected location: {}",
            user_config_path.display()
        );

        let project_content =
            fs::read_to_string(&project_config_path).expect("Failed to read project config file");
        let user_content =
            fs::read_to_string(&user_config_path).expect("Failed to read user config file");

        let project_on_disk: ProjectConfigs = serde_json::from_str(&project_content)
            .expect("Failed to parse project config JSON from disk");
        assert_eq!(project_on_disk.root, root_path);
        assert!(matches!(project_on_disk.lang, Some(LangType::Cpp)));
        assert_eq!(project_on_disk.std, Some(17));
        assert_eq!(project_on_disk.compiler_path, Some("g++".to_string()));
        assert!(
            project_on_disk
                .exclude
                .contains(&format!(".{}", crate::constants::PROGRAM_NAME)),
            "Project exclude list should include the program config directory"
        );

        let user_on_disk: UserDefinedConfig =
            toml::from_str(&user_content).expect("Failed to parse user config TOML from disk");
        assert_eq!(user_on_disk.project_name, "test_project".to_string());
        assert_eq!(
            format!("{:?}", user_on_disk.language),
            format!("{:?}", LangType::Cpp)
        );
        assert_eq!(user_on_disk.std, Some(17));
        assert_eq!(user_on_disk.compiler_path, Some("g++".to_string()));

        let init_configs = Configs::init(root_path.clone()).expect("Failed to init config");

        // Deep-compare the configs without deriving PartialEq/Eq.
        // We compare serde_json::Value because it supports structural equality and ignores object key order.
        fn normalize_project_json(v: &mut serde_json::Value) {
            let Some(obj) = v.as_object_mut() else { return };

            if let Some(exclude_v) = obj.get_mut("exclude") {
                if let Some(arr) = exclude_v.as_array_mut() {
                    arr.sort_by(|a, b| a.as_str().unwrap_or("").cmp(b.as_str().unwrap_or("")));
                }
            }
        }

        let mut new_project_v = serde_json::to_value(&new_config.project)
            .expect("Failed to convert new_config.project to JSON value");
        let mut init_project_v = serde_json::to_value(&init_configs.project)
            .expect("Failed to convert init_configs.project to JSON value");
        normalize_project_json(&mut new_project_v);
        normalize_project_json(&mut init_project_v);
        assert_eq!(
            init_project_v, new_project_v,
            "Re-initialized project config does not match the original"
        );

        let new_user_v = serde_json::to_value(&new_config.user)
            .expect("Failed to convert new_config.user to JSON value");
        let init_user_v = serde_json::to_value(&init_configs.user)
            .expect("Failed to convert init_configs.user to JSON value");
        assert_eq!(
            init_user_v, new_user_v,
            "Re-initialized user config does not match the original"
        );
    }

    #[test]
    fn test_get_file() {
        let config = Configs {
            project: ProjectConfigs {
                name: Some("test_project".to_string()),
                root: PathBuf::from("/test/root"),
                lang: Some(LangType::Cpp),
                std: Some(11),
                main: Some("main.cpp".to_string()),
                is_dot_main_file: true,
                allowed_extensions: vec![],
                exclude: HashSet::new(),
                compiler_path: None,
            },
            user: None,
        };

        // Test with "." which should resolve to main.cpp
        let file_path = config.get_file(".");
        assert_eq!(file_path, PathBuf::from("main.cpp"));

        // Test with empty string which should also resolve to main.cpp
        let file_path = config.get_file("");
        assert_eq!(file_path, PathBuf::from("main.cpp"));

        // Test with a specific file path
        let file_path = config.get_file("src/utils.cpp");
        assert_eq!(file_path, PathBuf::from("src/utils.cpp"));
    }
}
