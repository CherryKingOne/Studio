pub mod data;
pub mod detect_rules;
pub mod http;
pub mod manage;
pub mod manage_http;
pub mod responses;
pub mod types;

// Temporary source-level aliases keep cross-feature calls explicit while the
// remaining modules are migrated to operation-oriented names.
pub mod dto {
    pub use super::responses::*;
}
pub mod detector {
    pub use super::detect_rules::*;
}
pub mod model {
    pub use super::types::*;
}
pub mod repo {
    pub use super::data::*;
}
pub mod service {
    pub use super::manage::*;
}

pub use http::routes;
