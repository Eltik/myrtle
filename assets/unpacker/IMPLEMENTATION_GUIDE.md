# Unpacker Implementation Guide

## Phase 1: Binary Reader + Compression

### Step 1: `src/unity/endian_reader.rs`

This is your foundation. Everything else reads through this.

```rust
pub struct EndianReader {
    data: Vec<u8>,
    pos: usize,
    big_endian: bool,
}
```

**Methods you need:**
- `new(data, big_endian)`, `position()`, `set_position(pos)`, `remaining()`
- `read_u8`, `read_i16`, `read_u16`, `read_i32`, `read_u32`, `read_i64`, `read_u64` — all respecting `big_endian`
- `read_bool` — `read_u8() != 0`
- `read_cstring` — read bytes until `0x00`, return `String`
- `read_string` — read `read_i32()` length, then that many bytes as UTF-8
- `read_bytes(count)` — return `Vec<u8>`
- `align(n)` — advance `pos` to next multiple of `n`

**Tip:** Use `byteorder` crate's `ReadBytesExt` trait on a `Cursor<&[u8]>`, or just do manual slicing with `u32::from_be_bytes` / `from_le_bytes`. Manual is simpler and avoids the cursor abstraction.

### Step 2: `src/unity/lz4ak.rs`

Port the nibble-swap + offset-swap fix. ~60 lines. The key:

1. Walk through compressed data token by token
2. Swap each token byte's high/low nibbles
3. Swap each 2-byte match offset (big→little endian)
4. Feed the fixed buffer to `lz4_flex::decompress_size_prepended` or `lz4_flex::block::decompress`

**Test it:** Take a known LZ4AK-compressed block from a modern AB file, verify your fix + decompress matches the expected output.

### Step 3: `src/unity/compression.rs`

```rust
pub fn decompress(data: &[u8], uncompressed_size: usize, compression_type: u32) -> Result<Vec<u8>> {
    match compression_type {
        0 => Ok(data.to_vec()),                           // None
        1 => decompress_lzma(data, uncompressed_size),    // LZMA
        2 | 3 => decompress_lz4(data, uncompressed_size), // LZ4/LZ4HC
        4 => decompress_lz4ak(data, uncompressed_size),   // Arknights custom
        other => bail!("unknown compression type: {other}"),
    }
}
```

For LZMA: use `lzma-rs` crate. The Unity LZMA format has a 5-byte properties header (1 byte props + 4 byte dict size), then the compressed stream. You need to prepend the uncompressed size as a little-endian u64 for the standard LZMA format, or use `lzma_decompress_with_options`.

For LZ4: `lz4_flex::block::decompress(data, uncompressed_size)`.

For LZ4AK: call your `fix_lz4ak` first, then standard LZ4 decompress.

---

## Phase 2: Bundle Parser

### Step 4: `src/unity/bundle.rs`

This is the most important file. Parse the UnityFS header:

```rust
pub struct BundleFile {
    pub files: Vec<BundleEntry>,  // The contained serialized files
}

pub struct BundleEntry {
    pub path: String,
    pub data: Vec<u8>,
}

struct BlockInfo {
    uncompressed_size: u32,
    compressed_size: u32,
    flags: u16,
}

struct DirectoryEntry {
    offset: u64,
    size: u64,
    flags: u32,
    path: String,
}
```

**Parsing flow:**

1. **Read header** (all big-endian initially):
   ```
   signature = read_cstring()  // "UnityFS"
   version = read_u32()        // 6, 7, or 8
   version_player = read_cstring()
   version_engine = read_cstring()
   file_size = read_i64()
   compressed_blocks_info_size = read_u32()
   uncompressed_blocks_info_size = read_u32()
   data_flags = read_u32()
   ```

2. **Alignment** (if version >= 7): `align(16)`

3. **Locate block info**: If `data_flags & 0x80` (BlocksInfoAtTheEnd), seek to `file_size - compressed_blocks_info_size`, otherwise read from current position. Save the position where data blocks start (`data_start`).

4. **Decompress block info**: Use `data_flags & 0x3F` as compression type. Decompress to get the uncompressed block info bytes.

5. **Parse block info** (from the decompressed bytes, big-endian):
   ```
   skip 16 bytes (hash)
   block_count = read_i32()
   for block_count: read BlockInfo { uncompressed_size, compressed_size, flags(u16) }
   directory_count = read_i32()
   for directory_count: read DirectoryEntry { offset(i64), size(i64), flags(u32), path(cstring) }
   ```

6. **Decompress blocks**: Read each compressed block sequentially from the file data (starting at `data_start`). Each block's compression type is `block.flags & 0x3F`. Decompress each and **concatenate** into one big buffer.

7. **Extract files**: For each `DirectoryEntry`, slice `offset..offset+size` from the concatenated buffer. That's one serialized file.

**Memory optimization:** Don't hold all decompressed blocks plus all extracted files. Process one entry at a time if possible — decompress blocks, extract the entry, then parse it before moving to the next.

**ArchiveFlags reference:**
- `0x3F` = compression type mask
- `0x80` = blocks info at end of file
- `0x200` = padding at start (version >= 7, already handled by align(16))
- `0x400` = uses encryption (not needed for Arknights)

---

## Phase 3: Serialized File Parser

### Step 5: `src/unity/serialized_file.rs`

Parse the object table from each extracted file:

```rust
pub struct SerializedFile {
    pub objects: Vec<ObjectInfo>,
    pub types: Vec<SerializedType>,
    pub unity_version: String,
    pub target_platform: i32,
    pub enable_type_tree: bool,
    data: Vec<u8>,         // raw file data for reading objects later
    data_offset: u64,      // where object data starts
}

pub struct ObjectInfo {
    pub path_id: i64,
    pub byte_start: u64,   // absolute position in data
    pub byte_size: u32,
    pub type_index: usize,
    pub class_id: i32,
}

pub struct SerializedType {
    pub class_id: i32,
    pub type_tree: Option<TypeTree>,
}
```

**Header parsing** (starts big-endian, may switch):
```
metadata_size = read_u32()
file_size = read_u32()    // or u64 for version >= 22
version = read_u32()      // serialization version (expect 20-22)
data_offset = read_u32()  // or u64 for version >= 22
endian = read_u8()        // 0=little, 1=big
skip 3 bytes (reserved)
// NOW switch endianness to match `endian` flag
```

For version >= 22, re-read `metadata_size`, `file_size` (u64), and `data_offset` (u64).

**Type table:**
```
read unity_version (cstring)
read target_platform (i32)
read enable_type_tree (bool)
type_count = read_i32()
for type_count:
    class_id = read_i32()
    is_stripped = read_bool() (version >= 16)
    script_type_index = read_i16() (version >= 17)
    // hash bytes: 32 bytes if MonoBehaviour (class_id==114) or script_type_index >= 0
    //             16 bytes otherwise
    if enable_type_tree:
        read type_tree blob (Step 6)
```

**Object table:**
```
object_count = read_i32()
for object_count:
    align(4)  // for version >= 14
    path_id = read_i64()  // version >= 14
    byte_start = if version >= 22 { read_u64() } else { read_u32() as u64 }
    byte_start += data_offset   // IMPORTANT: offset into actual data
    byte_size = read_u32()
    type_index = read_i32() as usize
```

### Step 6: `src/unity/type_tree.rs`

Modern Unity (version >= 21) uses **blob format** for type trees:

```
node_count = read_i32()
string_buffer_size = read_i32()
for node_count:
    version = read_u16()
    level = read_u8()
    type_flags = read_u8()
    type_str_offset = read_u32()
    name_str_offset = read_u32()
    byte_size = read_i32()
    index = read_i32()
    meta_flag = read_u32()
    ref_type_hash = read_u64()  // version >= 19
string_buffer = read_bytes(string_buffer_size)
```

**String offsets:** if bit 31 is set (`offset & 0x80000000`), look up in a **common strings table** (hardcoded). Otherwise, index into `string_buffer`.

**Common strings** (partial list — there are ~1500):
```
0 => "AABB", 5 => "AnimationClip", 19 => "AnimationCurve",
76 => "Array", 81 => "Base", 86 => "bool",
334 => "int", 374 => "map", 400 => "PPtr<Object>",
// etc.
```

You can find the complete list in the old `unity-rs/src/helpers/type_tree_helper.rs` on the main branch. Copy the constant array.

**Build a tree** by using the `level` field — level 0 is root, level 1 is children of root, etc.

### Step 7: `src/unity/object_reader.rs`

Read an object's data using its type tree:

```rust
pub fn read_object(file: &SerializedFile, obj: &ObjectInfo) -> Result<serde_json::Value>
```

Create an `EndianReader` positioned at `obj.byte_start`, then recursively read the type tree:

```rust
fn read_value(reader: &mut EndianReader, node: &TypeTreeNode) -> Value {
    match node.type_name.as_str() {
        "bool" => Value::Bool(reader.read_bool()),
        "SInt8" | "char" => Value::Number(reader.read_i8().into()),
        "UInt8" => Value::Number(reader.read_u8().into()),
        "SInt16" | "short" => Value::Number(reader.read_i16().into()),
        "UInt16" | "unsigned short" => Value::Number(reader.read_u16().into()),
        "SInt32" | "int" => Value::Number(reader.read_i32().into()),
        "UInt32" | "unsigned int" => Value::Number(reader.read_u32().into()),
        "SInt64" | "long long" => Value::Number(reader.read_i64().into()),
        "UInt64" | "unsigned long long" => {
            // JSON can't represent u64 > i64::MAX, store as string
            let v = reader.read_u64();
            if v > i64::MAX as u64 { Value::String(v.to_string()) }
            else { Value::Number((v as i64).into()) }
        }
        "float" => json!(reader.read_f32()),
        "double" => json!(reader.read_f64()),
        "string" => {
            let len = reader.read_i32() as usize;
            let bytes = reader.read_bytes(len);
            reader.align(4);
            Value::String(String::from_utf8_lossy(&bytes).into_owned())
        }
        "Array" => {
            let size = reader.read_i32();
            let element_node = &node.children[1]; // children[0] is size, children[1] is element type
            let mut arr = Vec::with_capacity(size as usize);
            for _ in 0..size {
                arr.push(read_value(reader, element_node));
            }
            Value::Array(arr)
        }
        "map" => {
            // Similar to Array but children are key-value pairs
            let size = reader.read_i32();
            let pair_node = &node.children[0].children[1]; // Array > pair
            let mut map = serde_json::Map::new();
            for _ in 0..size {
                let key = read_value(reader, &pair_node.children[0]);
                let val = read_value(reader, &pair_node.children[1]);
                let key_str = match key { Value::String(s) => s, other => other.to_string() };
                map.insert(key_str, val);
            }
            Value::Object(map)
        }
        "TypelessData" => {
            let size = reader.read_i32() as usize;
            let bytes = reader.read_bytes(size);
            reader.align(4);
            // Store as base64 or as array — base64 is more compact
            Value::String(base64_encode(&bytes))
        }
        _ => {
            // Struct: read each child
            let mut map = serde_json::Map::new();
            for child in &node.children {
                map.insert(child.name.clone(), read_value(reader, child));
            }
            Value::Object(map)
        }
    }
    // Post-read alignment
    if node.meta_flag & 0x4000 != 0 {
        reader.align(4);
    }
}
```

**Important alignment detail:** The alignment check should happen **after** reading, not before. The old unity-rs code applies it after. Check both the existing implementation and test against real files to get this right — it's the most common source of parsing bugs.

**Special case for `TypelessData`**: This is how `image_data` on Texture2D is stored — a raw byte blob. You'll want to keep it as `Vec<u8>` for texture decoding, so consider returning a richer enum instead of `serde_json::Value` if you want to avoid base64 round-tripping. Or just have texture extraction read the raw bytes directly from the reader position.

---

## Phase 4: Exporters

### Step 8: `src/export/texture.rs`

After reading a Texture2D object as JSON, extract the fields:
- `m_Width`, `m_Height`, `m_TextureFormat` (i32 → enum), `image_data` (TypelessData bytes)
- If `image_data` is empty, check `m_StreamData.path` + `offset` + `size` for external .resS files

Then decode based on format:
```rust
match format {
    RGBA32(4) => direct_copy_with_channel_reorder(),
    RGB24(3) => expand_to_rgba(),
    ASTC_4x4(48) | ASTC_5x5(49) | ... => texture2ddecoder::decode_astc(),
    ETC2_RGB(45) => texture2ddecoder::decode_etc2_rgb(),
    ETC2_RGBA8(47) => texture2ddecoder::decode_etc2_rgba8(),
    ETC_RGB4(34) => texture2ddecoder::decode_etc1(),
    _ => bail!("unsupported format: {format}"),
}
```

Save with `image::save_buffer(path, &rgba, w, h, ColorType::Rgba8)`.

**Note:** Unity textures are stored bottom-up (origin at bottom-left). You need to **vertically flip** the decoded image before saving.

### Step 9: `src/export/text_asset.rs`

Read `m_Name` and `m_Script` (raw bytes) from the object. Then reuse your existing `decrypt.rs` logic from the downloader:

1. Try AES-CBC decrypt (skip 128-byte RSA header)
2. Try JSON parse → save as `.json`
3. Try BSON decode → save as `.json`
4. Fallback → save raw bytes with original extension

### Step 9a: FlatBuffer Gamedata Decoding

Arknights gamedata tables (character_table, skill_table, etc.) are stored as **FlatBuffers** inside anonymous `.bin` bundle files, not as plain JSON. The full pipeline:

#### Architecture

```
anon/*.bin (hash-named UnityFS bundles)
    ↓  [Extract TextAsset]
Raw bytes: 128-byte RSA signature + FlatBuffer binary
    ↓  [Manifest lookup]
.idx file maps extracted m_Name (with hash suffix) → real path (gamedata/excel/character_table)
    ↓  [FlatBuffer decode]
JSON output at proper game paths
```

#### The `.idx` Manifest

Located at `ArkAssets/<hash>.idx`. Format:
1. First 128 bytes = RSA signature (skip)
2. Remaining bytes = FlatBuffer containing `ResourceManifest`

The manifest has an `assetToBundleList` with entries:
- `name`: extracted filename with hash suffix (e.g., `"character_tabled88efb"`)
- `path`: real game path (e.g., `"dyn/gamedata/excel/character_tabled88efb.bytes"`)

Build a `HashMap<String, String>` mapping:
- Strip `"dyn/"` prefix and `.bytes` extension
- Strip 6-char hex hash suffix from the end via regex `[a-f0-9]{6}$`
- Result: `"character_tabled88efb"` → `"gamedata/excel/character_table"`

#### FlatBuffer Detection

After skipping the 128-byte RSA header, check if the remaining data is a valid FlatBuffer:
1. Read root table offset (u32 LE at byte 0)
2. Verify it points within the buffer
3. Read vtable offset (i32 LE at root table offset)
4. Verify vtable pointer is valid

```rust
fn is_flatbuffer(data: &[u8]) -> bool {
    if data.len() < 8 { return false; }
    let root_offset = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    if root_offset + 4 > data.len() { return false; }
    let vtable_rel = i32::from_le_bytes(data[root_offset..root_offset+4].try_into().unwrap());
    let vtable_pos = (root_offset as i64 - vtable_rel as i64) as usize;
    vtable_pos + 4 <= data.len()
}
```

#### FlatBuffer Decoding Strategy

Two approaches, from simplest to most complete:

**Option A: Schema-less string extraction (quick & dirty)**
Walk the FlatBuffer and extract all string references. This loses structure but gets the raw data out. Useful as a fallback.

**Option B: Generated schemas (full fidelity)**
The old myrtle repo has ~50+ generated FlatBuffer schema files under `generated_fbs/`. These are Rust modules generated from `.fbs` schema files that define the exact table layouts for each gamedata file.

For Option B:
1. Add `flatbuffers = "24"` to Cargo.toml
2. Port or regenerate the `.fbs` schemas from the old repo
3. Match filename patterns to schema types:
   - `character_table` → `character_table_generated.rs`
   - `skill_table` → `skill_table_generated.rs`
   - etc.
4. Decode using the schema, serialize to JSON

**Recommended approach:** Start with Option A for initial functionality. Add Option B schemas incrementally for the tables you actually need (character_table, skill_table, etc.).

#### Module Structure

```
export/
  flatbuffers_decode.rs   -- FlatBuffer detection + generic decode
  manifest.rs             -- .idx manifest parsing for path resolution
```

#### Pipeline in `text_asset.rs`

Update the export flow:
```
1. Read m_Script raw bytes
2. If starts with 128-byte RSA header:
   a. Skip RSA header
   b. If remaining data is FlatBuffer → decode to JSON
   c. Else try AES-CBC decrypt → parse as JSON/BSON
3. If valid UTF-8 text → save as .txt
4. Fallback → save as .bytes
```

#### Test

```rust
#[test]
fn test_decode_gamedata() {
    // Parse an anon bundle, extract TextAssets
    // Skip 128-byte RSA header, check is_flatbuffer()
    // Decode and verify JSON output has expected structure
}
```

### Step 10: `src/export/audio.rs`

Read `m_Name` and audio data (from `m_AudioData` or `m_Resource`). Check magic bytes:
- `OggS` (4F 67 67 53) → save as `.ogg`
- `RIFF` (52 49 46 46) → save as `.wav`
- `ftyp` at offset 4 → save as `.m4a`
- Otherwise → save as `.bytes` (FSB5 needs FMOD, skip for now)

---

## Build & Test Order

1. Get `endian_reader.rs` working, write unit tests for it
2. Get `compression.rs` + `lz4ak.rs` working — test with `lz4_flex` on known data
3. Get `bundle.rs` parsing a real AB file header — use a small AB file from `../downloader/ArkAssets/`
4. Get `serialized_file.rs` listing objects in that AB file (just print path_id, class_id, size)
5. Get `type_tree.rs` + `object_reader.rs` reading a TextAsset object as JSON
6. Get `texture.rs` extracting a PNG from a Texture2D
7. Wire up CLI and process a full directory

Start with step 1. Once your `EndianReader` works and has tests, everything else builds on it cleanly.
