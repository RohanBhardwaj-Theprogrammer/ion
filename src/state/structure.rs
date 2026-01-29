//! Project structure indexing and file queries.
//!
//! This module scans a project root directory and builds an in-memory tree of files.
//! It is used by commands like `run`, `build`, and dependency analysis.
//!
//! **Path conventions**
//! - `ProjectStructure` uses `Configs::get_project_root()` as the root directory.
//! - `FileNode::path` is stored as an absolute path (canonicalized when possible).
//! - Some APIs accept either root-relative paths (like `src/main.cpp`) or absolute paths.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::filetype::SourceFileType;
use crate::config::Configs;

/// A single discovered file in the project.
///
/// This is the “leaf” type returned by most query methods on [`ProjectStructure`].
///
/// Notes:
/// - The stored path is absolute (canonicalized when possible).
/// - The file type is inferred from the extension via [`SourceFileType::from_extension`].

#[derive(Debug, Clone)]
pub struct FileNode {
    file_type: SourceFileType,
    path: PathBuf, // absolute (usually canonical) path
}

impl FileNode {
    fn new(file_type: SourceFileType, path: PathBuf) -> Self {
        FileNode { file_type, path }
    }

    /// Returns the detected [`SourceFileType`].
    pub fn get_type(&self) -> &SourceFileType {
        &self.file_type
    }

    /// Returns the stored absolute path of the file.
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns `true` if this node is a header file.
    pub fn is_header(&self) -> bool {
        matches!(self.file_type, SourceFileType::Header)
    }

    /// Returns the directory containing this file.
    pub fn dir_name(&self) -> Option<&Path> {
        self.path.parent()
    }
}

#[test]
fn test_file_node_ops() {
    let file_node = FileNode::new(SourceFileType::Header, PathBuf::from("include/main.h"));
    assert_eq!(file_node.get_type(), &SourceFileType::Header);
    assert_eq!(file_node.get_path(), &PathBuf::from("include/main.h"));
    assert!(file_node.is_header());
    assert_eq!(file_node.dir_name(), Some(Path::new("include")));
}

/// Internal directory node used to build a tree of files.
///
/// This type is an implementation detail of [`ProjectStructure`].
/// The `path` is stored root-relative.

#[derive(Debug, Clone)]
struct DirNode {
    path: PathBuf, // root-relative path of this directory

    files: HashMap<String, FileNode>,  // key is file name
    subdirs: HashMap<String, DirNode>, // key is dir name
}

impl DirNode {
    fn new(path: PathBuf) -> Self {
        DirNode {
            path,
            files: HashMap::new(),
            subdirs: HashMap::new(),
        }
    }

    /// Add a file to this directory or its subdirectories.
    ///
    /// `path_components` should be a split path *relative to this `DirNode`*.
    /// For example, to add `src/utils/helper.h` under the `src` directory node,
    /// pass `path_components = &["utils", "helper.h"]`.
    pub fn add_file(&mut self, path_components: &[&str], file_node: FileNode) {
        if path_components.is_empty() {
            return;
        }
        let first_component = path_components[0];
        if path_components.len() == 1 {
            // This is the file to add
            self.files.insert(first_component.to_string(), file_node);
        } else {
            // This is a directory, recurse into it
            let subdir = self
                .subdirs
                .entry(first_component.to_string())
                .or_insert_with(|| DirNode::new(self.path.join(first_component)));
            subdir.add_file(&path_components[1..], file_node);
        }
    }

    fn get_file_by_path(&self, path_components: &[&str]) -> Option<&FileNode> {
        if path_components.is_empty() {
            return None;
        }
        let first_component = path_components[0];
        if path_components.len() == 1 {
            // This is the file to get
            self.files.get(first_component)
        } else {
            // This is a directory, recurse into it
            let subdir = self.subdirs.get(first_component)?;
            subdir.get_file_by_path(&path_components[1..])
        }
    }

    #[allow(dead_code)]
    fn get_subdir_by_path(&self, path_components: &[&str]) -> Option<&DirNode> {
        if path_components.is_empty() {
            return Some(self);
        }
        let first_component = path_components[0];
        let subdir = self.subdirs.get(first_component)?;
        subdir.get_subdir_by_path(&path_components[1..])
    }

    /// Recursively collect all files from this directory and subdirectories.
    fn collect_files_recursive<'a>(&'a self, result: &mut Vec<&'a FileNode>) {
        for file in self.files.values() {
            result.push(file);
        }
        for subdir in self.subdirs.values() {
            subdir.collect_files_recursive(result);
        }
    }

    /// Recursively collect all files matching `file_type`.
    pub fn collect_files_for_type<'a>(
        &'a self,
        file_type: &'a SourceFileType,
        result: &mut Vec<&'a FileNode>,
    ) {
        for file in self.files.values() {
            if file.get_type() == file_type {
                result.push(file);
            }
        }
        for subdir in self.subdirs.values() {
            subdir.collect_files_for_type(file_type, result);
        }
    }
}

#[test]
fn test_dir_node_ops() {
    let mut dir_node = DirNode::new(PathBuf::from("src"));
    let file_node1 = FileNode::new(SourceFileType::Source, PathBuf::from("src/main.cpp"));
    let file_node2 = FileNode::new(SourceFileType::Header, PathBuf::from("src/utils/helper.h"));
    dir_node.add_file(&["main.cpp"], file_node1);
    dir_node.add_file(&["utils", "helper.h"], file_node2);
    let mut collected_files = Vec::new();
    dir_node.collect_files_recursive(&mut collected_files);
    assert_eq!(collected_files.len(), 2);

    assert_eq!(dir_node.files.len(), 1);
    assert!(dir_node.subdirs.contains_key("main.cpp") == false);

    assert_eq!(dir_node.subdirs["utils"].files.len(), 1);
}

/// In-memory index of a project directory.
///
/// The structure is built by scanning the configured root directory and recording files.
/// Directories excluded via [`Configs::is_excluded_dir`] are skipped.
///
/// Example (uses the crate test project):
/// ```no_run
/// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
///
/// let configs = Configs::test_config(None);
/// let structure = ProjectStructure::new(&configs);
///
/// // Query some basic facts
/// assert!(!structure.get_all_files().is_empty());
/// ```

#[derive(Debug, Clone)]
pub struct ProjectStructure {
    entry_file: Option<FileNode>,
    #[allow(dead_code)]
    dot_dir: Option<DirNode>,
    root_path: PathBuf,               // canonical root path
    dirs: HashMap<String, DirNode>, // key is folder name (not full path), folders directly under root
    files: HashMap<String, FileNode>, // key is file name, files directly under root
}

impl ProjectStructure {
    /// Resolve a path under the project root.
    ///
    /// This is an internal helper that:
    /// - accepts root-relative or absolute paths,
    /// - ensures they exist,
    /// - ensures they do not escape the configured project root,
    /// - returns a *root-relative* path.
    fn resolve_under_root(&self, path: &Path) -> Result<PathBuf, String> {
        // Handle "." as root itself
        if path == Path::new(".") {
            return Ok(PathBuf::from("."));
        }

        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root_path.join(path)
        };

        if !absolute_path.exists() {
            return Err("Path doesn't exist under the project directory".to_string());
        }

        // Get root-relative path
        let root_relative_path = absolute_path
            .strip_prefix(&self.root_path)
            .map_err(|_| "Path is not under project root".to_string())?;

        Ok(root_relative_path.to_path_buf())
    }

    /// Canonicalize a path after validating it is under the project root.
    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, String> {
        let root_relative_path = self.resolve_under_root(path)?;
        let absolute_path = self.root_path.join(&root_relative_path);
        let canonical_path = fs::canonicalize(&absolute_path).map_err(|e| e.to_string())?;
        Ok(canonical_path)
    }

    /// Build a new [`ProjectStructure`] by scanning the project root.
    ///
    /// - Uses `configs.get_project_root()` as the root.
    /// - Skips excluded directories according to `configs.is_excluded_dir()`.
    /// - Builds an internal directory tree for efficient lookup.
    /// - Also indexes the tool directory `.{PROGRAM_NAME}` (if present) into `dot_dir`.
    ///
    /// Example:
    /// ```no_run
    /// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
    ///
    /// let configs = Configs::test_config(None);
    /// let structure = ProjectStructure::new(&configs);
    ///
    /// // Find candidate entry points
    /// let entries = structure.get_entry_files();
    /// assert!(!entries.is_empty());
    /// ```
    pub fn new(configs: &Configs) -> Self {
        let root_path = fs::canonicalize(configs.get_project_root())
            .unwrap_or_else(|_| configs.get_project_root().clone());

        let mut dirs: HashMap<String, DirNode> = HashMap::new();
        let mut files: HashMap<String, FileNode> = HashMap::new();

        let dot_dir_path = root_path.join(format!(".{}", crate::constants::PROGRAM_NAME));
        let mut dot_dir: DirNode = DirNode::new(PathBuf::from(format!(
            ".{}",
            crate::constants::PROGRAM_NAME
        )));

        let project_nodes = get_files_node(root_path.clone(), configs);

        for file_path in &project_nodes {
            let root_relative_path = match file_path.strip_prefix(&root_path) {
                Ok(p) => p,
                Err(_) => continue, // Skip files not under root
            };
            let path_components: Vec<&str> = root_relative_path
                .iter()
                .filter_map(|os_str| os_str.to_str())
                .collect();

            if path_components.is_empty() {
                continue;
            }

            let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let file_type = SourceFileType::from_extension(ext);
            // Store root-relative path, not absolute
            let file_node = FileNode::new(file_type, file_path.clone());

            if path_components.len() == 1 {
                // File directly under root
                files.insert(path_components[0].to_string(), file_node);
            } else {
                // File in a subdirectory - key is the folder name directly under root
                let dir_name = path_components[0].to_string();
                let dir_node = dirs
                    .entry(dir_name.clone())
                    .or_insert_with(|| DirNode::new(PathBuf::from(&dir_name)));
                dir_node.add_file(&path_components[1..], file_node);
            }
        }

        // Process dot_dir (excluded directory like .cbuild)
        if dot_dir_path.exists() {
            let dot_dir_nodes = get_files_node_for_dot_dir(dot_dir_path.clone());
            for file_path in &dot_dir_nodes {
                let relative_path = match file_path.strip_prefix(&dot_dir_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let path_components: Vec<&str> = relative_path
                    .iter()
                    .filter_map(|os_str| os_str.to_str())
                    .collect();

                if path_components.is_empty() {
                    continue;
                }

                let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let file_type = SourceFileType::from_extension(ext);
                // Store path relative to dot_dir
                let file_node = FileNode::new(file_type, file_path.clone());

                dot_dir.add_file(&path_components, file_node);
            }
        }

        ProjectStructure {
            entry_file: None,
            dot_dir: Some(dot_dir),
            root_path,
            dirs,
            files,
        }
    }

    /// Create an empty structure (no root, no files).
    ///
    /// This is used when the project root cannot be discovered.
    pub fn none() -> Self {
        ProjectStructure {
            entry_file: None,
            dot_dir: None,
            root_path: PathBuf::new(),
            dirs: HashMap::new(),
            files: HashMap::new(),
        }
    }

    /// Returns the configured project root path.
    pub fn get_root_path(&self) -> &PathBuf {
        &self.root_path
    }

    /// Create a scoped view rooted at `subfolder`.
    ///
    /// The returned [`ProjectStructureAsRef`] behaves like a lightweight cursor that
    /// interprets relative paths as being *relative to* that subfolder.
    ///
    /// Example:
    /// ```no_run
    /// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
    ///
    /// let configs = Configs::test_config(None);
    /// let ps = ProjectStructure::new(&configs);
    ///
    /// let src = ps.to("test_sample/src")?;
    /// assert!(src.get_file_by_path(std::path::Path::new("main.cpp")).is_some());
    /// # Ok::<(), String>(())
    /// ```
    pub fn to<P: AsRef<Path>>(&self, subfolder: P) -> Result<ProjectStructureAsRef<'_>, String> {
        let subfolder = subfolder.as_ref();

        let root_relative = if subfolder == Path::new(".") {
            PathBuf::from(".")
        } else if subfolder.is_absolute() {
            self.resolve_under_root(subfolder)?
        } else {
            let abs = self.root_path.join(subfolder);
            if !abs.exists() {
                return Err("Subfolder does not exist under the project root".to_string());
            }
            subfolder.to_path_buf()
        };

        let abs = self.root_path.join(&root_relative);
        if !abs.is_dir() {
            return Err("Requested subfolder is not a directory".to_string());
        }

        Ok(ProjectStructureAsRef::new(self, root_relative))
    }

    /// Returns all source files (`.c`, `.cpp`, …) discovered in the project.
    pub fn get_source_file(&self) -> Vec<&FileNode> {
        let mut all_files = Vec::new();
        // Files directly under root
        for file in self.files.values() {
            if file.get_type() == &SourceFileType::Source {
                all_files.push(file);
            }
        }
        // Files in subdirectories
        for dir in self.dirs.values() {
            dir.collect_files_for_type(&SourceFileType::Source, &mut all_files);
        }
        all_files
    }

    /// Returns all header files (`.h`, `.hpp`, …) discovered in the project.
    pub fn get_header_file(&self) -> Vec<&FileNode> {
        let mut all_files = Vec::new();
        // Files directly under root
        for file in self.files.values() {
            if file.is_header() {
                all_files.push(file);
            }
        }
        // Files in subdirectories
        for dir in self.dirs.values() {
            dir.collect_files_for_type(&SourceFileType::Header, &mut all_files);
        }

        all_files
    }

    /// Find header files by name.
    ///
    /// `header_name` is matched as a *path suffix* (`Path::ends_with`), so values like
    /// `"logger.h"` will match `.../include/logger.h` and `.../src/logger.h`.
    ///
    /// Returns absolute paths.
    ///
    /// Example:
    /// ```no_run
    /// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
    ///
    /// let configs = Configs::test_config(None);
    /// let structure = ProjectStructure::new(&configs);
    ///
    /// let paths = structure.get_header_file_paths("logger.h").unwrap();
    /// assert!(!paths.is_empty());
    /// ```
    pub fn get_header_file_paths(&self, header_name: &str) -> Option<Vec<PathBuf>> {
        let header_files = self.get_header_file();

        let matches: Vec<PathBuf> = header_files
            .iter()
            .filter(|f| f.get_path().ends_with(header_name))
            .map(|f| f.get_path().clone())
            .collect();

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    /// Find an implementation file for a header.
    ///
    /// Heuristic: find the first source file whose stem starts with the header stem.
    /// For example `math_utils.h` may match `math_utils.cpp`.
    ///
    /// Returns an absolute path.
    pub fn get_header_impl_file(&self, header_path: &Path) -> Option<PathBuf> {
        let stem = header_path.file_stem()?.to_str()?;

        let source_files = self.get_source_file();
        for source in source_files {
            if let Some(source_stem) = source.get_path().file_stem().and_then(|s| s.to_str()) {
                if source_stem.starts_with(stem) {
                    return Some(source.get_path().clone());
                }
            }
        }
        None
    }

    /// Returns all include directories (directories containing header files).
    ///
    /// The returned paths are absolute.
    pub fn get_include_directories(&self) -> HashSet<PathBuf> {
        let header_files = self.get_header_file();
        let mut dirs: HashSet<PathBuf> = HashSet::new();
        for header in header_files {
            if let Some(parent) = header.get_path().parent() {
                dirs.insert(parent.to_path_buf());
            }
        }
        dirs
    }

    /// Returns `true` if `path` exists and is under the project root.
    ///
    /// `path` may be absolute or root-relative.
    ///
    /// Example:
    /// ```no_run
    /// use std::path::Path;
    /// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
    ///
    /// let configs = Configs::test_config(None);
    /// let structure = ProjectStructure::new(&configs);
    /// assert!(structure.exists(Path::new("src/main.cpp")));
    /// ```
    pub fn exists(&self, path: &Path) -> bool {
        self.resolve_under_root(path).is_ok()
    }

    /// Add an existing file to the structure.
    ///
    /// This updates the in-memory index and returns the root-relative path.
    /// The file must already exist on disk.
    ///
    /// Note: this does **not** create the file on disk.
    pub fn create_file(
        &mut self,
        path: &Path,
        file_type: SourceFileType,
    ) -> Result<PathBuf, String> {
        let root_relative_path = self.resolve_under_root(path)?;

        let path_components: Vec<&str> = root_relative_path
            .iter()
            .filter_map(|os_str| os_str.to_str())
            .collect();

        if path_components.is_empty() {
            return Err("Empty path components".to_string());
        }

        let file_node = FileNode::new(file_type, self.canonicalize_path(path)?.to_path_buf());

        if path_components.len() == 1 {
            // File directly under root
            self.files.insert(path_components[0].to_string(), file_node);
        } else {
            let dir_name = path_components[0].to_string();
            let dir_node = self
                .dirs
                .entry(dir_name.clone())
                .or_insert_with(|| DirNode::new(PathBuf::from(&dir_name)));
            dir_node.add_file(&path_components[1..], file_node);
        }

        Ok(root_relative_path)
    }

    /// Create a directory on disk (under the project root) and add it to the structure.
    ///
    /// `path` is treated as root-relative.
    ///
    /// Example:
    /// ```no_run
    /// use std::path::Path;
    /// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
    ///
    /// let configs = Configs::test_config(None);
    /// let mut structure = ProjectStructure::new(&configs);
    /// let created = structure.create_dir(Path::new("build/generated"))?;
    /// assert!(created.ends_with("build/generated"));
    /// # Ok::<(), String>(())
    /// ```
    pub fn create_dir(&mut self, path: &Path) -> Result<PathBuf, String> {
        // Handle "." as root
        if path == Path::new(".") {
            return Ok(PathBuf::from("."));
        }

        let absolute_path = self.root_path.join(path);

        // Create the directory on disk if it doesn't exist
        fs::create_dir_all(&absolute_path).map_err(|e| e.to_string())?;

        // Get root-relative path
        let root_relative_path = absolute_path
            .strip_prefix(&self.root_path)
            .map_err(|_| "Path is not under project root".to_string())?
            .to_path_buf();

        let path_components: Vec<&str> = root_relative_path
            .iter()
            .filter_map(|os_str| os_str.to_str())
            .collect();

        if path_components.is_empty() {
            return Ok(PathBuf::from(".")); // It's the root itself
        }

        // Insert into the directory structure
        let dir_name = path_components[0].to_string();
        let dir_node = self
            .dirs
            .entry(dir_name.clone())
            .or_insert_with(|| DirNode::new(PathBuf::from(&dir_name)));

        let mut current_dir = dir_node;
        for component in &path_components[1..] {
            current_dir = current_dir
                .subdirs
                .entry(component.to_string())
                .or_insert_with(|| DirNode::new(current_dir.path.join(component)));
        }

        Ok(root_relative_path)
    }

    /// Get all files with a specific extension (case-insensitive, e.g., ".cpp" or "cpp")
    /// Argument `extension` can be with or without leading dot.
    ///
    /// Example:
    /// ```no_run
    /// use c_cpp_build_system_n_pkg_manager::{config::Configs, state::ProjectStructure};
    ///
    /// let configs = Configs::test_config(None);
    /// let structure = ProjectStructure::new(&configs);
    /// let cpp_files = structure.get_files_with_extension(".cpp");
    /// assert!(!cpp_files.is_empty());
    /// ```
    pub fn get_files_with_extension(&self, extension: &str) -> Vec<&FileNode> {
        let normalized = extension
            .trim()
            .strip_prefix('.')
            .unwrap_or(extension.trim());

        let mut all_files = Vec::new();
        // Files directly under root
        for file in self.files.values() {
            all_files.push(file);
        }
        // Files in subdirectories
        for dir in self.dirs.values() {
            dir.collect_files_recursive(&mut all_files);
        }
        all_files
            .into_iter()
            .filter(|file| {
                if let Some(ext) = file.get_path().extension() {
                    ext.to_string_lossy().eq_ignore_ascii_case(normalized)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Remove a file from both the in-memory structure and the filesystem.
    ///
    /// `path` may be absolute or root-relative.
    pub fn remove_file(&mut self, path: &Path) -> Result<(), String> {
        let root_relative_path = self.resolve_under_root(path)?;

        let path_components: Vec<&str> = root_relative_path
            .iter()
            .filter_map(|os_str| os_str.to_str())
            .collect();

        if path_components.is_empty() {
            return Err("Empty path components".to_string());
        }

        let file_name = path_components.last().unwrap();

        if path_components.len() == 1 {
            // File directly under root
            if self.files.remove(*file_name).is_some() {
                fs::remove_file(self.canonicalize_path(path)?).map_err(|e| e.to_string())?; // remove from disk
                return Ok(());
            } else {
                return Err("File not found".to_string());
            }
        }

        // Navigate to the appropriate DirNode
        let dir_name = path_components[0];
        let dir_node = self.dirs.get_mut(dir_name).ok_or("Directory not found")?;

        let mut current_dir = dir_node;
        for component in &path_components[1..path_components.len() - 1] {
            current_dir = current_dir
                .subdirs
                .get_mut(*component)
                .ok_or("Directory not found in path")?;
        }

        // Remove the file
        if current_dir.files.remove(*file_name).is_some() {
            fs::remove_file(self.canonicalize_path(path)?).map_err(|e| e.to_string())?; // remove from disk
            Ok(())
        } else {
            Err("File not found".to_string())
        }
    }

    /// Remove a directory from both the in-memory structure and the filesystem.
    ///
    /// `path` may be absolute or root-relative.
    ///
    /// Warning: this uses `fs::remove_dir` (not recursive) to match the existing behavior.
    pub fn remove_dir(&mut self, path: &Path) -> Result<(), String> {
        let root_relative_path = self.resolve_under_root(path)?;

        let path_components: Vec<&str> = root_relative_path
            .iter()
            .filter_map(|os_str| os_str.to_str())
            .collect();

        if path_components.is_empty() {
            return Err("Cannot remove root directory".to_string());
        }

        let dir_name = path_components.last().unwrap();

        if path_components.len() == 1 {
            // Directory directly under root
            if self.dirs.remove(*dir_name).is_some() {
                fs::remove_dir(self.canonicalize_path(path)?).map_err(|e| e.to_string())?; // remove from disk
                return Ok(());
            } else {
                return Err("Directory not found".to_string());
            }
        }

        // Navigate to the parent DirNode
        let first_dir = path_components[0];
        let dir_node = self.dirs.get_mut(first_dir).ok_or("Directory not found")?;

        let mut current_dir = dir_node;
        for component in &path_components[1..path_components.len() - 1] {
            current_dir = current_dir
                .subdirs
                .get_mut(*component)
                .ok_or("Directory not found in path")?;
        }

        // Remove the directory
        if current_dir.subdirs.remove(*dir_name).is_some() {
            fs::remove_dir(self.canonicalize_path(path)?).map_err(|e| e.to_string())?; // remove from disk
            Ok(())
        } else {
            Err("Directory not found".to_string())
        }
    }

    // not used currently
    #[allow(dead_code)]
    fn truncate(&mut self) -> Result<PathBuf, String> {
        // Remove all entries under the project root but do not remove the root itself.
        if !self.root_path.is_dir() {
            return Err("Project root does not exist".to_string());
        }

        for entry in fs::read_dir(&self.root_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            // Ensure we never touch paths outside the project root
            if !path.starts_with(&self.root_path) {
                return Err("Detected path outside project root during truncate".to_string());
            }

            if path.is_dir() {
                fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
        }

        // Reset in-memory structure
        self.dirs.clear();
        self.files.clear();

        Ok(PathBuf::from("."))
    }

    /// Returns all files in the project.
    pub fn get_all_files(&self) -> Vec<&FileNode> {
        let mut all_files = Vec::new();
        // Files directly under root
        for file in self.files.values() {
            all_files.push(file);
        }
        // Files in subdirectories
        for dir in self.dirs.values() {
            dir.collect_files_recursive(&mut all_files);
        }
        all_files
    }

    /// Get a file by its path.
    ///
    /// `path` may be root-relative or absolute.
    pub fn get_file_by_path(&self, path: &Path) -> Option<&FileNode> {
        let root_relative = self.resolve_under_root(path).ok()?;
        let path_components: Result<Vec<&str>, _> = root_relative
            .iter()
            .map(|os_str| os_str.to_str().ok_or("Non-UTF8 path component"))
            .collect();

        let path_components = path_components.ok()?; // or handle the error as you wish

        if path_components.is_empty() {
            return None;
        }

        if path_components.len() == 1 {
            // File directly under root
            return self.files.get(path_components[0]);
        }
        let dir_name = path_components[0];
        let dir_node = self.dirs.get(dir_name)?;
        dir_node.get_file_by_path(&path_components[1..])
    }

    /// Get a file by its filename (e.g., "main.cpp")
    pub fn get_file_by_name(&self, name: &str) -> Option<&FileNode> {
        self.get_all_files().into_iter().find(|file| {
            file.get_path()
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|fname| fname == name)
        })
    }

    /// Get all object files (.o, .obj)
    pub fn get_object_files(&self) -> Vec<&FileNode> {
        self.get_all_files()
            .into_iter()
            .filter(|file| matches!(file.get_type(), SourceFileType::Object))
            .collect()
    }

    /// Get all entry files (files with main function) directly under src/
    ///
    /// This only checks files directly inside the `src/` directory (not nested subdirectories).
    pub fn get_entry_files(&self) -> Vec<&FileNode> {
        if let Some(ref entry) = self.entry_file {
            return vec![entry];
        }

        let mut entry_files = Vec::new();
        if let Some(src_dir) = self.dirs.get("src") {
            use crate::deps::headers_utils::is_main_file;
            for file in src_dir.files.values() {
                if is_main_file(file.get_path()) {
                    entry_files.push(file);
                }
            }
        }

        entry_files
    }

    /// Returns all include directories (directories containing header files).
    ///
    /// The returned paths are absolute.
    pub fn get_include_dirs(&self) -> HashSet<PathBuf> {
        self.get_header_file()
            .into_iter()
            .filter_map(|header| {
                header.get_path().parent().map(|p| {
                    if p.as_os_str().is_empty() {
                        PathBuf::from(".")
                    } else {
                        p.to_path_buf()
                    }
                })
            })
            .collect()
    }

    // BUG: inefficient, improve later
    // REVIEW: we are hrere at now
    /// Get all files of a specific type
    pub fn get_files_by_type<'a>(&'a self, file_type: &'a SourceFileType) -> Vec<&'a FileNode> {
        let mut all_files = Vec::new();
        // Files directly under root
        for file in self.files.values() {
            if file.get_type() == file_type {
                all_files.push(file);
            }
        }
        // Files in subdirectories
        for dir in self.dirs.values() {
            dir.collect_files_for_type(file_type, &mut all_files);
        }
        all_files
    }

    //HACK: inefficient, improve later
    // REVIEW: need to review the logic here
    /// Get all files in a specific directory (root-relative path)
    ///
    /// `dir_path` may be root-relative or absolute. The returned file paths are absolute.
    pub fn get_files_in_dir(&self, dir_path: &Path) -> Vec<&FileNode> {
        let root_relative_dir = match self.resolve_under_root(dir_path) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        // Handle "." as root
        let is_root =
            root_relative_dir == Path::new(".") || root_relative_dir.as_os_str().is_empty();

        let mut result = Vec::new();
        for file in self.get_all_files() {
            let Some(parent_abs) = file.get_path().parent() else {
                continue;
            };

            // Compare using root-relative directory paths.
            // `file.get_path()` is stored as an absolute/canonical path.
            let parent_rel = match parent_abs.strip_prefix(&self.root_path) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let parent_is_root = parent_rel.as_os_str().is_empty();
            if (is_root && parent_is_root) || parent_rel == root_relative_dir {
                result.push(file);
            }
        }
        result
    }

    // TODO: make it more efficient by caching all files in a Vec
    pub fn get_by_index(&self, index: usize) -> Option<&FileNode> {
        let all_files = self.get_all_files();
        if index < all_files.len() {
            Some(all_files[index])
        } else {
            None
        }
    }
    // TODO: make it more efficient by caching all files in a Vec
    pub fn get_by_index_from(&self, path: &Path, index: usize) -> Option<&FileNode> {
        let files_in_dir = self.get_files_in_dir(path);
        if index < files_in_dir.len() {
            Some(files_in_dir[index])
        } else {
            None
        }
    }

    /// Get the dot_dir (e.g., .cbuild) for accessing excluded program files.
    ///
    /// This returns an internal type and is primarily meant for internal/debug usage.
    #[allow(private_interfaces)]
    pub fn get_dot_dir(&self) -> Option<&DirNode> {
        self.dot_dir.as_ref()
    }

    #[cfg(any(debug_assertions, test))]
    #[allow(dead_code)]
    pub fn debug_print(&self) {
        use crate::utils::path_utils;

        fn pretty_abs_display(path: &Path) -> String {
            path_utils::normalize_path_for_display(path)
        }

        fn file_type_short(t: &SourceFileType) -> &'static str {
            match t {
                SourceFileType::Header => "Header",
                SourceFileType::Source => "Source",
                SourceFileType::Entry => "Entry",
                SourceFileType::Unknown => "Unknown",
                SourceFileType::Object => "Object",
                SourceFileType::Binary => "Binary",
                SourceFileType::Dll => "Dll",
            }
        }

        fn rel_display(root: &Path, path: &Path) -> String {
            path.strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string())
        }

        fn print_dirnode(root_abs: &Path, name: &str, node: &DirNode, prefix: &str, is_last: bool) {
            let branch = if is_last { "└── " } else { "├── " };
            // Directory nodes are stored root-relative.
            let dir_rel = node.path.to_string_lossy().to_string();
            let dir_abs = pretty_abs_display(&root_abs.join(&node.path));
            println!(
                "{}{}{} (rel: {}, abs: {})",
                prefix, branch, name, dir_rel, dir_abs
            );

            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            let mut subdir_names: Vec<_> = node.subdirs.keys().cloned().collect();
            subdir_names.sort();
            let mut file_names: Vec<_> = node.files.keys().cloned().collect();
            file_names.sort();

            let total_children = subdir_names.len() + file_names.len();
            let mut printed = 0usize;

            for sub_name in subdir_names {
                printed += 1;
                let is_last_child = printed == total_children;
                let sub = &node.subdirs[&sub_name];
                print_dirnode(root_abs, &sub_name, sub, &child_prefix, is_last_child);
            }

            for file_name in file_names {
                printed += 1;
                let is_last_child = printed == total_children;
                let branch = if is_last_child {
                    "└── "
                } else {
                    "├── "
                };
                let f = &node.files[&file_name];
                let rel = rel_display(root_abs, f.get_path());
                let abs = pretty_abs_display(f.get_path());
                println!(
                    "{}{}{} [{}] (rel: {}, abs: {})",
                    child_prefix,
                    branch,
                    file_name,
                    file_type_short(f.get_type()),
                    rel,
                    abs
                );
            }
        }

        let root_abs = &self.root_path;

        // Header
        println!("ProjectStructure");
        println!("root: {}", pretty_abs_display(root_abs));
        println!(
            "top-level: {} dirs, {} files",
            self.dirs.len(),
            self.files.len()
        );
        println!("total files: {}", self.get_all_files().len());

        // Root-level listing
        let mut root_dir_names: Vec<_> = self.dirs.keys().cloned().collect();
        root_dir_names.sort();
        let mut root_file_names: Vec<_> = self.files.keys().cloned().collect();
        root_file_names.sort();

        println!(".");
        let total_root_children = root_dir_names.len() + root_file_names.len();
        let mut printed = 0usize;

        for dir_name in root_dir_names {
            printed += 1;
            let is_last = printed == total_root_children;
            let node = &self.dirs[&dir_name];
            print_dirnode(root_abs, &dir_name, node, "", is_last);
        }

        for file_name in root_file_names {
            printed += 1;
            let is_last = printed == total_root_children;
            let branch = if is_last { "└── " } else { "├── " };
            let f = &self.files[&file_name];
            let rel = rel_display(root_abs, f.get_path());
            let abs = pretty_abs_display(f.get_path());
            println!(
                "{}{} [{}] (rel: {}, abs: {})",
                branch,
                file_name,
                file_type_short(f.get_type()),
                rel,
                abs
            );
        }

        // Optional tool directory section
        if let Some(dot) = &self.dot_dir {
            println!();
            let tool_rel = dot.path.to_string_lossy().to_string();
            let tool_abs = pretty_abs_display(&root_abs.join(&dot.path));
            println!("tool dir: (rel: {}, abs: {})", tool_rel, tool_abs);
            let mut dot_files = Vec::new();
            dot.collect_files_recursive(&mut dot_files);
            println!("tool files: {}", dot_files.len());
        }
    }
}

/// Walk `dir_path` and return canonicalized file paths.
///
/// This is used specifically for the tool directory (e.g. `.{PROGRAM_NAME}`), so no exclusion
/// checks are applied.
fn get_files_node_for_dot_dir(dir_path: PathBuf) -> HashSet<PathBuf> {
    let mut file_set = HashSet::new();
    if !dir_path.is_dir() {
        return file_set;
    }

    let mut stack = vec![dir_path];
    while let Some(current_dir) = stack.pop() {
        let read_dir = match fs::read_dir(&current_dir) {
            Ok(rd) => rd,
            Err(_) => continue,
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                match fs::canonicalize(&path) {
                    Ok(canonical) => {
                        file_set.insert(canonical);
                    }
                    Err(_) => {
                        file_set.insert(path);
                    }
                }
            }
        }
    }
    file_set
}

/// Walk a directory tree, collecting files while skipping excluded directories.
///
/// Returned file paths are canonicalized when possible.
fn get_files_node(dir_path: PathBuf, configs: &Configs) -> HashSet<PathBuf> {
    let mut file_set = HashSet::new();
    if !dir_path.is_dir() {
        return file_set;
    }

    #[cfg(any(debug_assertions, test))]
    fn pretty_abs_display(path: &Path) -> String {
        crate::utils::path_utils::normalize_path_for_display(path)
    }

    let mut stack = vec![dir_path];
    #[cfg(any(debug_assertions, test))]
    let mut excludes_files = Vec::new();
    while let Some(current_dir) = stack.pop() {
        let read_dir = match fs::read_dir(&current_dir) {
            Ok(rd) => rd,
            Err(_) => {
                #[cfg(any(debug_assertions, test))]
                println!(
                    "DEBUG: Failed to read directory: {}",
                    pretty_abs_display(&current_dir)
                );
                continue;
            }
        };
        for entry in read_dir.flatten() {
            let path = entry.path();

            #[cfg(any(debug_assertions, test))]
            println!("DEBUG: Visiting path: {}", pretty_abs_display(&path));

            if path.is_dir() {
                if configs.is_excluded_dir(&path) {
                    #[cfg(any(debug_assertions, test))]
                    {
                        println!(
                            "DEBUG: Directory is excluded: {}",
                            pretty_abs_display(&path)
                        );
                        excludes_files.push(path.clone());
                    }
                    continue;
                }

                #[cfg(any(debug_assertions, test))]
                println!(
                    "DEBUG: Pushing directory to stack: {}",
                    pretty_abs_display(&path)
                );
                stack.push(path);
            } else if path.is_file() {
                // Only canonicalize files, not directories
                match fs::canonicalize(&path) {
                    Ok(canonical) => {
                        file_set.insert(canonical);
                    }
                    Err(_) => {
                        file_set.insert(path);
                    } // fallback to original path
                }
            }
        }
    }

    #[cfg(any(debug_assertions, test))]
    {
        println!("DEBUG: Total files found: {}", file_set.len());
        println!("the files retrieved are : ");
        for file in &file_set {
            println!("DEBUG: File - {}", pretty_abs_display(file));
        }

        println!(" \n\n\n\n\nDEBUG: Excluded directories: ");
        for ex_dir in excludes_files {
            println!("DEBUG: Excluded Dir - {}", pretty_abs_display(&ex_dir));
        }
    }

    file_set
}

/// A lightweight borrowed “view” of a [`ProjectStructure`], anchored at a current directory.
///
/// This is a placeholder for a future proxy API.
#[allow(dead_code)]
pub struct ProjectStructureAsRef<'a> {
    structure: &'a ProjectStructure,
    current_dir: PathBuf, // root-relative path
}

impl<'a> ProjectStructureAsRef<'a> {
    /// Create a new borrowed view.
    pub fn new(structure: &'a ProjectStructure, current_dir: PathBuf) -> Self {
        ProjectStructureAsRef {
            structure,
            current_dir,
        }
        //  too make the proxy, so that no need to rewrite the whole functiosn from the ProjectSgtructure to the ProjectStructureAsRef
        // and it will invoke and current_dir, under this , and no need to writ eh whole names
        //    macro_rules! proxy(){

        //    }
    }

    /// Create a deeper scoped view under the current view.
    ///
    /// This enables chaining like `ps.to("test_sample")?.to("src")?`.
    pub fn to<P: AsRef<Path>>(&self, subfolder: P) -> Result<ProjectStructureAsRef<'a>, String> {
        let subfolder = subfolder.as_ref();

        let root_relative = if subfolder == Path::new(".") {
            self.current_dir.clone()
        } else if subfolder.is_absolute() {
            let rel = self.structure.resolve_under_root(subfolder)?;
            if self.current_dir != Path::new(".") && !rel.starts_with(&self.current_dir) {
                return Err("Path is not under the current view".to_string());
            }
            rel
        } else {
            self.current_dir.join(subfolder)
        };

        let abs = self.structure.root_path.join(&root_relative);
        if !abs.exists() {
            return Err("Subfolder does not exist under the current view".to_string());
        }
        if !abs.is_dir() {
            return Err("Requested subfolder is not a directory".to_string());
        }

        Ok(ProjectStructureAsRef::new(self.structure, root_relative))
    }

    /// Returns the view's root-relative directory.
    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    fn anchor_abs(&self) -> PathBuf {
        let abs = self.structure.root_path.join(&self.current_dir);
        fs::canonicalize(&abs).unwrap_or(abs)
    }

    fn abs_from_view(&self, path: &Path) -> PathBuf {
        if path == Path::new(".") {
            return self.structure.root_path.join(&self.current_dir);
        }

        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.structure.root_path.join(&self.current_dir).join(path)
        }
    }

    /// Returns `true` if `path` exists and is under this view.
    ///
    /// `path` may be absolute, or relative to the view root.
    pub fn exists(&self, path: &Path) -> bool {
        let abs = self.abs_from_view(path);
        if !abs.exists() {
            return false;
        }

        let abs = fs::canonicalize(&abs).unwrap_or(abs);
        let anchor = self.anchor_abs();
        abs.starts_with(&anchor)
    }

    /// Get a file by path, where relative paths are interpreted relative to the view.
    pub fn get_file_by_path(&self, path: &Path) -> Option<&FileNode> {
        if !self.exists(path) {
            return None;
        }
        let abs =
            fs::canonicalize(self.abs_from_view(path)).unwrap_or_else(|_| self.abs_from_view(path));
        self.structure.get_file_by_path(&abs)
    }

    /// Get files directly inside `dir_path` (relative to the view).
    pub fn get_files_in_dir(&self, dir_path: &Path) -> Vec<&FileNode> {
        let abs = self.abs_from_view(dir_path);
        if !abs.is_dir() {
            return Vec::new();
        }
        // Delegate to the base implementation; it accepts absolute paths.
        self.structure.get_files_in_dir(&abs)
    }

    /// Get all files under this view (recursive).
    pub fn get_all_files(&self) -> Vec<&FileNode> {
        let anchor = self.anchor_abs();
        self.structure
            .get_all_files()
            .into_iter()
            .filter(|f| f.get_path().starts_with(&anchor))
            .collect()
    }

    /// Get all header files under this view.
    pub fn get_header_file(&self) -> Vec<&FileNode> {
        self.get_all_files()
            .into_iter()
            .filter(|f| f.is_header())
            .collect()
    }

    /// Get all source files under this view.
    pub fn get_source_file(&self) -> Vec<&FileNode> {
        self.get_all_files()
            .into_iter()
            .filter(|f| f.get_type() == &SourceFileType::Source)
            .collect()
    }

    /// Get all files with an extension under this view.
    pub fn get_files_with_extension(&self, extension: &str) -> Vec<&FileNode> {
        let normalized = extension
            .trim()
            .strip_prefix('.')
            .unwrap_or(extension.trim());

        self.get_all_files()
            .into_iter()
            .filter(|file| {
                file.get_path()
                    .extension()
                    .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case(normalized))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Configs;
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static FS_TEST_LOCK: Mutex<()> = Mutex::new(());

    // ------------------------------------------------------------
    // Helper: create ProjectStructure from existing test_project
    // ------------------------------------------------------------
    fn make_test_project_structure() -> (Configs, ProjectStructure) {
        let root_abs = fs::canonicalize("./tests/test_project")
            .unwrap_or_else(|_| PathBuf::from("./tests/test_project"));

        let root_str = root_abs.to_string_lossy().to_string();
        let configs = Configs::test_config(Some(&root_str));
        let structure = ProjectStructure::new(&configs);

        (configs, structure)
    }

    fn snapshot_tree_paths(root_abs: &Path) -> HashSet<PathBuf> {
        let mut out = HashSet::new();
        let mut stack = vec![root_abs.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let rd = match fs::read_dir(&dir) {
                Ok(rd) => rd,
                Err(_) => continue,
            };

            for entry in rd.flatten() {
                let path = entry.path();
                let rel = match path.strip_prefix(root_abs) {
                    Ok(p) => p.to_path_buf(),
                    Err(_) => continue,
                };
                out.insert(rel);
                if path.is_dir() {
                    stack.push(path);
                }
            }
        }
        out
    }

    fn unique_rel_path(root_abs: &Path, base: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        for i in 0u32..5000 {
            let name = format!("__ps_test_{}_{}_{}_{}", base, pid, nanos, i);
            let rel = PathBuf::from(name);
            if !root_abs.join(&rel).exists() {
                return rel;
            }
        }
        // Extremely unlikely; fall back to a fixed name.
        PathBuf::from(format!("__ps_test_{}", base))
    }

    struct FsCleanup {
        files: Vec<PathBuf>,
        dirs: Vec<PathBuf>,
    }

    impl FsCleanup {
        fn new() -> Self {
            Self {
                files: Vec::new(),
                dirs: Vec::new(),
            }
        }

        fn track_file(&mut self, abs: PathBuf) {
            self.files.push(abs);
        }

        fn track_dir(&mut self, abs: PathBuf) {
            self.dirs.push(abs);
        }
    }

    impl Drop for FsCleanup {
        fn drop(&mut self) {
            for file in self.files.iter().rev() {
                let _ = fs::remove_file(file);
            }
            for dir in self.dirs.iter().rev() {
                let _ = fs::remove_dir_all(dir);
            }
        }
    }

    // ------------------------------------------------------------
    // READ-ONLY TESTS (no filesystem mutation)
    // ------------------------------------------------------------

    #[test]
    fn indexes_known_files_and_paths() {
        let (_configs, structure) = make_test_project_structure();

        let expected_files = [
            "main.cpp",
            "logger.cpp",
            "logger.h",
            "math_utils.cpp",
            "math_utils.h",
            "util.cpp",
            "util.h",
            "some_code.cpp",
            "some_code.h",
            "some_message.cpp",
            "some_message.h",
        ];

        for file in expected_files {
            assert!(
                structure.get_file_by_name(file).is_some(),
                "Expected `{}` to be indexed",
                file
            );
        }

        assert!(structure.exists(Path::new("main.cpp")));
        assert!(!structure.exists(Path::new("__does_not_exist__.cpp")));
    }

    #[test]
    fn source_and_header_files_are_classified_correctly() {
        let (_configs, structure) = make_test_project_structure();

        let sources = structure.get_source_file();
        let headers = structure.get_header_file();

        assert!(sources.iter().any(|f| f.get_path().ends_with("main.cpp")));
        assert!(headers.iter().any(|f| f.get_path().ends_with("logger.h")));

        assert!(!sources.iter().any(|f| f.get_path().ends_with("logger.h")));
    }

    #[test]
    fn extension_queries_work_with_and_without_dot() {
        let (_configs, structure) = make_test_project_structure();

        let cpp1 = structure.get_files_with_extension(".cpp");
        let cpp2 = structure.get_files_with_extension("cpp");

        assert_eq!(cpp1.len(), cpp2.len());
        assert!(cpp1.iter().any(|f| f.get_path().ends_with("logger.cpp")));
    }

    #[test]
    fn header_lookup_and_impl_resolution_works() {
        let (_configs, structure) = make_test_project_structure();

        let headers = ["logger.h", "math_utils.h", "util.h"];

        for header in headers {
            let paths = structure
                .get_header_file_paths(header)
                .expect("Header should exist");

            let header_path = &paths[0];
            let impl_path = structure
                .get_header_impl_file(header_path)
                .expect("Implementation should exist");

            let impl_stem = impl_path.file_stem().unwrap().to_string_lossy();
            let header_stem = header_path.file_stem().unwrap().to_string_lossy();
            assert!(impl_stem.starts_with(header_stem.as_ref()));
        }
    }

    #[test]
    fn get_files_in_dir_respects_boundaries() {
        let (_configs, structure) = make_test_project_structure();

        let root_files = structure.get_files_in_dir(Path::new("."));
        assert!(root_files
            .iter()
            .any(|f| f.get_path().ends_with("main.cpp")));

        assert!(!root_files
            .iter()
            .any(|f| { f.get_path().to_string_lossy().contains("test_sample/src") }));

        let nested = structure.get_files_in_dir(Path::new("test_sample/src"));
        assert!(nested
            .iter()
            .any(|f| { f.get_path().ends_with("test_sample/src/main.cpp") }));
    }

    #[test]
    fn test_config_does_not_exclude_dirs_by_default() {
        let (_configs, structure) = make_test_project_structure();
        // `Configs::test_config` currently uses an empty excluded set.
        // So directories like `build/` and nested `.cbuild/` should be indexed.
        assert!(structure
            .get_file_by_path(Path::new("build/main.exe"))
            .is_some());
        assert!(structure
            .get_file_by_path(Path::new("test_sample/.cbuild/.info"))
            .is_some());
    }

    #[test]
    fn absolute_and_relative_paths_resolve_identically() {
        let (configs, structure) = make_test_project_structure();

        let rel = structure.get_file_by_path(Path::new("main.cpp")).unwrap();

        let abs = structure
            .get_file_by_path(&configs.get_project_root().join("main.cpp"))
            .unwrap();

        assert_eq!(rel.get_path(), abs.get_path());
    }

    #[test]
    fn index_based_access_is_safe() {
        let (_configs, structure) = make_test_project_structure();

        let all = structure.get_all_files();
        assert!(!all.is_empty());

        for i in 0..all.len() {
            assert!(structure.get_by_index(i).is_some());
        }

        assert!(structure.get_by_index(all.len()).is_none());
    }

    // ------------------------------------------------------------
    // MUTATION TESTS (create → test → cleanup)
    // ------------------------------------------------------------

    #[test]
    fn create_and_remove_file_restores_state() {
        let _lock = FS_TEST_LOCK.lock().unwrap();
        let (configs, mut structure) = make_test_project_structure();

        let root_abs = configs.get_project_root().clone();
        let before = snapshot_tree_paths(&root_abs);

        let mut cleanup = FsCleanup::new();

        let test_file_rel = unique_rel_path(&root_abs, "file.cpp");
        let test_file_abs = root_abs.join(&test_file_rel);
        cleanup.track_file(test_file_abs.clone());

        // Create file on disk
        fs::write(&test_file_abs, "int temp() { return 42; }").unwrap();

        // Register in structure
        let created = structure
            .create_file(&test_file_rel, SourceFileType::Source)
            .expect("create_file should succeed");

        assert_eq!(created, test_file_rel);
        assert!(structure.get_file_by_path(&test_file_rel).is_some());

        // Remove file
        structure
            .remove_file(&test_file_rel)
            .expect("remove_file should succeed");

        assert!(!test_file_abs.exists());
        assert!(structure.get_file_by_path(&test_file_rel).is_none());

        drop(cleanup);
        let after = snapshot_tree_paths(&root_abs);
        assert_eq!(before, after, "Directory tree must be unchanged after test");
    }

    #[test]
    fn create_and_remove_directory_restores_state() {
        let _lock = FS_TEST_LOCK.lock().unwrap();
        let (configs, mut structure) = make_test_project_structure();

        let root_abs = configs.get_project_root().clone();
        let before = snapshot_tree_paths(&root_abs);

        let mut cleanup = FsCleanup::new();

        let test_dir_rel = unique_rel_path(&root_abs, "dir");
        let test_dir_abs = root_abs.join(&test_dir_rel);
        cleanup.track_dir(test_dir_abs.clone());

        // Create directory
        let created = structure
            .create_dir(&test_dir_rel)
            .expect("create_dir should succeed");

        assert_eq!(created, test_dir_rel);
        assert!(test_dir_abs.exists());

        // Create a file inside it
        let nested_file_rel = test_dir_rel.join("nested.cpp");
        let nested_file_abs = root_abs.join(&nested_file_rel);
        cleanup.track_file(nested_file_abs.clone());

        fs::write(&nested_file_abs, "int nested() { return 0; }").unwrap();

        structure
            .create_file(&nested_file_rel, SourceFileType::Source)
            .expect("create nested file");

        assert!(structure.get_file_by_path(&nested_file_rel).is_some());

        // Remove nested file first (required)
        structure
            .remove_file(&nested_file_rel)
            .expect("remove nested file");

        // Remove directory
        structure
            .remove_dir(&test_dir_rel)
            .expect("remove_dir should succeed");

        assert!(!test_dir_abs.exists());

        drop(cleanup);
        let after = snapshot_tree_paths(&root_abs);
        assert_eq!(before, after, "Directory tree must be unchanged after test");
    }

    // ------------------------------------------------------------
    // DEBUG / VISUAL INSPECTION
    // ------------------------------------------------------------

    #[test]
    fn print_structure_does_not_panic() {
        let (_configs, structure) = make_test_project_structure();
        structure.debug_print();
    }
}
