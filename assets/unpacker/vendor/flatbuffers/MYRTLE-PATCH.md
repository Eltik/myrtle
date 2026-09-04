# MYRTLE PATCH — flatbuffers 25.12.19

Vendored copy of the crates.io `flatbuffers` 25.12.19 source, wired in through
`[patch.crates-io]` in `assets/unpacker/Cargo.toml`. Two patches, both in
`src/verifier.rs`, each adding one `VerifierOptions` field whose default is
upstream behaviour: `max_alignment` and `ignore_utf8_errors`.

Both matter more than they look, because the unpacker no longer has an
unchecked fallback: a table that verifies under no schema is SKIPPED rather
than decoded (see `decode_flatbuffer` in `src/flatbuffers_decode.rs`). A
verifier that rejects a buffer it could read perfectly well now costs a table.

## Patch 1 — `max_alignment`

### Why

The unpacker picks the CN or the Yostar (EN/Global) schema for a gamedata table
by *verifying* the buffer before it decodes anything (see
`select_schema_by_verification` in `src/flatbuffers_decode.rs`). Verification is
the only signal that fires before a mis-schema'd decode materialises gigabytes
of garbage JSON.

`roguelike_topic_table` verified under **neither** schema on **either** server,
so it had no route to a correct decode: the CN schema is right for the CN
binary, but the verifier rejected it, and on EN the resulting unchecked decode
was not even valid UTF-8.

The rejection is always the same, and it is not a schema mismatch:

    Type f64 at position 7819508 is unaligned   (CN 26-09-03-04-06-00_ed95a2)
    Type f64 at position 7285868 is unaligned   (EN 26-08-28-10-20-08_ea3678)

`7819508 % 8 == 4` and `7285868 % 8 == 4`. Both are 4-aligned, not 8-aligned.
Hypergryph's serializer aligns 8-byte scalars (`double`/`long`) to 4 bytes
rather than to 8. That is out of spec for FlatBuffers, but it is harmless for
reading: this crate reads every scalar through `EndianScalar::from_le` on a
`read_scalar_at` that does an **unaligned** `ptr::read_unaligned`, so a
4-aligned `f64` decodes correctly. Only the verifier objects.

### The patch

`VerifierOptions` gains one field:

```rust
    /// MYRTLE PATCH. Upper bound on the alignment `is_aligned` demands of a
    /// scalar, in bytes. The effective requirement is
    /// `align_of::<T>().min(max_alignment)`, so the default `usize::MAX` is
    /// exactly upstream behaviour (natural alignment) and cannot change any
    /// verdict.
    pub max_alignment: usize,
```

`Default` sets it to `usize::MAX`, and `is_aligned` caps the demanded alignment
with it:

```rust
-        if pos % core::mem::align_of::<T>() == 0 {
+        // MYRTLE PATCH: cap the demanded alignment at `opts.max_alignment`.
+        // `usize::MAX` (the default) leaves `align_of::<T>()` untouched.
+        if pos % core::mem::align_of::<T>().min(self.opts.max_alignment).max(1) == 0 {
```

The `.max(1)` only stops `max_alignment: 0` from being a modulo-by-zero panic.

With `max_alignment: usize::MAX` the verifier is bit-for-bit upstream. The
unpacker sets `max_alignment: 4` in `verifier_opts()` (emitted into
`src/flatbuffers_decode.rs` by `src/bin/generate_fbs.rs`), which relaxes the
demand for `f64`/`i64`/`u64` to 4 bytes and leaves every smaller scalar,
every vtable offset and every bounds check exactly as upstream.

## Patch 2 — `ignore_utf8_errors`

### Why

`&str`'s verifier runs `core::str::from_utf8` over the string's bytes and
rejects the whole buffer if they do not decode. That check is right for a
reader that hands the `&str` to code assuming valid UTF-8. Ours does not: every
string reaching the JSON emitter goes through `fb_json_macros::json_str`, i.e.
`String::from_utf8_lossy`, so a bad byte becomes U+FFFD and nothing downstream
can observe it.

Measured: CN `level_data` had one file of 2907 (`battle/level_script_table`)
failing verification with

    Utf8 error for string in 52..55: invalid utf-8 sequence of 1 bytes from index 0

The bounds of that string verify; only its three content bytes do not decode.
Rejecting the buffer over that used to mean "fall through to the unchecked
decode"; now it would mean "skip the table entirely", which is a real loss for
a defect the emitter already handles.

### The patch

```rust
    /// MYRTLE PATCH. Skip the UTF-8 content check on strings. The default
    /// `false` is exactly upstream behaviour. Only meaningful for a reader that
    /// never hands a `&str` to code assuming valid UTF-8; ours goes through
    /// `String::from_utf8_lossy`.
    pub ignore_utf8_errors: bool,
```

`Default` sets it to `false`, and `&str`'s `run_verifier` guards the content
check with it:

```rust
-        let s = core::str::from_utf8(&v.buffer[range.clone()]);
-        if let Err(error) = s {
-            return Err(InvalidFlatbuffer::Utf8Error { error, range, error_trace: Default::default() });
-        }
+        // MYRTLE PATCH: `ignore_utf8_errors` skips the content check. The
+        // bounds of the string have already been verified above; only whether
+        // the bytes decode is in question, and a lossy reader does not care.
+        if !v.opts.ignore_utf8_errors {
+            let s = core::str::from_utf8(&v.buffer[range.clone()]);
+            if let Err(error) = s {
+                return Err(InvalidFlatbuffer::Utf8Error { error, range, error_trace: Default::default() });
+            }
+        }
```

The string's `range` is still bounds-checked by `verify_vector_range::<u8>`
immediately above, and the null-terminator check is untouched. Only the content
validation is skipped. `verifier_opts()` sets `ignore_utf8_errors: true`.

## Re-vendoring

Copy `~/.cargo/registry/src/*/flatbuffers-<version>` over `vendor/flatbuffers`,
re-apply the hunks above (three for `max_alignment`, three for
`ignore_utf8_errors`), and keep this file.
