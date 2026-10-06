//! Trevor: the Arknights lore corpus builder and retrieval service.
//!
//! The crate is a sibling of `backend/` and `discord/`, with no path
//! dependency on either. It talks to the backend over HTTP, exactly as the
//! discord bot does.

pub mod corpus;
pub mod eval;
pub mod goldgen;
pub mod reference;
pub mod router;
pub mod search;
pub mod tools;
pub mod util;
