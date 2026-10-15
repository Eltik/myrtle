//! The one definition of the `FlatBuffers` verifier options.
//!
//! The generated decoder (`flatbuffers_decode::verify_schemas`) and
//! `unpacker schema-sync` both import `verifier_opts` from here, so a schema the
//! sync accepts is a schema `extract` accepts. `generate-fbs` emits a `use` of
//! this function, never a copy of it.

/// Verifier options for schema selection and for the schema sync.
///
/// Deliberately generous. These buffers are real game data, not adversarial
/// input, and the only question being asked is "do the offsets in this buffer
/// make sense under this schema". A false NEGATIVE would push a perfectly good
/// table onto the skip path, so every limit sits far above what any real table
/// needs; `max_apparent_size` is the flatbuffers default (2 GiB).
///
/// `max_alignment` is not an upstream flatbuffers option; it comes from the
/// vendored copy in `vendor/flatbuffers` (see MYRTLE-PATCH.md). Hypergryph's
/// serializer aligns 8-byte scalars to 4 bytes, so upstream's `is_aligned`
/// rejects a buffer the crate then reads back perfectly well through its
/// unaligned scalar reads: `roguelike_topic_table` failed verification under
/// BOTH schemas on BOTH servers with `Type f64 at position N is unaligned`,
/// N % 8 == 4 (CN 7819508, EN 7285868). Capping the demand at 4 accepts that
/// layout and leaves every other check untouched.
///
/// `ignore_utf8_errors` is the second vendored option. Upstream rejects a
/// buffer when a string's BYTES do not decode as UTF-8, which is right for a
/// reader that hands the `&str` to code assuming valid UTF-8. Ours does not:
/// every string reaching the emitter goes through `fb_json_macros::json_str`,
/// i.e. `String::from_utf8_lossy`, so the bad byte becomes U+FFFD and nothing
/// downstream can observe it. An unverified table is SKIPPED rather than
/// decoded, so a content check the emitter already handles must not be allowed
/// to cost a whole table.
///
/// `ignore_missing_null_terminator` stays FALSE, deliberately. It is tempting
/// for the same reason (this reader is length-prefixed and never scans for a
/// NUL) but it is measurably load-bearing for schema SELECTION. The
/// `battle/level_script_table` buffer verifies under its own schema and fails
/// `prts___levels` on exactly that check; with the option on it verifies under
/// BOTH and the wrong one wins. EN `roguelike_topic_table` and `building_data`
/// likewise fail the CN schema only on a missing null terminator, and routing
/// them back to CN is the multi-GB garbage this whole mechanism exists to
/// prevent. A missing terminator is a good discriminator; a bad UTF-8 byte is
/// not.
#[must_use]
pub const fn verifier_opts() -> ::flatbuffers::VerifierOptions {
    ::flatbuffers::VerifierOptions {
        max_depth: 256,
        max_tables: usize::MAX >> 1,
        max_apparent_size: 1 << 31,
        ignore_missing_null_terminator: false,
        max_alignment: 4,
        ignore_utf8_errors: true,
    }
}
