pub mod api;
pub mod book_actions;
pub mod book_data;
pub mod book_types;

pub mod dto {
    pub use super::book_types::*;
}
pub mod model {
    pub use super::book_data::*;
}
pub mod repo {
    pub use super::book_data::*;
}
pub mod service {
    pub use super::book_actions::*;
}
pub mod handler {
    pub use super::api::*;
}

pub use api::routes;
