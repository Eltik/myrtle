//! The `FlatBuffers` schema store and the tooling around it.
//!
//! `opts` holds the one definition of the verifier options; `fields` reads
//! the field and enum types the verifier cannot check; `store` the
//! on-disk layout and the helpers `generate-fbs` shares with the sync;
//! `reflect` verifies a buffer against a `.fbs` through a compiled binary
//! schema; `sync` is `unpacker schema-sync`; `self_check` proves `reflect`
//! agrees with the generated decoder.

pub mod fields;
pub mod r#gen;
pub mod il2cpp;
pub mod opts;
pub mod reflect;
pub mod self_check;
pub mod store;
pub mod sync;
