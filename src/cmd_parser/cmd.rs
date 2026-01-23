#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Build,
    Run,
    Check,
    Init,
    Clean,
    Log,
    Env,
    Config,
    Help,
    Version,
    Previous,
    Watcher,
}
