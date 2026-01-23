pub mod path_utils;
pub mod utils;

pub use path_utils::*;
pub use utils::*;

// Module aliases for backward compatibility
pub mod trim {
    pub use super::utils::quotes;
}

pub mod chrono {
    pub use super::utils::local_time;
    pub use super::utils::time_stamp;
}

pub mod truncate {
    pub use super::utils::truncate;
}

pub mod prompt {
    pub use super::utils::input;
}
