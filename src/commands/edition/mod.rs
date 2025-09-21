//! Edition management commands

mod analyze;
mod check;
mod migrate;

pub use analyze::handle_analyze;
pub use check::handle_check;
pub use migrate::handle_migrate;
