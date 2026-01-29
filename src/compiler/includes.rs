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
    use std::path::PathBuf;

    #[test]
    fn include_files_tests() {
        let root_path = "./tests/test_project";
        let main_file = format!("{}/main.cpp", root_path);
        let main_file_path = PathBuf::from(&main_file);
        let configs = Configs::default(main_file_path.parent().unwrap().to_path_buf());
        let include_files = IncludeFiles::new_from_main(&configs, &main_file);
        if !include_files.main_file.contains("main.cpp") {
            println!(
                "DEBUG: include_files.main_file = {}",
                include_files.main_file
            );
        }
        assert!(
            include_files.main_file.contains("main.cpp"),
            "main_file was: {}",
            include_files.main_file
        );
        assert!(!include_files.header_files.is_empty());
        let expected_sources = [
            "main.cpp",
            "some_code.cpp",
            "util.cpp",
            "math_utils.cpp",
            "logger.cpp",
        ];
        for src in &expected_sources {
            let found = include_files.source_files.iter().any(|f| f.contains(src));
            assert!(found, "Source file {} should be found", src);
        }
    }
}
