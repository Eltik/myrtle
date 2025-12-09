//! SpritePacker extraction for Arknights character portraits
//!
//! Arknights uses a custom SpritePacker MonoBehaviour to store sprite atlas data.
//! This module handles parsing and extracting individual sprites from the packed textures.

use image::{ImageBuffer, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents a single sprite's location within an atlas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasSprite {
    pub name: String,
    pub guid: String,
    pub atlas: i32,
    pub rect: AtlasRect,
    pub rotate: u8,
}

/// Rectangle coordinates within an atlas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Information about an atlas texture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasInfo {
    pub index: i32,
    pub texture: PtrRef,
    pub alpha: PtrRef,
    pub size: i32,
}

/// PPtr reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtrRef {
    #[serde(rename = "m_FileID")]
    pub file_id: i32,
    #[serde(rename = "m_PathID")]
    pub path_id: i64,
}

/// Parsed SpritePacker data from a MonoBehaviour
#[derive(Debug, Clone)]
pub struct SpritePackerData {
    pub name: String,
    pub sprites: Vec<AtlasSprite>,
    pub atlas: AtlasInfo,
    pub index: i32,
}

impl SpritePackerData {
    /// Parse SpritePacker data from a MonoBehaviour JSON
    pub fn from_json(json: &Value) -> Result<Self, String> {
        let name = json
            .get("m_Name")
            .and_then(|v| v.as_str())
            .ok_or("Missing m_Name")?
            .to_string();

        let sprites_value = json.get("_sprites").ok_or("Missing _sprites field")?;
        let sprites: Vec<AtlasSprite> = serde_json::from_value(sprites_value.clone())
            .map_err(|e| format!("Failed to parse _sprites: {}", e))?;

        let atlas_value = json.get("_atlas").ok_or("Missing _atlas field")?;
        let atlas: AtlasInfo = serde_json::from_value(atlas_value.clone())
            .map_err(|e| format!("Failed to parse _atlas: {}", e))?;

        let index = json
            .get("_index")
            .and_then(|v| v.as_i64())
            .ok_or("Missing _index")? as i32;

        Ok(SpritePackerData {
            name,
            sprites,
            atlas,
            index,
        })
    }
}

/// Extracts individual sprites from a texture atlas
pub struct SpriteExtractor {
    /// The RGB texture (1024x1024 typically)
    texture: RgbaImage,
    /// The alpha texture (optional)
    alpha: Option<RgbaImage>,
    /// Atlas size
    size: u32,
}

impl SpriteExtractor {
    /// Create a new extractor with the given textures
    pub fn new(texture: RgbaImage, alpha: Option<RgbaImage>) -> Self {
        let size = texture.width();
        Self {
            texture,
            alpha,
            size,
        }
    }

    /// Extract a single sprite from the atlas
    ///
    /// # Arguments
    /// * `sprite` - The sprite definition with rect and rotation info
    ///
    /// # Returns
    /// The extracted sprite as an RgbaImage
    pub fn extract_sprite(&self, sprite: &AtlasSprite) -> Result<RgbaImage, String> {
        let rect = &sprite.rect;

        // Validate bounds
        if rect.x < 0 || rect.y < 0 || rect.w <= 0 || rect.h <= 0 {
            return Err(format!(
                "Invalid rect for sprite {}: {:?}",
                sprite.name, rect
            ));
        }

        let x = rect.x as u32;
        let y = rect.y as u32;
        let w = rect.w as u32;
        let h = rect.h as u32;

        // Check bounds (y is from bottom in Unity)
        let unity_y = self.size.saturating_sub(y + h);

        if x + w > self.size || unity_y + h > self.size {
            return Err(format!(
                "Sprite {} extends beyond texture bounds: x={}, y={}, w={}, h={}, size={}",
                sprite.name, x, unity_y, w, h, self.size
            ));
        }

        // Create the output image
        let (out_w, out_h) = if sprite.rotate == 1 {
            (h, w) // Rotated: swap dimensions
        } else {
            (w, h)
        };

        let mut output: RgbaImage = ImageBuffer::new(out_w, out_h);

        // Extract pixels
        for dy in 0..h {
            for dx in 0..w {
                let src_x = x + dx;
                let src_y = unity_y + dy;

                // Get RGB pixel
                let pixel = self.texture.get_pixel(src_x, src_y);
                let mut rgba = *pixel;

                // Apply alpha from separate alpha texture if available
                if let Some(ref alpha_tex) = self.alpha {
                    let alpha_pixel = alpha_tex.get_pixel(src_x, src_y);
                    // Use the red channel as alpha (grayscale alpha map)
                    rgba[3] = alpha_pixel[0];
                }

                // Handle rotation
                let (dest_x, dest_y) = if sprite.rotate == 1 {
                    // Rotated 90 degrees clockwise
                    (h - 1 - dy, dx)
                } else {
                    (dx, dy)
                };

                output.put_pixel(dest_x, dest_y, rgba);
            }
        }

        Ok(output)
    }

    /// Extract all sprites from this atlas
    pub fn extract_all(
        &self,
        sprites: &[AtlasSprite],
    ) -> HashMap<String, Result<RgbaImage, String>> {
        sprites
            .iter()
            .map(|sprite| (sprite.name.clone(), self.extract_sprite(sprite)))
            .collect()
    }
}

/// Check if a MonoBehaviour JSON contains SpritePacker data
pub fn is_sprite_packer(json: &Value) -> bool {
    json.get("_sprites").is_some() && json.get("_atlas").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sprite_packer() {
        let json = serde_json::json!({
            "m_Name": "portraits#5",
            "_sprites": [
                {
                    "name": "char_137_brownb_1",
                    "guid": "abc123",
                    "atlas": 0,
                    "rect": { "x": 2, "y": 0, "w": 180, "h": 360 },
                    "rotate": 0
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

        let data = SpritePackerData::from_json(&json).unwrap();
        assert_eq!(data.name, "portraits#5");
        assert_eq!(data.sprites.len(), 1);
        assert_eq!(data.sprites[0].name, "char_137_brownb_1");
        assert_eq!(data.sprites[0].rect.w, 180);
        assert_eq!(data.atlas.size, 1024);
    }
}
