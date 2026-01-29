pub trait ToCompilerArgs {
    fn to_args(&self) -> Vec<String>;
}

pub trait FromBuildConfigs<T>{
    fn from_build_config(&mut self, congigs_data :T) ;
}