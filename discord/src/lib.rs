// Poise requires command bodies, parent-command stubs, hooks, and checks to be
// `async fn`s regardless of whether they await - opting out of this lint here
// avoids littering the codebase with `#[allow]`s on every command/hook.
#![allow(clippy::unused_async)]
// Codebase convention: error/panic docs are omitted on internal helpers. The
// public CLI entry point and `Config::load*` callers handle errors at the call
// site, and panics on bot startup are intentional.
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
// Style preference: `match` on `Option` reads more clearly than `map_or_else`
// with two large closures, especially when one branch is a fallback chain.
#![allow(clippy::option_if_let_else)]

pub mod api;
pub mod checks;
pub mod cmds;
pub mod config;
pub mod db;
pub mod handler;
pub mod hooks;
pub mod types;
