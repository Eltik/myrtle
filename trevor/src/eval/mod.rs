//! The retrieval evaluation harness. No model and no network in here: the
//! bins hand it a retrieval function, and everything else is arithmetic, so
//! a run is reproducible bit for bit.

pub mod golden;
pub mod goldset;
pub mod metrics;
pub mod run;
pub mod stats;
