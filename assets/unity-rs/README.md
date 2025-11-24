# UnityRs

A Rust library for reading, extracting, and manipulating Unity asset files and asset bundles. This project is a Rust port of [UnityPy](https://github.com/K0lb3/UnityPy), providing high-performance access to Unity's serialized files, bundles, and web files.

## Features

- **Fast and Memory Efficient** - Native Rust performance with zero-copy deserialization where possible
- **Asset Bundle Support** - Load and extract from `.unity3d`, `.bundle`, `.assetbundle` files
- **Compression Formats** - Automatic decompression of LZ4, LZMA, Brotli, and gzip compressed assets
- **Texture Extraction** - Export textures in 60+ formats (DXT/BCn, ASTC, ETC, PVRTC, etc.) to PNG
- **Audio Extraction** - Convert AudioClip assets to WAV format using FMOD
- **Mesh Export** - Extract 3D meshes to OBJ format
- **Full Type System** - Access to 200+ Unity class types with full serialization support
- **TypeTree Support** - Parse objects without compiled type definitions
- **Cross-File References** - Automatic resolution of PPtr references across multiple files

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
unity-rs = { git = "https://github.com/Eltik/UnityRs" }
```

Or use cargo add:

```bash
cargo add --git https://github.com/Eltik/UnityRs unity-rs
```

### Platform-Specific Notes

**Audio Extraction (FMOD)**

Audio extraction requires FMOD libraries. On macOS, you'll need to set the library path:

```bash
export DYLD_LIBRARY_PATH=/path/to/UnityRs/src/resources/FMOD/Darwin
```

On Linux/Windows, ensure the FMOD libraries are in your system library path.

## Quick Start

### Loading and Inspecting Assets

```rust
use unity_rs::Environment;

fn main() -> std::io::Result<()> {
    // Create an environment
    let mut env = Environment::new();

    // Load a Unity asset file
    env.load_file(
        unity_rs::helpers::import_helper::FileSource::Path("assets.unity3d".to_string()),
        None,
        false
    );

    // List all objects in the environment
    for obj in env.objects() {
        println!("Object: {:?} (ID: {})", obj.obj_type, obj.path_id);
    }

    Ok(())
}
```

### Extracting All Assets

```rust
use unity_rs::tools::extract_assets;

fn main() -> std::io::Result<()> {
    // Extract all assets from a bundle to a directory
    let count = extract_assets(
        "assets.unity3d",      // Source file or folder
        "output/",             // Destination directory
        true                   // Use container paths for organization
    )?;

    println!("Extracted {} assets", count);
    Ok(())
}
```

### Loading from Multiple Sources

```rust
use unity_rs::Environment;
use unity_rs::helpers::import_helper::FileSource;

fn main() -> std::io::Result<()> {
    let mut env = Environment::new();

    // Load from file path
    env.load_file(FileSource::Path("bundle.unity3d".to_string()), None, false);

    // Load from bytes
    let data = std::fs::read("another.unity3d")?;
    env.load_file(FileSource::Bytes(data), Some("custom_name".to_string()), false);

    // Load entire folder
    env.load_folder("assets_directory/")?;

    // Load ZIP archive
    env.load_zip_file(FileSource::Path("assets.zip".to_string()))?;

    Ok(())
}
```

## Working with Objects

### Reading Object Data

```rust
use unity_rs::Environment;
use unity_rs::helpers::import_helper::FileSource;
use unity_rs::generated::Texture2D;

fn main() -> std::io::Result<()> {
    let mut env = Environment::new();
    env.load_file(FileSource::Path("bundle.unity3d".to_string()), None, false);

    // Iterate through all objects
    for mut obj in env.objects() {
        match obj.obj_type {
            unity_rs::ClassIDType::Texture2D => {
                // Read the object data as JSON
                if let Ok(data) = obj.read(false) {
                    // Deserialize to Texture2D struct
                    if let Ok(texture) = serde_json::from_value::<Texture2D>(data) {
                        println!("Texture: {}x{}",
                            texture.m_Width.unwrap_or(0),
                            texture.m_Height.unwrap_or(0)
                        );
                    }
                }
            },
            _ => {}
        }
    }

    Ok(())
}
```

### Using the Container

The container provides named access to objects via their asset paths:

```rust
use unity_rs::Environment;
use unity_rs::helpers::import_helper::FileSource;

fn main() -> std::io::Result<()> {
    let mut env = Environment::new();
    env.load_file(FileSource::Path("bundle.unity3d".to_string()), None, false);

    // Access objects by their container path
    let container = env.container();

    for (path, obj) in container {
        println!("Asset: {} -> {:?}", path, obj.obj_type);
    }

    Ok(())
}
```

## Exporting Specific Asset Types

### Textures

```rust
use unity_rs::export::texture_2d_converter::save_texture_as_png;
use unity_rs::generated::Texture2D;
use std::path::Path;

fn export_texture(texture: &Texture2D, output: &Path) -> unity_rs::UnityResult<()> {
    save_texture_as_png(
        texture,
        output,
        true  // Flip vertically (Unity textures are upside down)
    )
}
```

### Audio Clips

```rust
use unity_rs::export::audio_clip_converter::extract_audioclip_samples;
use unity_rs::generated::AudioClip;

fn export_audio(mut audio: AudioClip) -> unity_rs::UnityResult<()> {
    // Extract samples to WAV format
    let samples = extract_audioclip_samples(&mut audio, true)?;

    for (name, data) in samples {
        std::fs::write(format!("{}.wav", name), data)?;
    }

    Ok(())
}
```

### Meshes

```rust
use unity_rs::export::mesh_exporter::export_mesh;
use unity_rs::generated::Mesh;

fn export_mesh_obj(mesh: &Mesh) -> unity_rs::UnityResult<String> {
    export_mesh(mesh, "obj")  // Returns OBJ format string
}
```

### Sprites

```rust
use unity_rs::export::sprite_helper::save_sprite;
use unity_rs::generated::Sprite;
use unity_rs::files::SerializedFile;

fn export_sprite(sprite: &Sprite, assets_file: &SerializedFile, output: &str)
    -> unity_rs::UnityResult<()>
{
    save_sprite(sprite, assets_file, output)
}
```

## Advanced Features

### Split File Support

UnityRs automatically handles split files (`.split0`, `.split1`, etc.):

```rust
let mut env = Environment::new();

// Automatically merges all .split files
env.load_file(
    FileSource::Path("archive.ress.split0".to_string()),
    None,
    false
);
```

### Encrypted Asset Bundles

Set decryption key for encrypted bundles:

```rust
use unity_rs::set_assetbundle_decrypt_key;

// Set AES decryption key (16 bytes)
set_assetbundle_decrypt_key(vec![0x01, 0x02, /* ... */, 0x10]);

// Now load encrypted bundle
let mut env = Environment::new();
env.load_file(FileSource::Path("encrypted.unity3d".to_string()), None, false);
```

### TypeTree Parsing

For MonoBehaviour and other custom types:

```rust
use unity_rs::{set_parse_typetree, set_fallback_unity_version};

// Enable TypeTree parsing
set_parse_typetree(true);

// Set fallback Unity version for files without version info
set_fallback_unity_version((2019, 4, 0, 1));

let mut env = Environment::new();
env.load_file(FileSource::Path("bundle.unity3d".to_string()), None, false);

for mut obj in env.objects() {
    if obj.obj_type == unity_rs::ClassIDType::MonoBehaviour {
        // Read using TypeTree
        if let Ok(data) = obj.read_typetree(None, true, false) {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
    }
}
```

### Binary I/O

Low-level binary reading and writing:

```rust
use unity_rs::streams::endian_reader::{BinaryReader, MemoryReader};
use unity_rs::streams::endian::Endian;

let data = vec![0x01, 0x02, 0x03, 0x04];
let mut reader = MemoryReader::new(data, Endian::Little, 0);

let value = reader.read_i32()?;
let byte = reader.read_u8()?;
```

## Configuration

### Global Settings

```rust
use unity_rs::{set_fallback_unity_version, set_parse_typetree};

// Set Unity version for files without embedded version
set_fallback_unity_version((2020, 3, 0, 1));

// Enable/disable TypeTree parsing
set_parse_typetree(true);
```

## API Overview

### Core Types

- **`Environment`** - Main entry point for loading and managing Unity files
- **`ObjectReader`** - Represents a Unity object with methods to read its data
- **`SerializedFile`** - A Unity serialized file (asset file)
- **`BundleFile`** - A Unity asset bundle
- **`WebFile`** - A Unity web player file

### Enums

- **`ClassIDType`** - Unity object types (Texture2D, AudioClip, Mesh, etc.)
- **`TextureFormat`** - Texture compression formats
- **`AudioCompressionFormat`** - Audio compression types
- **`BuildTarget`** - Target platforms (Windows, Android, iOS, etc.)

### Export Functions

```rust
// High-level extraction
pub fn extract_assets(src: &Path, dst: &Path, use_container: bool) -> io::Result<usize>

// Texture conversion
pub fn save_texture_as_png(texture: &Texture2D, output: &Path, flip: bool) -> UnityResult<()>

// Audio extraction
pub fn extract_audioclip_samples(audio: &mut AudioClip, decode_fmod: bool)
    -> UnityResult<HashMap<String, Vec<u8>>>

// Mesh export
pub fn export_mesh(mesh: &Mesh, format: &str) -> UnityResult<String>

// Sprite export
pub fn save_sprite(sprite: &Sprite, assets_file: &SerializedFile, output: &str)
    -> UnityResult<()>
```

## Supported Formats

### Asset Files

- `.unity3d` - Asset bundles
- `.assetbundle` - Asset bundles
- `.bundle` - Asset bundles
- `.assets` - Raw serialized files
- `.resource` - Resource files
- `.ress` - Resource files (with split support)
- `.sharedAssets` - Shared asset files

### Compression

- LZ4
- LZMA
- Brotli
- Gzip

### Texture Formats (60+)

- Uncompressed: RGBA32, RGB24, ARGB32, etc.
- DXT/BC: DXT1, DXT5, BC4, BC5, BC6H, BC7
- Mobile: ETC, ETC2, PVRTC, ASTC
- And many more...

### Audio Formats

- PCM (WAV export)
- FMOD FSB5 (with FMOD library)
- Vorbis
- MP3
- And more...

## Examples

Check out the `tests/` directory for more examples:

```bash
cargo test --test test_extractor -- --nocapture
```

## Platform Support

- ✅ macOS (tested)
- ✅ Linux (should work)
- ✅ Windows (should work)

Note: Audio extraction requires platform-specific FMOD libraries.

## Comparison with UnityPy

| Feature | UnityPy (Python) | UnityRs (Rust) |
|---------|------------------|----------------|
| Performance | Good | Excellent |
| Memory Usage | Higher | Lower |
| Type Safety | Runtime | Compile-time |
| Async/Parallel | Limited | Native support |
| Dependencies | Many Python packages | Minimal Rust crates |

## Contributing

This is a port of UnityPy to Rust. Contributions are welcome! Please ensure:

1. Code follows Rust conventions
2. Tests pass: `cargo test`
3. Documentation is updated
4. Compatibility with UnityPy behavior is maintained where applicable

## License

Based on [UnityPy](https://github.com/K0lb3/UnityPy) by K0lb3.

## Acknowledgments

- **K0lb3** for the original [UnityPy](https://github.com/K0lb3/UnityPy) project
- Unity Technologies for the Unity engine and file formats

## Related Projects

- [UnityPy](https://github.com/K0lb3/UnityPy) - Original Python implementation
- [ArkUnpacker](https://github.com/isHarryh/Ark-Unpacker) - Unpacker that uses UnityPy
