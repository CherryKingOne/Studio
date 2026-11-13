pub mod exports;
pub mod files;

pub use exports::service as export_service;
pub use files::service as file_service;

// Functional modules use these names for the shared file registration
// capability. The implementation lives with delivery/files.
pub mod repo {
    pub use super::files::repo::*;
}

pub mod service {
    pub use super::files::service::*;
}
