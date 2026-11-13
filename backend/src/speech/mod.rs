pub mod cache;
pub mod http;
pub mod providers;
pub mod synthesize;
pub mod synthesize_http;
pub mod types;

pub mod dto {
    pub use super::types::*;
}
pub mod handler {
    pub use super::synthesize_http::*;
}
pub mod routes {
    pub use super::http::*;
}
pub mod service {
    pub use super::synthesize::*;
}
pub mod provider {
    pub use super::providers::*;
}

pub use http::routes;
pub use synthesize::{create_chapter_job, create_segment_job, run_tts};
