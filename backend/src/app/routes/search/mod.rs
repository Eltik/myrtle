//! User search route
//!
//! Provides endpoints for searching users with various filters,
//! sorting options, and pagination.

pub mod filters;
pub mod handler;
pub mod query_builder;
pub mod types;

pub use handler::search_users;
