use super::build_settings::BuildSettings;
use super::includes::IncludeFiles;
use super::trailt::ToCompilerArgs;
use crate::config::Configs;
use crate::state::ProjectStructure;

//TODO: need to add the prevention for the no file args case and only dir args case and only header files case
pub struct Compiler {
    pub compiler_path: String,
    pub main_path: String,
    pub include_files: IncludeFiles,
    pub settings: BuildSettings,
    pub build_name: String,
    // pub output_path: Option<String>,
}

impl Compiler {
    pub fn new(
        main_file_path: &str,
        include_files: IncludeFiles,
        build_setting: BuildSettings,
        config: &Configs,
        project_structure: &mut ProjectStructure,
        // output_path: Option<&str>,
    ) -> Self {
        let compiler_path = config
            .compiler_path()
            .unwrap_or_else(|| panic!("No compiler PROVIDED"));

        let project_path = config.get_root_path();
        let exe_name = std::path::Path::new(main_file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("a.out");

        // if !std::fs::exists(format!("{}/build", project_path)).unwrap_or(false) {
        //     std::fs::create_dir_all(format!("{}/build", project_path)).unwrap();
        // }
        //instead using this
        if !project_structure.exists(std::path::Path::new("build")) {
            project_structure
                .create_dir(std::path::Path::new("build"))
                .unwrap();
        }

        let build_name = format!("{}/build/{}.exe", project_path.display(), exe_name);

        Compiler {
            compiler_path,
            main_path: main_file_path.to_string(),
            include_files,
            settings: build_setting,
            build_name,
            // output_path: output_path.map(|s| s.to_string()),
        }
    }

    pub fn default(_compiler_path: &str) -> Self {
        todo!()
    }

    pub fn run_build(&self) -> Self {
        todo!()
    }

    pub fn release_build(&self) -> Self {
        todo!()
    }

    pub fn debug_build(&self) -> Self {
        todo!()
    }

    pub fn fast_build(&self) -> Self {
        todo!()
    }

    pub fn custom_build(_path_to_toml: &str) -> Self {
        todo!("custom build from toml file not implemented yet")
    }

    // pub fn set_output_path(&mut self, output_path: &str) {
    //     self.output_path = Some(output_path.to_string());
    // }
    pub fn compile_with_options(
        &self,
        _opt_level: u8,
        _std: u8,
        _i_extra: &Vec<String>,
        _build_profile: &str,
    ) -> Result<String, String> {
        todo!("compile with options not implemented yet")
    }
    
    pub fn compile(&self) -> Result<String, String> {
        use std::fs;
        use std::path::Path;
        use std::process::Command;
        
        if self.include_files.source_files.is_empty() {
            return Err("No source files provided for compilation.".to_string());
        }

        // Ensure the build directory exists
        if let Some(parent) = Path::new(&self.build_name).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create build directory: {}", e));
                }
            }
        }

        #[cfg(any(test, debug_assertions))]
        let output = {
            println!("Compiling with command:");
            let mut debug_command = format!("{}", &self.compiler_path);
            for arg in self.settings.to_args() {
                debug_command.push_str(&format!(" {}", arg));
            }
            for arg in self.include_files.to_args() {
                debug_command.push_str(&format!(" {}", arg));
            }
            debug_command.push_str(&format!(" -o {}", &self.build_name));
            println!("[Compiler: compile : g++ commands : {}", debug_command);

            let mut cmd = Command::new(&self.compiler_path);
            cmd.args(self.settings.to_args())
                .args(self.include_files.to_args())
                .arg("-o")
                .arg(&self.build_name);
            println!("\t [Compiler : compile]: Command struct built");
            dbg!(&cmd);
            cmd.output()
        };

        #[cfg(not(any(test, debug_assertions)))]
        let output = Command::new(&self.compiler_path)
            // Add flags from build settings first (standards, -fsyntax-only, etc.)
            .args(self.settings.to_args())
            // Then include dirs and source files
            .args(self.include_files.to_args())
            .arg("-o")
            .arg(&self.build_name)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(self.build_name.to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
