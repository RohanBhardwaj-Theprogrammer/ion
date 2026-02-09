use crate::compiler::ToCompilerArgs;
use crate::config::Configs;
use crate::deps::DependencyGraph;
use crate::state::structure::ProjectStructure;

#[derive(Clone, Debug)]
pub struct IncludeFiles {
    pub main_file: String,
    pub header_files: Vec<String>,
    pub source_files: Vec<String>,
}

impl IncludeFiles {
    pub fn new(
        deps: &DependencyGraph,
        _configs: &Configs,
        _project_structure: &ProjectStructure,
        main_file: &str,
    ) -> Self {
        let header_files = deps.get_main_includes();
        let source_files = deps.get_source_files_from(main_file);

        IncludeFiles {
            main_file: main_file.to_string(),
            header_files,
            source_files,
        }
    }

    #[cfg(test)]
    pub fn new_from_main(configs: &Configs, main_file: &str) -> Self {
        let project_structure = ProjectStructure::new(configs);
        let deps = DependencyGraph::new(main_file, &project_structure);
        Self::new(&deps, configs, &project_structure, main_file)
    }
}

impl ToCompilerArgs for IncludeFiles {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for header in &self.header_files {
            args.push(format!("-I{}", header));
        }
        for source in &self.source_files {
            args.push(source.clone());
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Configs;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use std::path::PathBuf;

    #[test]
    fn include_files_tests() {
        let temp = TempDir::new().unwrap();

        // Minimal realistic layout.
        temp.child("src").create_dir_all().unwrap();
        temp.child("include").create_dir_all().unwrap();

        temp.child("include/logger.h")
            .write_str("#pragma once\nvoid log();\n")
            .unwrap();
        temp.child("include/util.h")
            .write_str("#pragma once\nint util();\n")
            .unwrap();

        temp.child("src/logger.cpp")
            .write_str("#include \"logger.h\"\nvoid log(){}\n")
            .unwrap();
        temp.child("src/util.cpp")
            .write_str("#include \"util.h\"\nint util(){return 1;}\n")
            .unwrap();

        temp.child("src/main.cpp")
            .write_str(
                "#include \"logger.h\"\n#include \"util.h\"\nint main(){log(); return util();}\n",
            )
            .unwrap();

        let configs = Configs::default(temp.path().to_path_buf());
        let main_file = temp.child("src/main.cpp").path().to_string_lossy().to_string();
        let include_files = IncludeFiles::new_from_main(&configs, &main_file);

        assert!(
            include_files.main_file.contains("main.cpp"),
            "main_file was: {}",
            include_files.main_file
        );

        // `header_files` actually contains include directories (see DependencyGraph::get_main_includes).
        let include_dir = std::fs::canonicalize(temp.child("include").path()).unwrap();
        let has_include_dir = include_files
            .header_files
            .iter()
            .any(|p| PathBuf::from(p) == include_dir);
        assert!(
            has_include_dir,
            "include directories should include {:?}; got: {:?}",
            include_dir,
            include_files.header_files
        );

        // Source discovery should include main + implementation files for included headers.
        for expected in ["main.cpp", "logger.cpp", "util.cpp"] {
            let found = include_files
                .source_files
                .iter()
                .any(|f| f.contains(expected));
            assert!(found, "Source file {} should be found", expected);
        }
    }
}
