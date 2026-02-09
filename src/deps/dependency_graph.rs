use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use super::headers_utils::{get_header, is_main_file, is_std_header};
use crate::state::filetype::SourceFileType;
use crate::state::structure::ProjectStructure;

/// Returns the `SourceFileType` for a given file path based on its extension.
/// Unknown extensions are mapped to `SourceFileType::Unknown`.

//FIXME: need to use the Path/PathBuf type here instead of &str
fn get_file_type(file_path: &str) -> SourceFileType {
    std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(SourceFileType::from_extension)
        .unwrap_or(SourceFileType::Unknown)
}

/// Represents a node in the dependency graph, corresponding to a file (source/header).
/// Stores file path, type, dependencies, parent, and implementation file (for headers).

#[derive(Debug, Clone)]
struct FileNode {
    file_path: PathBuf,
    file_type: SourceFileType,
    impl_file: Option<usize>,
    dependencies: Option<Vec<usize>>,
    #[allow(dead_code)]
    parent: Option<usize>,
}

/// Represents the dependency graph of a C/C++ project.
/// Tracks all files (sources/headers), their dependencies, and include directories.

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    main_file_idx: usize,
    nodes: Vec<FileNode>,
    file_path_to_idx: HashMap<PathBuf, usize>,
    includes_dir: HashSet<PathBuf>,
}

impl DependencyGraph {
    // assumes that projectStructure has cannonicalized paths

    /// Constructs a new `DependencyGraph` for the given main file and project structure.
    ///
    /// # Arguments
    /// * `main_file_path` - Path to the main source file (entry point).
    /// * `project_structure` - Project structure for resolving headers and includes.
    ///
    /// # Returns
    /// * `DependencyGraph` - The constructed dependency graph rooted at the main file.
    pub fn new(main_file_path: &str, project_structure: &ProjectStructure) -> Self {
        let main_file_idx = 0usize;
        let file_type = get_file_type(main_file_path);
        let mut nodes: Vec<FileNode> = Vec::new();
        let mut file_path_to_idx: HashMap<PathBuf, usize> = HashMap::new();
        let main_file_node = FileNode {
            file_path: fs::canonicalize(main_file_path)
                .unwrap_or_else(|_| PathBuf::from(main_file_path)),
            file_type,
            impl_file: None,
            dependencies: None,
            parent: None,
        };

        nodes.push(main_file_node);

        file_path_to_idx.insert(
            std::fs::canonicalize(main_file_path).unwrap_or_else(|_| PathBuf::from(main_file_path)),
            main_file_idx,
        );
        let file_headers = get_header(main_file_path);
        for header in file_headers {
            if is_main_file(std::path::Path::new(&header)) {
                continue;
            }

            let header_file_paths_opts = project_structure.get_header_file_paths(&header);

            if let Some(header_file_coll) = header_file_paths_opts {
                for header_file_path in header_file_coll {
                    let header_file_idx;

                    if let Some(idx) = file_path_to_idx.get(header_file_path.as_path()) {
                        header_file_idx = *idx;
                    } else {
                        header_file_idx = Self::create_node_rec(
                            main_file_idx,
                            header_file_path.to_str().unwrap_or(""),
                            project_structure,
                            &mut nodes,
                            &mut file_path_to_idx,
                        );
                    }
                    match &mut nodes[main_file_idx].dependencies {
                        Some(deps) => deps.push(header_file_idx),
                        None => {
                            nodes[main_file_idx].dependencies = Some(vec![header_file_idx]);
                        }
                    }
                }
            }
        }
        return Self {
            main_file_idx,
            nodes,
            file_path_to_idx,
            includes_dir: project_structure.get_include_directories(),
        };
    }

    /// Recursively creates a node in the dependency graph for a given file.
    ///
    /// Ensures each file is only processed once (by canonical path), and builds out
    /// dependency and implementation relationships for headers and sources.
    ///
    /// # Arguments
    /// * `parent_idx` - Index of the parent node in the graph.
    /// * `file_path` - Path to the file to add.
    /// * `project_structure` - Project structure for header/source lookup.
    /// * `nodes` - Mutable reference to the vector of `FileNode` objects (the graph).
    /// * `file_to_idx` - Mutable map from canonical file path to node index.
    ///
    /// # Returns
    /// * `usize` - Index of the created or existing node in the graph.
    fn create_node_rec(
        parent_idx: usize,
        file_path: &str,
        project_structure: &ProjectStructure,
        nodes: &mut Vec<FileNode>,
        file_to_idx: &mut HashMap<PathBuf, usize>,
    ) -> usize {
        let canonical_path =
            std::fs::canonicalize(file_path).unwrap_or_else(|_| PathBuf::from(file_path));
        // Check if already processed to avoid infinite recursion
        if let Some(&existing_idx) = file_to_idx.get(&canonical_path) {
            return existing_idx;
        }
        // Reserve index BEFORE recursing to prevent infinite loops
        let current_idx = nodes.len();
        file_to_idx.insert(canonical_path.clone(), current_idx);
        let file_node = FileNode {
            file_path: canonical_path,
            file_type: get_file_type(file_path),
            impl_file: None,
            dependencies: None,
            parent: Some(parent_idx),
        };
        nodes.push(file_node);
        // If this is a header, try to find its implementation file and add it
        if nodes[current_idx].file_type == SourceFileType::Header {
            if let Some(impl_file_path) =
                project_structure.get_header_impl_file(std::path::Path::new(file_path))
            {
                let impl_file_idx = Self::create_node_rec(
                    current_idx,
                    impl_file_path.to_str().unwrap_or(""),
                    project_structure,
                    nodes,
                    file_to_idx,
                );
                nodes[current_idx].impl_file = Some(impl_file_idx);
            }
        }
        // Add dependencies for all headers included by this file
        let file_headers = get_header(file_path);
        for header in file_headers {
            // Skip standard library headers
            if is_std_header(&header) {
                continue;
            }
            let header_file_paths_opts = project_structure.get_header_file_paths(&header);
            if let Some(header_file_paths) = header_file_paths_opts {
                for header_file_path in header_file_paths {
                    let header_file_idx =
                        if let Some(&idx) = file_to_idx.get(header_file_path.as_path()) {
                            idx
                        } else {
                            Self::create_node_rec(
                                current_idx,
                                header_file_path.to_str().unwrap_or(""),
                                project_structure,
                                nodes,
                                file_to_idx,
                            )
                        };
                    match &mut nodes[current_idx].dependencies {
                        Some(deps) => deps.push(header_file_idx),
                        None => {
                            nodes[current_idx].dependencies = Some(vec![header_file_idx]);
                        }
                    }
                }
            }
        }
        current_idx
    }

    /// Returns the list of include directories for the project as strings.
    pub fn get_main_includes(&self) -> Vec<String> {
        return self
            .includes_dir
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
    }

    /// Returns all header files included (directly or indirectly) from the given file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to analyze.
    ///
    /// # Returns
    /// * `Vec<String>` - List of included header file paths as strings.
    pub fn get_includes_from(&self, file_path: &str) -> Vec<String> {
        let canonical_path =
            fs::canonicalize(file_path).unwrap_or_else(|_| PathBuf::from(file_path));
        let file_node_idx_opt = self.file_path_to_idx.get(&canonical_path);
        if file_node_idx_opt.is_none() {
            return Vec::new();
        }
        let file_node_idx = *file_node_idx_opt.unwrap();
        let mut includes = HashSet::new();
        let mut queue = Vec::new();
        queue.push(file_node_idx);
        while let Some(current_idx) = queue.pop() {
            let current_node = &self.nodes[current_idx];
            if current_node.file_type == SourceFileType::Source
                || current_node.file_type == SourceFileType::Entry
                || includes.contains(current_node.file_path.to_string_lossy().as_ref())
            {
                continue;
            }
            if let Some(deps) = &current_node.dependencies {
                for dep_idx in deps {
                    let dep_node = &self.nodes[*dep_idx];
                    includes.insert(dep_node.file_path.to_string_lossy().to_string());
                    queue.push(*dep_idx);
                }
            }
        }
        includes.into_iter().collect()
    }

    /// Returns all source files (non-header) that are dependencies of the given file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to analyze.
    ///
    /// # Returns
    /// * `Vec<String>` - List of dependent source file paths as strings.
    pub fn get_source_files_from(&self, file_path: &str) -> Vec<String> {
        let canonical_path =
            fs::canonicalize(file_path).unwrap_or_else(|_| PathBuf::from(file_path));
        let file_node_idx_opt = self.file_path_to_idx.get(&canonical_path);
        if file_node_idx_opt.is_none() {
            return Vec::new();
        }
        let file_node_idx = *file_node_idx_opt.unwrap();
        let mut source_files = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = Vec::new();
        queue.push(file_node_idx);
        while let Some(current_idx) = queue.pop() {
            if visited.contains(&current_idx) {
                continue;
            }
            visited.insert(current_idx);
            let current_node = &self.nodes[current_idx];
            if current_node.file_type != SourceFileType::Header
                && current_node.file_type != SourceFileType::Unknown
            {
                source_files.insert(current_node.file_path.to_string_lossy().to_string());
            }
            if let Some(deps) = &current_node.dependencies {
                for dep_idx in deps {
                    if !visited.contains(dep_idx) {
                        queue.push(*dep_idx);
                    }
                }
            }
            if let Some(impl_idx) = current_node.impl_file {
                if !visited.contains(&impl_idx) {
                    queue.push(impl_idx);
                }
            }
        }
        source_files.into_iter().collect()
    }

    /// Returns all source files (non-header) that are dependencies of the main file.
    pub fn get_all_source_files(&self) -> Vec<String> {
        let main_file_path = &self.nodes[self.main_file_idx].file_path;
        let main_file_path_str = main_file_path.to_str().expect("NOT A VALID PATH");

        return self.get_source_files_from(main_file_path_str);
    }

    #[cfg(any(test, debug_assertions))]
    pub fn debug_print(&self) {
        for (idx, node) in self.nodes.iter().enumerate() {
            println!("Node {}: {:?}", idx, node.file_path);
            if let Some(deps) = &node.dependencies {
                for dep_idx in deps {
                    println!("  -> Dep: {:?}", self.nodes[*dep_idx].file_path);
                }
            }
            if let Some(impl_idx) = node.impl_file {
                println!("  -> Impl: {:?}", self.nodes[impl_idx].file_path);
            }
        }
    }
}

//________________________TESTS________________________//

#[cfg(test)]
mod tests {}
