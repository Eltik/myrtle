//! Generated DPS + HPS operator implementations.
//!
//! `generated*` holds the transpiled per-operator bodies, `dispatch*` the `match`
//! tables routing a `char_id` to them. All four come from
//! `cargo run --bin generate-dps`; this `mod.rs` is hand-written.

mod dispatch;
mod dispatch_hps;
mod generated;
mod generated_hps;
mod init;

pub use dispatch::dispatch;
pub use dispatch_hps::dispatch as dispatch_hps;
pub use init::apply as apply_init;
