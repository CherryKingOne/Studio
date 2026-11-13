pub mod api;
pub mod segment_actions;
pub mod segment_data;
pub mod segment_types;

pub mod dto {
    pub use super::segment_types::*;
}
pub mod model {
    pub use super::segment_data::*;
}
pub mod repo {
    pub use super::segment_data::*;
}
pub mod service {
    pub use super::segment_actions::*;
}
pub mod handler {
    pub use super::api::*;
}

pub use api::routes;
