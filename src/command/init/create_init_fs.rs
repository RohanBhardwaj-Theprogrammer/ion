use crate::command::init::LangType;
use crate::utils::is_cbuild_project;
use std::fs;
use std::path::Path;

pub fn generate_dotfile(path: &str, force: bool) -> Result<std::path::PathBuf, String> {
    let is_project = is_cbuild_project(path);

    if is_project && force {
        crate::utils::truncate(path).map_err(|e| e.to_string())?;
    } else if is_project && !force {
        return Err(format!(
            "File {} already exists. Use --force|-f to overwrite.",
            path
        ));
    }

    let dotfolder = Path::new(path).join(format!(".{}", crate::constants::PROGRAM_NAME));

    fs::create_dir_all(&dotfolder).map_err(|e| e.to_string())?;

    #[cfg(any(test, debug_assertions))]
    {
        let debug_string = format!(
            " Debug: generate_dotfile called with path: {} , force : {} ",
            path, force
        );
        println!(" dotfolder path : {} ", dotfolder.display());

        println!(" Debug Info : {} ", debug_string);

        println!(" Functions being called is generate_dotfile ");
    }
    Ok(dotfolder)
}

pub fn generate_project_structure(
    path: &Path,
    structure_path: Option<&Path>,
    lang: &LangType,
    force: bool,
) -> Result<(), String> {
    if let Some(structure_path) = structure_path {
        sketch::generate_custom_structure(structure_path, path, force)
    } else {
        sketch::std_structure_at(Path::new(path), lang, force)
    }
}

pub mod sketch {
    use std::fs;
    use std::path::Path;

    const CTEMPLATE_MAIN: &str = "#include <stdio.h>\n  int main() {\n        printf(\"Hello, World!\\n\");\n        return 0;\n    }";
    const CPPTEMPLATE_MAIN: &str = "#include <iostream>\n using std::cout ;\n using std::endl;\n\n  int main() {\n        cout << \"Hello, World!\" <<endl;\n        return 0;\n    }\n";

    use crate::command::init::LangType;

    use serde::Deserialize;
    use serde_json;

    /*
     * A structure to represent custom project structures.
     * {
     *  name: String,
     *  folders: Vec<CustomProjectStructure>,
     * files: Vec<String>,
     *
     * }
     *  for example :
     *
     *  {
     *  name: "src",
     *  folders: [
     *      {
     *         name: "modules",
     *        folders: [],
     *       files: ["mod1.c", "mod2.c"],
     *     }
     *  ],
     * files: ["main.c", "utils.c"],
     *
     * }
     */

    #[derive(Debug, Deserialize)]
    struct CustomProjectStructure {
        name: String,
        folders: Option<Vec<CustomProjectStructure>>,
        files: Option<Vec<String>>,
    }

    pub fn std_structure_at(path: &Path, lang: &LangType, force: bool) -> Result<(), String> {
        #[cfg(any(test, debug_assertions))]
        println!(
            " =======================================\n                    Only support fixed type yet \n                 ======================================="
        );

        let main_file = path.join("src").join(format!(
            "main.{}",
            match &lang {
                LangType::C => "c",
                _ => "cpp",
            }
        ));

        let src = path.join("src");
        let include = path.join("include");
        let build = path.join("build");

        if src.exists() || include.exists() || build.exists() {
            if force {
                println!(
                    "Warning: Some directories/files already exist at {}. Overwriting due to --force flag.",
                    path.display()
                );
            } else {
                return Err(format!(
                    "Some directories/files already exist at {}. Use --force|-f to overwrite.",
                    path.display()
                ));
            }
        }

        create_dir(&src)?;
        std::fs::File::create(&main_file).map_err(|e| e.to_string())?;
        match lang {
            LangType::C => {
                fs::write(&main_file, CTEMPLATE_MAIN).map_err(|e| e.to_string())?;
            }
            _ => {
                fs::write(&main_file, CPPTEMPLATE_MAIN).map_err(|e| e.to_string())?;
            }
        }

        create_dir(&include)?;
        create_dir(&build)?;

        Ok(())
    }

    fn create_dir(path: &Path) -> Result<(), String> {
        fs::create_dir_all(path)
            .map_err(|e| format!("Error creating directory {}: {}", path.display(), e))
    }

    pub(crate) fn generate_custom_structure(
        format_path: &Path,
        target_path: &Path,
        force: bool,
    ) -> Result<(), String> {
        if target_path.exists() {
            if force {
                println!(
                    "Warning: Target path {} already exists. Overwriting due to --force flag.",
                    target_path.display()
                );
            } else {
                return Err(format!(
                    "Target path {} already exists. Use --force|-f to overwrite.",
                    target_path.display()
                ));
            }
        }
        let format_content = fs::read_to_string(format_path).map_err(|e| e.to_string())?;
        let custom_format: CustomProjectStructure =
            serde_json::from_str(&format_content).map_err(|e| e.to_string())?;

        let base_path = target_path.to_path_buf();
        println!(" Structure Preview : ");
        tree_view(&custom_format);
        println!(
            " \n Proceeding to create structure at : {} ",
            base_path.display()
        );
        create_from_structure(&custom_format, &base_path)
    }

    fn create_from_structure(
        structure: &CustomProjectStructure,
        target_path: &Path,
    ) -> Result<(), String> {
        // safe handling to avoid double joining
        let dir_path = {
            if target_path.ends_with(&structure.name)
                || structure.name.is_empty()
                || structure.name == "."
            {
                target_path.to_path_buf()
            } else {
                target_path.join(&structure.name)
            }
        };

        create_dir(&dir_path).map_err(|e| e.to_string())?;
        #[cfg(any(test, debug_assertions))]
        {
            println!(" Creating directory : {} ", dir_path.display());
        }

        if let Some(files) = &structure.files {
            for file in files {
                let file_path = dir_path.join(file);
                fs::File::create(&file_path).map_err(|e| e.to_string())?;

                #[cfg(any(test, debug_assertions))]
                {
                    println!(" Creating file : {} ", file_path.display());
                    if let Ok(metadata) = fs::metadata(&file_path) {
                        println!("  File created with size: {} bytes", metadata.len());
                    }
                }
            }
        }

        if let Some(folders) = &structure.folders {
            for folder in folders {
                create_from_structure(folder, &dir_path)?;
            }
        }

        Ok(())
    }

    pub fn tree_view(structure: &CustomProjectStructure) {
        struct PrintState<'a> {
            node: &'a CustomProjectStructure,
            prefix: String,
            is_last: bool,
        }

        let mut stack: Vec<PrintState> = Vec::new();
        stack.push(PrintState {
            node: structure,
            prefix: " ".repeat(5).to_string(),
            is_last: true,
        });

        while let Some(state) = stack.pop() {
            let branch = if state.is_last {
                "└── "
            } else {
                "├── "
            };
            println!("{}{}{}/", state.prefix, branch, state.node.name);

            // Print files
            if let Some(files) = &state.node.files {
                let last_files_index = files.len().saturating_sub(1);
                let have_any_folder = if state.node.folders.is_some()
                    && !state.node.folders.as_ref().unwrap().is_empty()
                {
                    true
                } else {
                    false
                };

                for (i, file) in files.iter().enumerate() {
                    let file_branch = if i == last_files_index && !have_any_folder {
                        "└── "
                    } else {
                        "├── "
                    };
                    let mut file_prefix = state.prefix.clone();
                    file_prefix.push_str(if state.is_last { "    " } else { "│   " });
                    println!("{}{}{}", file_prefix, file_branch, file);
                }
            }

            // Print folders
            if let Some(folders) = &state.node.folders {
                let last_folder_index = folders.len().saturating_sub(1);
                let mut new_prefix = state.prefix.clone();
                new_prefix.push_str(if state.is_last { "    " } else { "│   " });
                for (i, folder) in folders.iter().enumerate().rev() {
                    let is_last_folder = i == last_folder_index;
                    stack.push(PrintState {
                        node: folder,
                        prefix: new_prefix.clone(),
                        is_last: is_last_folder,
                    });
                }
            }
        }
    }
}

//________________________________TESTS____________________________________
#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::init::LangType;
    use std::fs;
    use std::path::Path;

    fn write_json(path: &Path, contents: &str) {
        fs::write(path, contents).expect("failed to write test json");
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_is_cbuild_project_detects_marker() {
        let test_path = "test_cbuild_project__2333";

        fs::create_dir_all(test_path).unwrap();
        fs::create_dir_all(format!("{}/.{}", test_path, crate::constants::PROGRAM_NAME)).unwrap();

        assert!(is_cbuild_project(test_path));
    }

    #[test]
    fn test_is_cbuild_project_returns_false_when_missing() {
        let test_path = "test_cbuild_project_missing";
        fs::create_dir_all(test_path).unwrap();

        assert!(!is_cbuild_project(test_path));
    }

    #[test]
    fn test_generate_dotfile_creates_layout() {
        let temp_dir = "test_generate_structure";
        fs::create_dir_all(temp_dir).unwrap();

        generate_dotfile(temp_dir, true).unwrap();

        let dot_dir = format!("{}/.{}", temp_dir, crate::constants::PROGRAM_NAME);
        let info_file = format!("{}/.info", dot_dir);

        assert!(Path::new(&dot_dir).exists());
        assert!(Path::new(&info_file).exists());
    }

    #[test]
    fn test_generate_dotfile_without_force_errors_if_exists() {
        let temp_dir = "test_generate_no_force";
        fs::create_dir_all(temp_dir).unwrap();
        fs::create_dir_all(format!("{}/.{}", temp_dir, crate::constants::PROGRAM_NAME)).unwrap();

        let result = generate_dotfile(temp_dir, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_structure_at_creates_directories() {
        let temp_dir = "test_structure_at";
        fs::create_dir_all(temp_dir).unwrap();

        sketch::std_structure_at(Path::new(temp_dir), &LangType::C, true).unwrap();

        assert!(Path::new(&format!("{}/src", temp_dir)).exists());
        assert!(Path::new(&format!("{}/include", temp_dir)).exists());
        assert!(Path::new(&format!("{}/build", temp_dir)).exists());
    }

    #[test]
    fn test_structure_at_errors_without_force_when_exists() {
        let temp_dir = "test_structure_no_force";
        fs::create_dir_all(format!("{}/src", temp_dir)).unwrap();

        let result = sketch::std_structure_at(Path::new(temp_dir), &LangType::Cpp, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_dotfile_wrapper_calls_generate() {
        let temp_dir = "test_sketch_dotfile";
        fs::create_dir_all(temp_dir).unwrap();

        super::generate_dotfile(temp_dir, true).unwrap();

        let dot_dir = format!("{}/.{}", temp_dir, crate::constants::PROGRAM_NAME);
        assert!(Path::new(&dot_dir).exists());
    }

    #[test]
    fn test_generate_custom_structure_files_only() {
        let tmp = "test_custom_structure_files_only";
        cleanup(tmp);

        let json_path = Path::new(tmp).with_extension("json");
        let json = r#"{
            "name": "proj",
            "files": ["a.txt", "b.txt"]
        }"#;
        write_json(&json_path, json);

        sketch::generate_custom_structure(&json_path, Path::new(tmp), true).unwrap();

        assert!(Path::new(tmp).join("proj").join("a.txt").exists());
        assert!(Path::new(tmp).join("proj").join("b.txt").exists());

        cleanup(tmp);
        let _ = fs::remove_file(json_path);
    }

    #[test]
    fn test_generate_custom_structure_nested_folders() {
        let tmp = "test_custom_structure_nested";
        cleanup(tmp);

        let json_path = Path::new(tmp).with_extension("json");
        let json = r#"{
            "name": "proj",
            "folders": [
                {
                    "name": "nested",
                    "files": ["x.c"],
                    "folders": [{"name": "inner", "files": ["y.c"]}]
                }
            ]
        }"#;
        write_json(&json_path, json);

        sketch::generate_custom_structure(&json_path, Path::new(tmp), true).unwrap();

        assert!(Path::new(tmp).join("proj/nested").exists());
        assert!(Path::new(tmp).join("proj/nested/x.c").exists());
        assert!(Path::new(tmp).join("proj/nested/inner/y.c").exists());

        cleanup(tmp);
        let _ = fs::remove_file(json_path);
    }
}
