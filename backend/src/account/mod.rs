pub mod api;
pub mod codes;
pub mod data;
pub mod login;
pub mod recovery;
pub mod register;
pub mod session;
pub mod tokens;
pub mod types;

// Compatibility aliases while callers are migrated to the functional tree.
pub mod dto {
    pub use super::types::*;
}
pub mod model {
    pub use super::data::User;
}
pub mod repo {
    pub use super::data::*;
}
pub mod service {
    pub use super::login::*;
}

pub use api::routes;
pub use login::{bootstrap_admin, change_password, login, me};
