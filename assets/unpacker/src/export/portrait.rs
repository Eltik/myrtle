use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use super::texture::DecodedTexture;

// ─── Data Structures ─────────────────────────────────────────────────────────

pub struct SpriteRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct AtlasSprite {
    pub name: String,
    pub rect: SpriteRect,
    pub rotate: u8,
}

pub struct SpritePackerData {
    pub sprites: Vec<AtlasSprite>,
    pub texture_pid: i64,
    pub alpha_pid: i64,
    pub atlas_size: u32,
}

// ─── Detection & Parsing ─────────────────────────────────────────────────────

/// Check if a bundle path is a portrait atlas bundle.
pub fn detect_portrait_bundle(bundle_subdir: &Path, input_dir: &Path) -> bool {
    let full_path = input_dir.join(bundle_subdir);
    let path_lower = full_path.to_string_lossy().to_lowercase();
    path_lower.contains("charportraits") || path_lower.starts_with("charportraits")
}

/// Check if a MonoBehaviour JSON is a SpritePacker (has `_sprites` + `_atlas`).
pub fn is_sprite_packer(val: &Value) -> bool {
    val.get("_sprites").is_some_and(|v| v.is_array()) && val.get("_atlas").is_some()
}

/// Parse a SpritePacker MonoBehaviour into structured data.
pub fn parse_sprite_packer(val: &Value) -> Option<SpritePackerData> {
    if !is_sprite_packer(val) {
        return None;
    }

    let atlas = val.get("_atlas")?;
    let texture_pid = atlas.get("texture")?.get("m_PathID")?.as_i64()?;
    let alpha_pid = atlas
        .get("alpha")
        .and_then(|a| a.get("m_PathID"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let atlas_size = atlas.get("size")?.as_u64()? as u32;

    let sprites_arr = val.get("_sprites")?.as_array()?;
    let mut sprites = Vec::with_capacity(sprites_arr.len());

    for s in sprites_arr {
        let name = s.get("name")?.as_str()?.to_string();
        let rect_val = s.get("rect")?;
        let rect = SpriteRect {
            x: rect_val.get("x")?.as_i64()? as i32,
            y: rect_val.get("y")?.as_i64()? as i32,
            w: rect_val.get("w")?.as_i64()? as i32,
            h: rect_val.get("h")?.as_i64()? as i32,
        };
        let rotate = s.get("rotate").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        sprites.push(AtlasSprite { name, rect, rotate });
    }

    Some(SpritePackerData {
        sprites,
        texture_pid,
        alpha_pid,
        atlas_size,
    })
}

// ─── Extraction ──────────────────────────────────────────────────────────────

/// Extract individual portraits from SpritePacker atlases.
///
/// Crops each sprite from the RGB atlas, merges the separate alpha atlas
/// (looked up by path_id then by `{name}a` convention), and saves as RGBA PNG.
pub fn extract_portraits(
    packers: &[SpritePackerData],
    textures_by_pid: &HashMap<i64, DecodedTexture>,
    output_dir: &Path,
) -> usize {
    let mut count = 0;

    // Build a name → pid lookup for alpha fallback
    let name_to_pid: HashMap<&str, i64> = textures_by_pid
        .iter()
        .map(|(pid, tex)| (tex.name.as_str(), *pid))
        .collect();

    for packer in packers {
        let Some(rgb_tex) = textures_by_pid.get(&packer.texture_pid) else {
            continue;
        };

        // Look up alpha: by path_id first, then by "{rgb_name}a" name convention
        let alpha_tex = if packer.alpha_pid != 0 {
            textures_by_pid.get(&packer.alpha_pid)
        } else {
            None
        }
        .or_else(|| {
            let alpha_name = format!("{}a", rgb_tex.name);
            name_to_pid
                .get(alpha_name.as_str())
                .and_then(|pid| textures_by_pid.get(pid))
        });

        for sprite in &packer.sprites {
            if let Some(data) = crop_sprite(rgb_tex, alpha_tex, sprite, packer.atlas_size) {
                let (out_w, out_h) = if sprite.rotate == 1 {
                    (sprite.rect.h as u32, sprite.rect.w as u32)
                } else {
                    (sprite.rect.w as u32, sprite.rect.h as u32)
                };

                let path = output_dir.join(format!("{}.png", sprite.name));
                match image::save_buffer(&path, &data, out_w, out_h, image::ColorType::Rgba8) {
                    Ok(()) => count += 1,
                    Err(e) => eprintln!("  error saving portrait {}: {e}", sprite.name),
                }
            }
        }
    }

    count
}

/// Crop a single sprite from the atlas, merging RGB + alpha per-pixel.
fn crop_sprite(
    rgb: &DecodedTexture,
    alpha: Option<&DecodedTexture>,
    sprite: &AtlasSprite,
    atlas_size: u32,
) -> Option<Vec<u8>> {
    let r = &sprite.rect;
    // Convert Unity bottom-left origin to top-left origin
    let unity_y = atlas_size as i32 - (r.y + r.h);

    if unity_y < 0 || r.x < 0 || r.w <= 0 || r.h <= 0 {
        return None;
    }

    let (out_w, out_h) = if sprite.rotate == 1 {
        (r.h as u32, r.w as u32) // Swap for 90° CW rotation
    } else {
        (r.w as u32, r.h as u32)
    };

    let mut output = vec![0u8; (out_w * out_h * 4) as usize];
    let atlas_w = rgb.width;

    for dy in 0..r.h {
        for dx in 0..r.w {
            let src_x = (r.x + dx) as u32;
            let src_y = (unity_y + dy) as u32;

            if src_x >= rgb.width || src_y >= rgb.height {
                continue;
            }

            let src_idx = ((src_y * atlas_w + src_x) * 4) as usize;

            // Determine destination pixel based on rotation
            let (dest_x, dest_y) = if sprite.rotate == 1 {
                // 90° CW: (dx, dy) in source → (h-1-dy, dx) in output
                ((r.h - 1 - dy) as u32, dx as u32)
            } else {
                (dx as u32, dy as u32)
            };

            let dest_idx = ((dest_y * out_w + dest_x) * 4) as usize;

            // Read RGB from base atlas
            let cr = rgb.rgba.get(src_idx).copied().unwrap_or(0);
            let cg = rgb.rgba.get(src_idx + 1).copied().unwrap_or(0);
            let cb = rgb.rgba.get(src_idx + 2).copied().unwrap_or(0);

            // Read alpha from separate alpha atlas (R channel)
            let ca = alpha
                .and_then(|a| a.rgba.get(src_idx))
                .copied()
                .unwrap_or(255);

            // Color bleeding fix
            if ca == 0 {
                output[dest_idx] = 0;
                output[dest_idx + 1] = 0;
                output[dest_idx + 2] = 0;
                output[dest_idx + 3] = 0;
            } else {
                output[dest_idx] = cr;
                output[dest_idx + 1] = cg;
                output[dest_idx + 2] = cb;
                output[dest_idx + 3] = ca;
            }
        }
    }

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_synthetic_sprite_packer() {
        let json = serde_json::json!({
            "m_Name": "portraits#5",
            "_sprites": [
                {
                    "name": "char_002_amiya",
                    "guid": "abc123",
                    "atlas": 0,
                    "rect": { "x": 2, "y": 0, "w": 180, "h": 360 },
                    "rotate": 0
                },
                {
                    "name": "char_003_kalts",
                    "guid": "def456",
                    "atlas": 0,
                    "rect": { "x": 0, "y": 724, "w": 360, "h": 180 },
                    "rotate": 1
                }
            ],
            "_atlas": {
                "index": 0,
                "texture": { "m_FileID": 0, "m_PathID": 123 },
                "alpha": { "m_FileID": 0, "m_PathID": 456 },
                "size": 1024
            },
            "_index": 5
        });

        let packer = parse_sprite_packer(&json).expect("should parse");
        assert_eq!(packer.sprites.len(), 2);
        assert_eq!(packer.sprites[0].name, "char_002_amiya");
        assert_eq!(packer.sprites[0].rect.w, 180);
        assert_eq!(packer.sprites[0].rect.h, 360);
        assert_eq!(packer.sprites[0].rotate, 0);
        assert_eq!(packer.sprites[1].name, "char_003_kalts");
        assert_eq!(packer.sprites[1].rotate, 1);
        assert_eq!(packer.texture_pid, 123);
        assert_eq!(packer.alpha_pid, 456);
        assert_eq!(packer.atlas_size, 1024);
    }

    #[test]
    fn not_sprite_packer() {
        let json = serde_json::json!({"m_Name": "not_a_packer", "foo": "bar"});
        assert!(!is_sprite_packer(&json));
        assert!(parse_sprite_packer(&json).is_none());
    }
}
