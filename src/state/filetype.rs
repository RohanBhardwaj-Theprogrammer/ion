#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum SourceFileType {
    Header,
    Source,
    Entry,
    Unknown,
    Object,
    Binary,
    Dll,
}

impl SourceFileType {
    /// Determine SourceFileType from file extension
    /// # Arguments
    /// * `extension` - File extension as &str  without the leading dot
    /// #example :
    ///   let file_type = SourceFileType::from_extension("cpp");
    /// # Returns
    /// * `SourceFileType` - Corresponding SourceFileType enum variant
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "h" | "hpp" | "hh" | "hxx" => SourceFileType::Header,
            "c" | "cpp" | "cc" | "cxx" | "m" | "mm" => SourceFileType::Source,
            "o" | "obj" | "a" => SourceFileType::Object,
            "exe" | "out" | "bin" => SourceFileType::Binary,
            "dll" | "so" | "dylib" => SourceFileType::Dll,
            _ => SourceFileType::Unknown,
        }
    }
}
