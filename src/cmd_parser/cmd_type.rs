
pub enum CmdType {
    Run(String, String, String),
    Build(String, String, bool),
    Clean(String),
    Check(String),
    Init(String),
    Help,
}