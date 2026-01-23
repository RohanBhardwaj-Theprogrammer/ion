use crate::compiler::trailt::ToCompilerArgs;
use std::collections::HashSet;

// --- Preprocessor Configuration ---
#[derive(Debug, Clone)]
pub struct Macros {
    pub def: HashSet<String>,
    pub undef: HashSet<String>,
}

impl Default for Macros {
    fn default() -> Self {
        Macros {
            def: HashSet::new(),
            undef: HashSet::new(),
        }
    }
}

impl ToCompilerArgs for Macros {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for def_macro in &self.def {
            args.push(format!("-D{}", def_macro));
        }
        for undef_macro in &self.undef {
            args.push(format!("-U{}", undef_macro));
        }
        args
    }
}

impl Macros {
    pub fn new() -> Self {
        Macros::default()
    }

    pub fn add_define(&mut self, macro_name: &str) {
        self.def.insert(macro_name.to_string());
    }

    pub fn add_undefine(&mut self, macro_name: &str) {
        self.undef.insert(macro_name.to_string());
    }

    pub fn remove_define(&mut self, macro_name: &str) {
        self.def.remove(macro_name);
    }

    pub fn remove_undefine(&mut self, macro_name: &str) {
        self.undef.remove(macro_name);
    }
}

#[derive(Debug, Clone)]
pub struct PreprocessorConfig {
    pub macros: Macros,
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        PreprocessorConfig {
            macros: Macros::default(),
        }
    }
}

impl ToCompilerArgs for PreprocessorConfig {
    fn to_args(&self) -> Vec<String> {
        self.macros.to_args()
    }
}

// --- Language Configuration ---
#[derive(Debug, Clone)]
pub enum LangX {
    C(i16),   // version
    Cpp(i16), // version
    Asm,
    AsmCpp,
}

impl ToCompilerArgs for LangX {
    fn to_args(&self) -> Vec<String> {
        match self {
            LangX::C(version) => vec![format!("-std=c{}", version)],
            LangX::Cpp(version) => vec![format!("-std=c++{}", version)],
            LangX::Asm => vec![],
            LangX::AsmCpp => vec![],
        }
    }
}

impl Default for LangX {
    fn default() -> Self {
        LangX::Cpp(17) // Default to C++17
    }
}

#[derive(Debug, Clone)]
pub enum LangException {
    NoExceptions,
    Exceptions,
    Default, // Let compiler decide
}

impl Default for LangException {
    fn default() -> Self {
        LangException::Default
    }
}

impl ToCompilerArgs for LangException {
    fn to_args(&self) -> Vec<String> {
        match self {
            LangException::NoExceptions => vec!["-fno-exceptions".to_string()],
            LangException::Exceptions => vec!["-fexceptions".to_string()],
            LangException::Default => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub standard: LangX,
    pub exception: LangException,
    pub strict_checking: bool,        // -fstrict-aliasing, etc.
    pub lax_vector_conversions: bool, // -flax-vector-conversions
}

impl Default for LanguageConfig {
    fn default() -> Self {
        LanguageConfig {
            standard: LangX::default(),
            exception: LangException::default(),
            strict_checking: false,
            lax_vector_conversions: false,
        }
    }
}

impl ToCompilerArgs for LanguageConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.standard.to_args();
        args.extend(self.exception.to_args());
        if self.strict_checking {
            args.push("-fstrict-aliasing".to_string());
        }
        if self.lax_vector_conversions {
            args.push("-flax-vector-conversions".to_string());
        }
        args
    }
}

// --- Control Configuration ---
#[derive(Debug, Clone)]
pub enum DependencyInfoBuild {
    None,
    M,
    MM,
    MD,
    MMD,
}

impl Default for DependencyInfoBuild {
    fn default() -> Self {
        DependencyInfoBuild::None
    }
}

impl ToCompilerArgs for DependencyInfoBuild {
    fn to_args(&self) -> Vec<String> {
        match self {
            DependencyInfoBuild::None => vec![],
            DependencyInfoBuild::M => vec!["-M".to_string()],
            DependencyInfoBuild::MM => vec!["-MM".to_string()],
            DependencyInfoBuild::MD => vec!["-MD".to_string()],
            DependencyInfoBuild::MMD => vec!["-MMD".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompilationMode {
    Default,     // Full compilation to executable
    ObjectOnly,  // -c : Compile to object files only
    Assemble,    // -S : Compile to assembly
    Preprocess,  // -E : Preprocess only
    SyntaxCheck, // -fsyntax-only : check syntax only (no codegen)
}

impl Default for CompilationMode {
    fn default() -> Self {
        CompilationMode::Default
    }
}

impl ToCompilerArgs for CompilationMode {
    fn to_args(&self) -> Vec<String> {
        match self {
            CompilationMode::Default => vec![],
            CompilationMode::ObjectOnly => vec!["-c".to_string()],
            CompilationMode::Assemble => vec!["-S".to_string()],
            CompilationMode::Preprocess => vec!["-E".to_string()],
            CompilationMode::SyntaxCheck => vec!["-fsyntax-only".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilationControl {
    pub compilation_mode: CompilationMode,
    pub dependency_info: DependencyInfoBuild,
    pub use_pipe: bool,
}

impl Default for CompilationControl {
    fn default() -> Self {
        CompilationControl {
            compilation_mode: CompilationMode::default(),
            dependency_info: DependencyInfoBuild::default(),
            use_pipe: false,
        }
    }
}

impl ToCompilerArgs for CompilationControl {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.compilation_mode.to_args();
        args.extend(self.dependency_info.to_args());
        if self.use_pipe {
            args.push("-pipe".to_string());
        }
        args
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoggingOptions {
    pub verbose: bool,
    pub show_commands: bool,
}

#[derive(Debug, Clone)]
pub struct ControlConfig {
    pub control: CompilationControl,
    pub logging: LoggingOptions,
}

impl Default for ControlConfig {
    fn default() -> Self {
        ControlConfig {
            control: CompilationControl::default(),
            logging: LoggingOptions::default(),
        }
    }
}

impl ToCompilerArgs for ControlConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.control.to_args();
        if self.logging.verbose {
            args.push("-v".to_string());
        }
        args
    }
}

// --- Optimization Configuration ---
#[derive(Debug, Clone)]
pub enum OptLevel {
    Default, // Let compiler decide
    O0,
    O1,
    O2,
    O3,
    Os,
    Oz,
    Ofast,
}

impl Default for OptLevel {
    fn default() -> Self {
        OptLevel::Default
    }
}

impl ToCompilerArgs for OptLevel {
    fn to_args(&self) -> Vec<String> {
        match self {
            OptLevel::Default => vec![],
            OptLevel::O0 => vec!["-O0".to_string()],
            OptLevel::O1 => vec!["-O1".to_string()],
            OptLevel::O2 => vec!["-O2".to_string()],
            OptLevel::O3 => vec!["-O3".to_string()],
            OptLevel::Os => vec!["-Os".to_string()],
            OptLevel::Oz => vec!["-Oz".to_string()],
            OptLevel::Ofast => vec!["-Ofast".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Lto {
    None,
    Thin,
    Full,
}

impl Default for Lto {
    fn default() -> Self {
        Lto::None
    }
}

impl ToCompilerArgs for Lto {
    fn to_args(&self) -> Vec<String> {
        match self {
            Lto::None => vec![],
            Lto::Thin => vec!["-flto=thin".to_string()],
            Lto::Full => vec!["-flto".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationOptions {
    pub level: OptLevel,
    pub march: Option<String>,
    pub mtune: Option<String>,
    pub lto: Lto,
    pub fpic: bool,
    pub funroll_loops: bool,
    pub fomit_frame_pointer: bool,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        OptimizationOptions {
            level: OptLevel::default(),
            march: None,
            mtune: None,
            lto: Lto::default(),
            fpic: false,
            funroll_loops: false,
            fomit_frame_pointer: false,
        }
    }
}

impl OptimizationOptions {
    /// Optimized settings for release builds
    pub fn release() -> Self {
        OptimizationOptions {
            level: OptLevel::O3,
            march: Some("native".to_string()),
            mtune: Some("native".to_string()),
            // Thin LTO isn't universally supported across GCC toolchains (notably some Windows builds).
            lto: Lto::Full,
            fpic: false,
            funroll_loops: true,
            fomit_frame_pointer: true,
        }
    }

    /// Size-optimized settings
    pub fn size_optimized() -> Self {
        OptimizationOptions {
            level: OptLevel::Os,
            march: None,
            mtune: None,
            lto: Lto::Full,
            fpic: false,
            funroll_loops: false,
            fomit_frame_pointer: false,
        }
    }
}

impl ToCompilerArgs for OptimizationOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.level.to_args();
        if let Some(march) = &self.march {
            args.push(format!("-march={}", march));
        }
        if let Some(mtune) = &self.mtune {
            args.push(format!("-mtune={}", mtune));
        }
        args.extend(self.lto.to_args());
        if self.fpic {
            args.push("-fPIC".to_string());
        }
        if self.funroll_loops {
            args.push("-funroll-loops".to_string());
        }
        if self.fomit_frame_pointer {
            args.push("-fomit-frame-pointer".to_string());
        }
        args
    }
}

#[derive(Debug, Clone, Default)]
pub struct MachineTarget {
    pub target_triple: Option<String>,
    pub cpu: Option<String>,
    pub features: Vec<String>,
}

impl ToCompilerArgs for MachineTarget {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(triple) = &self.target_triple {
            // --target=<triple> for cross-compilation (works with both GCC and Clang)
            args.push(format!("--target={}", triple));
        }
        if let Some(cpu) = &self.cpu {
            // -mcpu is for ARM/AArch64, use -march for x86
            args.push(format!("-mcpu={}", cpu));
        }
        for feature in &self.features {
            // Features like "sse4.2", "avx2", "64", "32"
            args.push(format!("-m{}", feature));
        }
        args
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub options: OptimizationOptions,
    pub target: MachineTarget,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        OptimizationConfig {
            options: OptimizationOptions::default(),
            target: MachineTarget::default(),
        }
    }
}

impl OptimizationConfig {
    pub fn release() -> Self {
        OptimizationConfig {
            options: OptimizationOptions::release(),
            target: MachineTarget::default(),
        }
    }
}

impl ToCompilerArgs for OptimizationConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.options.to_args();
        args.extend(self.target.to_args());
        args
    }
}

// --- Diagnostic Configuration ---
#[derive(Debug, Clone)]
pub enum DebugLevel {
    None,
    G,       // -g (default debug info)
    G1,      // -g1 (minimal debug info)
    G2,      // -g2 (default level, same as -g)
    G3,      // -g3 (maximal debug info, includes macros)
    Ggdb,    // -ggdb (GDB-optimized debug info)
    Gdwarf4, // -gdwarf-4 (DWARF version 4)
    Gdwarf5, // -gdwarf-5 (DWARF version 5, most recent)
}

impl Default for DebugLevel {
    fn default() -> Self {
        DebugLevel::None
    }
}

impl ToCompilerArgs for DebugLevel {
    fn to_args(&self) -> Vec<String> {
        match self {
            DebugLevel::None => vec![],
            DebugLevel::G => vec!["-g".to_string()],
            DebugLevel::G1 => vec!["-g1".to_string()],
            DebugLevel::G2 => vec!["-g2".to_string()],
            DebugLevel::G3 => vec!["-g3".to_string()],
            DebugLevel::Ggdb => vec!["-ggdb".to_string()],
            DebugLevel::Gdwarf4 => vec!["-gdwarf-4".to_string()],
            DebugLevel::Gdwarf5 => vec!["-gdwarf-5".to_string()],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugOptions {
    pub level: DebugLevel,
}

impl ToCompilerArgs for DebugOptions {
    fn to_args(&self) -> Vec<String> {
        self.level.to_args()
    }
}

#[derive(Debug, Clone)]
pub struct WarningOptions {
    pub wall: bool,
    pub wextra: bool,
    pub werror: bool,
    pub wpedantic: bool,
    pub pedantic_errors: bool,
}

impl Default for WarningOptions {
    fn default() -> Self {
        WarningOptions {
            wall: false,
            wextra: false,
            werror: false,
            wpedantic: false,
            pedantic_errors: false,
        }
    }
}

impl WarningOptions {
    /// Strict warning settings for release/production
    pub fn strict() -> Self {
        WarningOptions {
            wall: true,
            wextra: true,
            werror: true,
            wpedantic: true,
            pedantic_errors: false,
        }
    }
}

impl ToCompilerArgs for WarningOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.wall {
            args.push("-Wall".to_string());
        }
        if self.wextra {
            args.push("-Wextra".to_string());
        }
        if self.werror {
            args.push("-Werror".to_string());
        }
        if self.wpedantic {
            args.push("-Wpedantic".to_string());
        }
        if self.pedantic_errors {
            args.push("-pedantic-errors".to_string());
        }
        args
    }
}

/// Sanitizer options for runtime error detection
/// NOTE: address, thread, and memory sanitizers are mutually exclusive!
/// - Use address + undefined + leak together (common combo)
/// - Use thread alone
/// - Use memory alone
#[derive(Debug, Clone, Default)]
pub struct SanitizerOptions {
    pub address: bool,   // -fsanitize=address (detects memory errors)
    pub thread: bool,    // -fsanitize=thread (detects data races) - EXCLUSIVE
    pub memory: bool,    // -fsanitize=memory (detects uninitialized reads) - EXCLUSIVE
    pub undefined: bool, // -fsanitize=undefined (detects UB)
    pub leak: bool,      // -fsanitize=leak (detects memory leaks)
}

impl ToCompilerArgs for SanitizerOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.address {
            args.push("-fsanitize=address".to_string());
        }
        if self.thread {
            args.push("-fsanitize=thread".to_string());
        }
        if self.memory {
            args.push("-fsanitize=memory".to_string());
        }
        if self.undefined {
            args.push("-fsanitize=undefined".to_string());
        }
        if self.leak {
            args.push("-fsanitize=leak".to_string());
        }
        args
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    pub debug: DebugOptions,
    pub warnings: WarningOptions,
    pub sanitizers: SanitizerOptions,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        DiagnosticConfig {
            debug: DebugOptions::default(),
            warnings: WarningOptions::default(),
            sanitizers: SanitizerOptions::default(),
        }
    }
}

impl DiagnosticConfig {
    /// Debug build with debug symbols and sanitizers
    pub fn debug() -> Self {
        DiagnosticConfig {
            debug: DebugOptions {
                level: DebugLevel::G,
            },
            warnings: WarningOptions {
                wall: true,
                wextra: true,
                ..Default::default()
            },
            sanitizers: SanitizerOptions {
                address: true,
                undefined: true,
                ..Default::default()
            },
        }
    }

    /// Release build with strict warnings, no debug info
    pub fn release() -> Self {
        DiagnosticConfig {
            debug: DebugOptions::default(),
            warnings: WarningOptions::strict(),
            sanitizers: SanitizerOptions::default(),
        }
    }
}

impl ToCompilerArgs for DiagnosticConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.debug.to_args();
        args.extend(self.warnings.to_args());
        args.extend(self.sanitizers.to_args());
        args
    }
}

// --- Linker Configuration ---

/// Link mode: static, shared, or default (let compiler decide)
#[derive(Debug, Clone)]
pub enum LinkMode {
    Default, // Let compiler decide
    Static,  // -static (fully static linking)
    Shared,  // -shared (create shared library)
}

impl Default for LinkMode {
    fn default() -> Self {
        LinkMode::Default
    }
}

impl ToCompilerArgs for LinkMode {
    fn to_args(&self) -> Vec<String> {
        match self {
            LinkMode::Default => vec![],
            LinkMode::Static => vec!["-static".to_string()],
            LinkMode::Shared => vec!["-shared".to_string()],
        }
    }
}

/// Runtime library options
#[derive(Debug, Clone, Default)]
pub struct RuntimeOptions {
    pub static_libgcc: bool,    // -static-libgcc
    pub static_libstdcpp: bool, // -static-libstdc++
    pub pthread: bool,          // -pthread
}

impl ToCompilerArgs for RuntimeOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.static_libgcc {
            args.push("-static-libgcc".to_string());
        }
        if self.static_libstdcpp {
            args.push("-static-libstdc++".to_string());
        }
        if self.pthread {
            args.push("-pthread".to_string());
        }
        args
    }
}

/// ABI (Application Binary Interface) options
#[derive(Debug, Clone, Default)]
pub struct ABIOptions {
    pub abi_version: Option<String>, // -fabi-version=<n>
    pub no_gnu_unique: bool,         // -fno-gnu-unique
    pub no_common: bool,             // -fno-common
}

impl ToCompilerArgs for ABIOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(ver) = &self.abi_version {
            args.push(format!("-fabi-version={}", ver));
        }
        if self.no_gnu_unique {
            args.push("-fno-gnu-unique".to_string());
        }
        if self.no_common {
            args.push("-fno-common".to_string());
        }
        args
    }
}

/// Library paths and libraries to link
/// NOTE: Order matters! -l flags should come AFTER source/object files
/// The Compiler should ensure correct ordering when building final command
#[derive(Debug, Clone, Default)]
pub struct LinkPaths {
    pub lib_paths: Vec<String>, // -L<path> (library search paths)
    pub libs: Vec<String>,      // -l<name> (libraries to link)
    pub rpath: Vec<String>,     // -Wl,-rpath,<path> (runtime library path)
}

impl ToCompilerArgs for LinkPaths {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for path in &self.lib_paths {
            args.push(format!("-L{}", path));
        }
        for lib in &self.libs {
            args.push(format!("-l{}", lib));
        }
        for rpath in &self.rpath {
            args.push(format!("-Wl,-rpath,{}", rpath));
        }
        args
    }
}

/// Additional linker-specific options passed via -Wl
#[derive(Debug, Clone, Default)]
pub struct LinkerOptions {
    pub as_needed: bool,          // -Wl,--as-needed
    pub no_undefined: bool,       // -Wl,--no-undefined
    pub gc_sections: bool,        // -Wl,--gc-sections (remove unused sections)
    pub extra_flags: Vec<String>, // Additional -Wl,<flag> options
}

impl ToCompilerArgs for LinkerOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.as_needed {
            args.push("-Wl,--as-needed".to_string());
        }
        if self.no_undefined {
            args.push("-Wl,--no-undefined".to_string());
        }
        if self.gc_sections {
            args.push("-Wl,--gc-sections".to_string());
        }
        for flag in &self.extra_flags {
            args.push(format!("-Wl,{}", flag));
        }
        args
    }
}

/// Complete linker configuration
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    pub mode: LinkMode,
    pub runtime: RuntimeOptions,
    pub abi: ABIOptions,
    pub options: LinkerOptions,
    pub paths: LinkPaths,
    pub strip: bool, // -s (strip symbols for smaller binary)
}

impl Default for LinkerConfig {
    fn default() -> Self {
        LinkerConfig {
            mode: LinkMode::default(),
            runtime: RuntimeOptions::default(),
            abi: ABIOptions::default(),
            options: LinkerOptions::default(),
            paths: LinkPaths::default(),
            strip: false,
        }
    }
}

impl LinkerConfig {
    /// Release linker settings - strip symbols, gc-sections for smaller binary
    pub fn release() -> Self {
        LinkerConfig {
            mode: LinkMode::default(),
            runtime: RuntimeOptions::default(),
            abi: ABIOptions::default(),
            options: LinkerOptions {
                gc_sections: true, // Remove unused sections
                ..Default::default()
            },
            paths: LinkPaths::default(),
            strip: true,
        }
    }

    /// Static linking configuration
    pub fn static_link() -> Self {
        LinkerConfig {
            mode: LinkMode::Static,
            runtime: RuntimeOptions {
                static_libgcc: true,
                static_libstdcpp: true,
                ..Default::default()
            },
            abi: ABIOptions::default(),
            options: LinkerOptions::default(),
            paths: LinkPaths::default(),
            strip: false,
        }
    }
}

impl ToCompilerArgs for LinkerConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = self.mode.to_args();
        args.extend(self.runtime.to_args());
        args.extend(self.abi.to_args());
        args.extend(self.options.to_args());
        args.extend(self.paths.to_args());
        if self.strip {
            args.push("-s".to_string());
        }
        args
    }
}

// --- Build Settings (Main Configuration) ---
#[derive(Debug, Clone)]
pub struct BuildSettings {
    pub preprocessor: PreprocessorConfig,
    pub language: LanguageConfig,
    pub optimization: OptimizationConfig,
    pub diagnostics: DiagnosticConfig,
    pub linking: LinkerConfig,
    pub control: ControlConfig,
}

impl Default for BuildSettings {
    fn default() -> Self {
        BuildSettings {
            preprocessor: PreprocessorConfig::default(),
            language: LanguageConfig::default(),
            optimization: OptimizationConfig::default(),
            diagnostics: DiagnosticConfig::default(),
            linking: LinkerConfig::default(),
            control: ControlConfig::default(),
        }
    }
}

impl BuildSettings {
    /// Release build: O3, LTO, strip symbols, strict warnings
    pub fn release() -> Self {
        BuildSettings {
            preprocessor: {
                let mut config = PreprocessorConfig::default();
                config.macros.add_define("NDEBUG");
                config
            },
            language: LanguageConfig {
                standard: LangX::Cpp(17),
                exception: LangException::Default,
                strict_checking: false,
                lax_vector_conversions: false,
            },
            optimization: OptimizationConfig::release(),
            diagnostics: DiagnosticConfig::release(),
            linking: LinkerConfig::release(),
            control: ControlConfig::default(),
        }
    }

    /// Debug build: No optimization, debug symbols, sanitizers
    pub fn debug() -> Self {
        BuildSettings {
            preprocessor: {
                let mut config = PreprocessorConfig::default();
                config.macros.add_define("DEBUG");
                config
            },
            language: LanguageConfig::default(),
            optimization: OptimizationConfig::default(),
            diagnostics: DiagnosticConfig::debug(),
            linking: LinkerConfig::default(),
            control: ControlConfig::default(),
        }
    }

    /// Fast build: Minimal flags, quick compilation
    pub fn fast() -> Self {
        BuildSettings {
            preprocessor: PreprocessorConfig::default(),
            language: LanguageConfig {
                standard: LangX::Cpp(17),
                exception: LangException::Default,
                strict_checking: false,
                lax_vector_conversions: false,
            },
            optimization: OptimizationConfig::default(), // O0
            diagnostics: DiagnosticConfig::default(),    // No warnings
            linking: LinkerConfig::default(),
            control: ControlConfig {
                control: CompilationControl {
                    use_pipe: true, // Faster compilation
                    ..Default::default()
                },
                logging: LoggingOptions::default(),
            },
        }
    }

    /// Object-only build: Compile to .o files
    pub fn object() -> Self {
        BuildSettings {
            preprocessor: PreprocessorConfig::default(),
            language: LanguageConfig::default(),
            optimization: OptimizationConfig::default(),
            diagnostics: DiagnosticConfig::default(),
            linking: LinkerConfig::default(),
            control: ControlConfig {
                control: CompilationControl {
                    compilation_mode: CompilationMode::ObjectOnly,
                    ..Default::default()
                },
                logging: LoggingOptions::default(),
            },
        }
    }
}

impl ToCompilerArgs for BuildSettings {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Order matters for compiler flags:
        // 1. Control flags (compilation mode like -c, -S, -E)
        args.extend(self.control.to_args());
        // 2. Language standard
        args.extend(self.language.to_args());
        // 3. Preprocessor defines
        args.extend(self.preprocessor.to_args());
        // 4. Warnings and diagnostics
        args.extend(self.diagnostics.to_args());
        // 5. Optimization flags
        args.extend(self.optimization.to_args());
        // 6. Linker flags (should be last, -l flags after source files)
        //    Note: The Compiler should place source files before these
        args.extend(self.linking.to_args());

        args
    }
}
