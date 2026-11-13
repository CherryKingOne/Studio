pub mod endpoints;
pub mod http;
pub mod merge_chapter;
pub mod types;

pub mod dto {
    pub use super::types::*;
}
pub mod handler {
    pub use super::endpoints::*;
}
pub mod routes {
    pub use super::http::*;
}
pub mod service {
    pub use super::merge_chapter::*;
}

pub use http::routes;
pub use merge_chapter::{create_merge_job, run_merge};
