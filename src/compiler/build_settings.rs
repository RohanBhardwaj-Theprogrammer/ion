use crate::compiler::trailt::{FromBuildConfigs, ToCompilerArgs};
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
impl FromBuildConfigs<PreprocessorConfigConfig> for Macros {
    fn from_build_config(&mut self, mut macro_config: PreprocessorConfigConfig) {
        if let Some(defines) = macro_config.defines.take() {
            for def in defines {
                self.def.insert(def);
            }
        }
        if let Some(undefines) = macro_config.undefines.take() {
            for undef in undefines {
                self.undef.insert(undef);
            }
        }
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

impl FromBuildConfigs<PreprocessorConfigConfig> for PreprocessorConfig {
    fn from_build_config(&mut self, config_data: PreprocessorConfigConfig) {
        self.macros.from_build_config(config_data);
    }
}

// --- Language Configuration ---
#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
// will be checked from the lang option in the configs
pub enum LangX {
    C(i16),   // version
    Cpp(i16), // version
    Asm, // not in support yet, but a future consideration
    AsmCpp, // not in support yet, but a future consideration
}

impl ToCompilerArgs for LangX {
    fn to_args(&self) -> Vec<String> {
        match self {
            // Emit -std flag for any positive version (future-proof for new standards)
            LangX::C(version) if *version > 0 => vec![format!("-std=c{}", version)],
            LangX::Cpp(version) if *version > 0 => vec![format!("-std=c++{}", version)],
            LangX::C(_) | LangX::Cpp(_) => vec![],
            LangX::Asm | LangX::AsmCpp => vec![],
        }
    }
}


impl Default for LangX {
    fn default() -> Self {
        LangX::Cpp(17) // Default to C++17
    }
}

#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

impl FromBuildConfigs<LanguageConfigConfig> for LanguageConfig {
    fn from_build_config(&mut self, congigs_data :LanguageConfigConfig)  {
            if let Some(standard) = congigs_data.langx {
                self.standard = standard;
            }
        if let Some(exc) = congigs_data.exception {
            self.exception = exc;
        }
        if let Some(strict) = congigs_data.strict_checking {
            self.strict_checking = strict;
        }
        if let Some(lax_vec) = congigs_data.lax_vector_conversions {
            self.lax_vector_conversions = lax_vec;
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
#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

impl FromBuildConfigs<CompilationControlConfig> for CompilationControl {
    fn from_build_config(&mut self, config_data: CompilationControlConfig) {
        if let Some(mode) = config_data.compilation_mode {
            self.compilation_mode = mode;
        }
        if let Some(dep_info) = config_data.dependency_info {
            self.dependency_info = dep_info;
        }
        if let Some(pipe) = config_data.use_pipe {
            self.use_pipe = pipe;
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

impl FromBuildConfigs<LoggingOptionsConfig> for LoggingOptions {
    fn from_build_config(&mut self, config_data: LoggingOptionsConfig) {
        if let Some(verb) = config_data.verbose {
            self.verbose = verb;
        }
        if let Some(show_cmds) = config_data.show_commands {
            self.show_commands = show_cmds;
        }
    }
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

impl FromBuildConfigs<ControlConfigConfig> for ControlConfig {
    fn from_build_config(&mut self, config_data: ControlConfigConfig) {
        if let Some(control_cfg) = config_data.control {
            self.control.from_build_config(control_cfg);
        }
        if let Some(logging_cfg) = config_data.logging {
            self.logging.from_build_config(logging_cfg);
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
#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

impl FromBuildConfigs<OptimizationOptionConfig> for OptimizationOptions {
    fn from_build_config(&mut self, config_data: OptimizationOptionConfig) {
        if let Some(level) = config_data.level {
            self.level = level;
        }
        if let Some(march) = config_data.march {
            self.march = Some(march);
        }
        if let Some(mtune) = config_data.mtune {
            self.mtune = Some(mtune);
        }
        if let Some(lto) = config_data.lto {
            self.lto = lto;
        }
        if let Some(fpic) = config_data.fpic {
            self.fpic = fpic;
        }
        if let Some(funroll) = config_data.funroll_loops {
            self.funroll_loops = funroll;
        }
        if let Some(fomit) = config_data.fomit_frame_pointer {
            self.fomit_frame_pointer = fomit;
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

impl FromBuildConfigs<MachineTargetConfig> for MachineTarget {
    fn from_build_config(&mut self, config_data: MachineTargetConfig) {
        if let Some(triple) = config_data.target_triple {
            self.target_triple = Some(triple);
        }
        if let Some(cpu) = config_data.cpu {
            self.cpu = Some(cpu);
        }
        if let Some(features) = config_data.features {
            self.features = features;
        }
    }
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

impl FromBuildConfigs<OptimizationConfigConfig> for OptimizationConfig {
    fn from_build_config(&mut self, config_data: OptimizationConfigConfig) {
        if let Some(options_cfg) = config_data.options {
            self.options.from_build_config(options_cfg);
        }
        if let Some(target_cfg) = config_data.target {
            self.target.from_build_config(target_cfg);
        }
    }
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
#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

// impl FromBuildConfigs<DebugOptionsConfig> for DebugOptions {
//     fn from_build_config(&mut self, config_data: DebugOptionsConfig) {
//         if let Some(level) = config_data.level {
//             self.level = level;
//         }
//     }
// }

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

// impl FromBuildConfigs<WarningOptionsConfig> for WarningOptions {
//     fn from_build_config(&mut self, config_data: WarningOptionsConfig) {
//         if let Some(wall) = config_data.wall {
//             self.wall = wall;
//         }
//         if let Some(wextra) = config_data.wextra {
//             self.wextra = wextra;
//         }
//         if let Some(werror) = config_data.werror {
//             self.werror = werror;
//         }
//         if let Some(wpedantic) = config_data.wpedantic {
//             self.wpedantic = wpedantic;
//         }
//         if let Some(pedantic_errors) = config_data.pedantic_errors {
//             self.pedantic_errors = pedantic_errors;
//         }
//     }
// }

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

// impl FromBuildConfigs<SanitizerOptionsConfigs> for SanitizerOptions {
//     fn from_build_config(&mut self, config_data: SanitizerOptionsConfigs) {
//         if let Some(address) = config_data.address {
//             self.address = address;
//         }
//         if let Some(thread) = config_data.thread {
//             self.thread = thread;
//         }
//         if let Some(memory) = config_data.memory {
//             self.memory = memory;
//         }
//         if let Some(undefined) = config_data.undefined {
//             self.undefined = undefined;
//         }
//         if let Some(leak) = config_data.leak {
//             self.leak = leak;
//         }
//     }
// }

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

impl FromBuildConfigs<DiagnosticConfigConfig> for DiagnosticConfig {
    fn from_build_config(&mut self, config_data: DiagnosticConfigConfig) {
        if let Some(level) = config_data.debug_level {
            self.debug.level = level;
        }
        if let Some(wall) = config_data.wall {
            self.warnings.wall = wall;
        }
        if let Some(wextra) = config_data.wextra {
            self.warnings.wextra = wextra;
        }
        if let Some(werror) = config_data.werror {
            self.warnings.werror = werror;
        }
        if let Some(wpedantic) = config_data.wpedantic {
            self.warnings.wpedantic = wpedantic;
        }
        if let Some(pedantic_errors) = config_data.pedantic_errors {
            self.warnings.pedantic_errors = pedantic_errors;
        }
        if let Some(address) = config_data.sanitizer_address {
            self.sanitizers.address = address;
        }
        if let Some(thread) = config_data.sanitizer_thread {
            self.sanitizers.thread = thread;
        }
        if let Some(memory) = config_data.sanitizer_memory {
            self.sanitizers.memory = memory;
        }
        if let Some(undefined) = config_data.sanitizer_undefined {
            self.sanitizers.undefined = undefined;
        }
        if let Some(leak) = config_data.sanitizer_leak {
            self.sanitizers.leak = leak;
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
#[derive(Debug, Clone,serde::Serialize, serde::Deserialize)]
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

impl FromBuildConfigs<LinkerConfigConfig> for LinkerConfig {
    fn from_build_config(&mut self, config_data: LinkerConfigConfig) {
        if let Some(mode) = config_data.mode {
            self.mode = mode;
        }
        // RuntimeOptions
        if let Some(static_libgcc) = config_data.static_libgcc {
            self.runtime.static_libgcc = static_libgcc;
        }
        if let Some(static_libstdcpp) = config_data.static_libstdcpp {
            self.runtime.static_libstdcpp = static_libstdcpp;
        }
        if let Some(pthread) = config_data.pthread {
            self.runtime.pthread = pthread;
        }
        // ABIOptions
        if let Some(abi_version) = config_data.abi_version {
            self.abi.abi_version = Some(abi_version);
        }
        if let Some(no_gnu_unique) = config_data.no_gnu_unique {
            self.abi.no_gnu_unique = no_gnu_unique;
        }
        if let Some(no_common) = config_data.no_common {
            self.abi.no_common = no_common;
        }
        // LinkerOptions
        if let Some(as_needed) = config_data.as_needed {
            self.options.as_needed = as_needed;
        }
        if let Some(no_undefined) = config_data.no_undefined {
            self.options.no_undefined = no_undefined;
        }
        if let Some(gc_sections) = config_data.gc_sections {
            self.options.gc_sections = gc_sections;
        }
        if let Some(extra_flags) = config_data.extra_flags {
            self.options.extra_flags = extra_flags;
        }
        // LinkPaths
        if let Some(lib_paths) = config_data.lib_paths {
            self.paths.lib_paths = lib_paths;
        }
        if let Some(libs) = config_data.libs {
            self.paths.libs = libs;
        }
        if let Some(rpath) = config_data.rpath {
            self.paths.rpath = rpath;
        }
        // Strip
        if let Some(strip) = config_data.strip {
            self.strip = strip;
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
    
    // FIXME: 
    pub fn set_langx(&mut self, langx: LangX) {
        self.language.standard = langx;
    }

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

impl FromBuildConfigs<BuildProfileConfig> for BuildSettings {
    fn from_build_config(&mut self, mut config_data: BuildProfileConfig) {
        if let Some(preprocessor_cfg) = config_data.preprocessor.take() {
            self.preprocessor.from_build_config(preprocessor_cfg);
        }
        if let Some(language_cfg) = config_data.language.take() {
            self.language.from_build_config(language_cfg);
        }
        if let Some(optimization_cfg) = config_data.optimization.take() {
            self.optimization.from_build_config(optimization_cfg);
        }
        if let Some(diagnostics_cfg) = config_data.diagnostics.take() {
            self.diagnostics.from_build_config(diagnostics_cfg);
        }
        if let Some(linking_cfg) = config_data.linking.take() {
            self.linking.from_build_config(linking_cfg);
        }
        if let Some(control_cfg) = config_data.control.take() {
            self.control.from_build_config(control_cfg);
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

pub fn build_profile_as_per(profile_name: &Option<String>) -> BuildSettings {
    match profile_name {
        Some(profile) => {
            match profile.to_lowercase().as_str() {
                profile  if profile.starts_with("release") => {
                    BuildSettings::release()
                }
                profile if profile.starts_with("debug") => BuildSettings::debug(),
                profile if profile.starts_with("fast") => BuildSettings::fast(),
                profile if profile.starts_with("object") => BuildSettings::object(),
                _ => BuildSettings::default(),
            }
        }
        None => BuildSettings::default(),
    }
}


//-----------------------TOML COFIGURABLE STRUCTS-----------------------//
// USER CONFIGURABLE BUILD PROFILE STRUCTS FOR SERDE

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildProfileConfig {
    pub preprocessor: Option<PreprocessorConfigConfig>,
    pub language: Option<LanguageConfigConfig>,
    pub optimization: Option<OptimizationConfigConfig>,
    pub diagnostics: Option<DiagnosticConfigConfig>,
    pub linking: Option<LinkerConfigConfig>,
    pub control: Option<ControlConfigConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildProfileConfigs {
    pub profiles: HashMap<String, BuildProfileConfig>,
}

//REVIEW: need to review it , need to consider the usage and allowed profiles names as `release`, `debug`, `fast, `object`
impl BuildProfileConfigs {
    pub fn get(&self, profile_name: &str) -> Option<&BuildProfileConfig> {
        self.profiles.get(profile_name)
    }
}


// SUB CONFIGS

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreprocessorConfigConfig {
    pub defines: Option<Vec<String>>,
    pub undefines: Option<Vec<String>>,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct LanguageConfigConfig {
    pub langx   : Option<LangX>,
    pub exception: Option<LangException>,
    pub strict_checking: Option<bool>,
    pub lax_vector_conversions: Option<bool>,
}



#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MachineTargetConfig {
    pub target_triple: Option<String>,
    pub cpu: Option<String>,
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizationOptionConfig {
    pub level: Option<OptLevel>,
    pub march: Option<String>,
    pub mtune: Option<String>,
    pub lto: Option<Lto>,
    pub fpic: Option<bool>,
    pub funroll_loops: Option<bool>,
    pub fomit_frame_pointer: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizationConfigConfig {
    pub options: Option<OptimizationOptionConfig>,
    pub target: Option<MachineTargetConfig>,
}




#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticConfigConfig {
    pub debug_level: Option<DebugLevel>,
    pub wall: Option<bool>,
    pub wextra: Option<bool>,
    pub werror: Option<bool>,
    pub wpedantic: Option<bool>,
    pub pedantic_errors: Option<bool>,
    pub sanitizer_address: Option<bool>,
    pub sanitizer_thread: Option<bool>,
    pub sanitizer_memory: Option<bool>,
    pub sanitizer_undefined: Option<bool>,
    pub sanitizer_leak: Option<bool>,
}



#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkerConfigConfig {
    pub mode: Option<LinkMode>,
    pub static_libgcc: Option<bool>,
    pub static_libstdcpp: Option<bool>,
    pub pthread: Option<bool>,
    pub abi_version: Option<String>,
    pub no_gnu_unique: Option<bool>,
    pub no_common: Option<bool>,
    pub as_needed: Option<bool>,
    pub no_undefined: Option<bool>,
    pub gc_sections: Option<bool>,
    pub extra_flags: Option<Vec<String>>,
    pub lib_paths: Option<Vec<String>>,
    pub libs: Option<Vec<String>>,
    pub rpath: Option<Vec<String>>,
    pub strip: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompilationControlConfig {
    pub compilation_mode: Option<CompilationMode>,
    pub dependency_info: Option<DependencyInfoBuild>,
    pub use_pipe: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingOptionsConfig {
    pub verbose: Option<bool>,
    pub show_commands: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ControlConfigConfig {
    pub control: Option<CompilationControlConfig>,
    pub logging: Option<LoggingOptionsConfig>,
}
