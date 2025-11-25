//! AudioClip audio extraction
//!
//! Python equivalent: UnityPy/export/AudioClipConverter.py (239 lines)

pub mod fmod;

use crate::classes::generated::AudioClip;
use crate::helpers::resource_reader::get_resource_data;
use crate::{UnityError, UnityResult};
use std::collections::HashMap;

/// Extract all audio samples from an AudioClip
///
/// Python: extract_audioclip_samples(audio, convert_pcm_float=True)
pub fn extract_audioclip_samples(
    audio: &mut AudioClip,
    convert_pcm_float: bool,
) -> UnityResult<HashMap<String, Vec<u8>>> {
    // Get audio data from either m_AudioData or m_Resource
    let audio_data = if let Some(ref data) = audio.m_AudioData {
        data.clone()
    } else if let Some(ref resource) = audio.m_Resource {
        // Get the object_reader and downcast to access internal fields
        let obj_reader = audio
            .object_reader
            .as_ref()
            .ok_or_else(|| UnityError::Other("No object_reader".to_string()))?;

        // Downcast to concrete ObjectReader<()> to access assets_file field
        let concrete_reader = obj_reader
            .as_any()
            .downcast_ref::<crate::files::ObjectReader<()>>()
            .ok_or_else(|| UnityError::Other("Failed to downcast ObjectReader".to_string()))?;

        // Get the Weak<RefCell<SerializedFile>> from the concrete reader
        let weak_assets = concrete_reader
            .assets_file
            .as_ref()
            .ok_or_else(|| UnityError::Other("No assets_file".to_string()))?;

        // Upgrade Weak to Rc
        let rc_assets = weak_assets
            .upgrade()
            .ok_or_else(|| UnityError::Other("SerializedFile was dropped".to_string()))?;

        // Borrow mutably through RefCell
        let mut assets_file_mut = rc_assets.borrow_mut();

        // Check if offset and size are present
        let offset = resource.m_Offset.unwrap_or(0);
        let size = resource.m_Size.unwrap_or(0);

        if offset == 0 && size == 0 {
            return Err(UnityError::Other(
                "Resource has no offset/size data".to_string(),
            ));
        }

        // Extract m_Source as a string if possible
        let source_path = resource
            .m_Source
            .as_deref()
            .ok_or_else(|| UnityError::Other("Resource m_Source is missing".to_string()))?;

        get_resource_data(
            source_path,
            &mut *assets_file_mut,
            offset as usize,
            size as usize,
        )?
    } else {
        return Err(UnityError::Other("No audio data available".to_string()));
    };

    // Check magic bytes to identify format
    if audio_data.len() >= 8 {
        let magic = &audio_data[0..8];

        // Ogg Vorbis - already decoded
        if &magic[0..4] == b"OggS" {
            let mut samples = HashMap::new();
            let name = audio.m_Name.as_deref().unwrap_or("audio");
            samples.insert(format!("{}.ogg", name), audio_data);
            return Ok(samples);
        }

        // WAV - already decoded
        if &magic[0..4] == b"RIFF" {
            let mut samples = HashMap::new();
            let name = audio.m_Name.as_deref().unwrap_or("audio");
            samples.insert(format!("{}.wav", name), audio_data);
            return Ok(samples);
        }

        // M4A - already decoded
        if &magic[4..8] == b"ftyp" {
            let mut samples = HashMap::new();
            let name = audio.m_Name.as_deref().unwrap_or("audio");
            samples.insert(format!("{}.m4a", name), audio_data);
            return Ok(samples);
        }
    }

    // Python line 111: If not a known format, need to decode (FSB5, etc.)
    dump_samples(audio, &audio_data, convert_pcm_float)
}

/// Decode compressed audio formats using FMOD
///
/// Python: dump_samples(clip, audio_data, convert_pcm_float=True)
/// Python lines: 132-167
fn dump_samples(
    clip: &AudioClip,
    audio_data: &[u8],
    convert_pcm_float: bool,
) -> UnityResult<HashMap<String, Vec<u8>>> {
    // Delegate to FMOD module for full implementation
    fmod::dump_samples(clip, audio_data, convert_pcm_float)
}
