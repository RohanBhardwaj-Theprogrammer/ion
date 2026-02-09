

enum Compler {
    Gcc(String),
    Clang(String),
    Gpp(String),
}

pub struct GlobalConfigDummy {
    pub compilers : Vec<Compler>,
}

impl GlobalConfigDummy {
    pub fn new() -> Self {
        GlobalConfigDummy {
            compilers: vec![
                Compler::Gcc("gcc".to_string()),
                Compler::Clang("clang".to_string()),
                Compler::Gpp("g++".to_string()),
            ],
        }
    }
}