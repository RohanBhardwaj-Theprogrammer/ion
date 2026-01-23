pub trait ToCompilerArgs {
    fn to_args(&self) -> Vec<String>;
}
