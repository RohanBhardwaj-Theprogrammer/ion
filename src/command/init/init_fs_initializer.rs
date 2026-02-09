use crate::command::init::LangType;
use crate::utils::is_initialized_project;
use std::fs;
use std::path::Path;

pub fn generate_dotfile(path: &str, force: bool) -> Result<std::path::PathBuf, String> {
    let is_project = is_initialized_project(path);
    let dotfolder = Path::new(path).join(format!(".{}", crate::constants::PROGRAM_NAME));

    if is_project && force {
        #[cfg(any(test, debug_assertions))]
        eprintln!(
            "Warning: Directory {0} is already initialized. Overwriting due to --force flag.\n 
               [Note] : only the .{0} folder will be overwritten, existing project files and folders will be preserved.",
            path
        );
        crate::utils::truncate(&dotfolder).map_err(|e| e.to_string())?;
    } else if is_project && !force {
        return Err(format!(
            "File {} already exists. Use --force|-f to overwrite.",
            path
        ));
    }

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

    fn tree_view(structure: &CustomProjectStructure) {
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
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn generate_dotfile_creates_dotfolder_when_missing() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let root_s = root.to_str().expect("temp path should be valid utf-8");

        let dot = generate_dotfile(root_s, false).expect("generate_dotfile should succeed");
        assert_eq!(
            dot,
            root.join(format!(".{}", crate::constants::PROGRAM_NAME))
        );
        assert!(dot.exists(), "dot folder should exist");
        assert!(dot.is_dir(), "dot folder should be a directory");
    }

    #[test]
    fn generate_dotfile_errors_if_already_initialized_and_no_force() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let root_s = root.to_str().expect("temp path should be valid utf-8");

        // Create the marker directory to make it look initialized.
        std::fs::create_dir_all(root.join(format!(".{}", crate::constants::PROGRAM_NAME))).unwrap();

        let result = generate_dotfile(root_s, false);
        assert!(
            result.is_err(),
            "should error when already initialized without force"
        );
    }

    #[test]
    fn generate_dotfile_force_truncates_dotfolder_only() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let root_s = root.to_str().expect("temp path should be valid utf-8");

        let dot = root.join(format!(".{}", crate::constants::PROGRAM_NAME));
        std::fs::create_dir_all(&dot).unwrap();
        std::fs::write(dot.join("old.txt"), "old").unwrap();
        std::fs::write(root.join("keep.txt"), "keep").unwrap();

        let _ = generate_dotfile(root_s, true).expect("force should succeed");

        assert!(
            root.join("keep.txt").exists(),
            "non-dot files should be preserved"
        );
        assert!(
            !dot.join("old.txt").exists(),
            "dotfolder contents should be truncated"
        );
        assert!(dot.exists(), "dotfolder should still exist");
    }

    #[test]
    fn std_structure_at_creates_expected_layout_for_c_and_cpp() {
        let temp = TempDir::new().unwrap();

        let project_c = temp.child("proj_c");
        sketch::std_structure_at(project_c.path(), &LangType::C, false).expect("C structure");
        assert!(project_c.path().join("src").is_dir());
        assert!(project_c.path().join("include").is_dir());
        assert!(project_c.path().join("build").is_dir());
        assert!(project_c.path().join("src").join("main.c").is_file());

        let project_cpp = temp.child("proj_cpp");
        sketch::std_structure_at(project_cpp.path(), &LangType::Cpp, false).expect("CPP structure");
        assert!(project_cpp.path().join("src").join("main.cpp").is_file());
    }

    #[test]
    fn std_structure_at_errors_without_force_when_structure_exists() {
        let temp = TempDir::new().unwrap();
        let project = temp.child("proj_exists");
        std::fs::create_dir_all(project.path().join("src")).unwrap();

        let result = sketch::std_structure_at(project.path(), &LangType::Cpp, false);
        assert!(
            result.is_err(),
            "should error when src/include/build exist and no force"
        );
    }

    #[test]
    fn generate_custom_structure_creates_nested_files_and_folders() {
        let temp = TempDir::new().unwrap();
        let json = temp.child("format.json");
        json.write_str(
            r#"{
    "name": "my_project",
    "folders": [
    {
            "name": "src",
            "folders": [
        {
                    "name": "modules",
                    "files": ["mod1.c", "mod2.c"]
        }
      ],
            "files": ["main.c", "utils.c"]
    },
    {
            "name": "include",
            "files": ["main.h", "utils.h"]
    }
  ],
    "files": ["README.md", "LICENSE"]
}"#,
        )
        .unwrap();

        let target = temp.child("target");
        assert!(!target.path().exists());

        sketch::generate_custom_structure(json.path(), target.path(), false)
            .expect("should generate custom structure");

        let base = target.path().join("my_project");
        assert!(base.is_dir());
        assert!(base.join("README.md").is_file());
        assert!(base.join("LICENSE").is_file());
        assert!(base.join("src").join("main.c").is_file());
        assert!(base.join("src").join("utils.c").is_file());
        assert!(base.join("src").join("modules").join("mod1.c").is_file());
        assert!(base.join("src").join("modules").join("mod2.c").is_file());
        assert!(base.join("include").join("main.h").is_file());
        assert!(base.join("include").join("utils.h").is_file());
    }

    #[test]
    fn generate_project_structure_uses_custom_when_path_provided() {
        let temp = TempDir::new().unwrap();
        let json = temp.child("format.json");
        json.write_str(r#"{"name":"p","files":["a.txt"]}"#).unwrap();

        let target = temp.child("wrapper_target");
        assert!(!target.path().exists());

        generate_project_structure(target.path(), Some(json.path()), &LangType::Cpp, false)
            .expect("wrapper should generate custom structure");

        assert!(target.path().join("p").join("a.txt").is_file());
    }
}
