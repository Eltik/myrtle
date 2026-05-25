//! Generated DPS + HPS operator implementations.
//!
//! The `generated*` modules are the transpiled per-operator function bodies.
//! The `dispatch*` modules are the auto-generated `match` tables that route a
//! `char_id` to the right body. All four files are produced by
//! `cargo run --bin generate-dps` — this `mod.rs` is hand-written and stable.

mod dispatch;
mod dispatch_hps;
mod generated;
mod generated_hps;

pub use dispatch::dispatch;
pub use dispatch_hps::dispatch as dispatch_hps;
