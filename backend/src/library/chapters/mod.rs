pub mod api;
pub mod chapter_actions;
pub mod chapter_data;
pub mod chapter_types;

pub mod dto {
    pub use super::chapter_types::*;
}
pub mod model {
    pub use super::chapter_data::*;
}
pub mod repo {
    pub use super::chapter_data::*;
}
pub mod service {
    pub use super::chapter_actions::*;
}
pub mod handler {
    pub use super::api::*;
}

pub use api::routes;
