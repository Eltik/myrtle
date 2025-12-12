//! FMOD integration for compressed audio decoding
//!
//! Python equivalent: AudioClipConverter.py lines 30-82, 114-167

use crate::classes::generated::AudioClip;
use crate::{UnityError, UnityResult};
use lazy_static::lazy_static;
use libfmod::{CreateSoundexInfo, Init, Mode, Sound, SoundFormat, System, TimeUnit};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global FMOD system instances cache
// Python: SYSTEM_INSTANCES = {}  # (channels, flags) -> (pyfmodex_system_instance, lock)
lazy_static! {
    static ref SYSTEM_INSTANCES: Mutex<HashMap<(i32, Init), Arc<Mutex<System>>>> =
        Mutex::new(HashMap::new());

    // Global lock to serialize FMOD System creation/destruction
    // FMOD has a hard limit on simultaneous System instances (around 8-16)
    // This mutex ensures only one FMOD operation happens at a time
    static ref FMOD_GLOBAL_LOCK: Mutex<()> = Mutex::new(());
}

/// Create a new FMOD system instance (no caching to avoid resource limits)
///
/// Python: get_pyfmodex_system_instance(channels, flags)
/// Python lines: 118-129
fn get_fmod_system(channels: i32, flags: Init) -> UnityResult<System> {
    // Create new FMOD system without caching to avoid hitting FMOD's system instance limit
    // Python lines 125-126: system = pyfmodex.System(); system.init(channels, flags, None)
    let system = System::create()
        .map_err(|e| UnityError::Other(format!("Failed to create FMOD System: {:?}", e)))?;

    system
        .init(channels, flags, None)
        .map_err(|e| UnityError::Other(format!("Failed to initialize FMOD System: {:?}", e)))?;

    Ok(system)
}

/// Decode compressed audio using FMOD
///
/// Python: dump_samples(clip, audio_data, convert_pcm_float=True)
/// Python lines: 132-167
pub fn dump_samples(
    clip: &AudioClip,
    audio_data: &[u8],
    convert_pcm_float: bool,
) -> UnityResult<HashMap<String, Vec<u8>>> {
    // Acquire global lock to prevent concurrent FMOD System creation
    // FMOD has a limit on simultaneous System instances and will fail with
    // "Not enough memory or resources" if too many are created at once
    let _fmod_guard = FMOD_GLOBAL_LOCK
        .lock()
        .map_err(|e| UnityError::Other(format!("Failed to acquire FMOD lock: {:?}", e)))?;

    // Python lines 136-139: Check if pyfmodex is available
    // In Rust, we try to get the system and fail if FMOD isn't available

    let channels = clip.m_Channels.unwrap_or(2);
    let frequency = clip.m_Frequency.unwrap_or(44100);

    // Python lines 141-143: Create FMOD system (no caching to avoid resource limits)
    let system = get_fmod_system(channels, Init::NORMAL)?;

    // Python lines 145-153: Create sound from memory
    let exinfo = CreateSoundexInfo {
        length: audio_data.len() as u32,
        numchannels: channels,
        defaultfrequency: frequency,
        ..Default::default()
    };

    let sound = system
        .create_sound_from(audio_data, Mode::OPENMEMORY, exinfo)
        .map_err(|e| UnityError::Other(format!("Failed to create sound from memory: {:?}", e)))?;

    // Python lines 155-167: Iterate over subsounds
    let num_subsounds = sound
        .get_num_sub_sounds()
        .map_err(|e| UnityError::Other(format!("Failed to get num_sub_sounds: {:?}", e)))?;

    let mut samples = HashMap::new();

    for i in 0..num_subsounds {
        // Python lines 158-161: Generate filename
        let name = clip.m_Name.as_deref().unwrap_or("audio");
        let filename = if i > 0 {
            format!("{}-{}.wav", name, i)
        } else {
            format!("{}.wav", name)
        };

        // Python line 162: Get subsound
        let subsound = sound
            .get_sub_sound(i)
            .map_err(|e| UnityError::Other(format!("Failed to get subsound {}: {:?}", i, e)))?;

        // Python line 163: Convert subsound to WAV
        let wav_data = subsound_to_wav(&subsound, convert_pcm_float)?;
        samples.insert(filename, wav_data);

        // Python line 164: Release subsound explicitly
        let _ = subsound.release();
    }

    // Python line 166: Release sound explicitly before system
    let _ = sound.release();

    // Explicitly release FMOD system to avoid resource exhaustion
    let _ = system.release();

    Ok(samples)
}

/// Convert FMOD subsound to WAV format
///
/// Python: subsound_to_wav(subsound, convert_pcm_float=True)
/// Python lines: 170-238
fn subsound_to_wav(subsound: &Sound, convert_pcm_float: bool) -> UnityResult<Vec<u8>> {
    use crate::streams::endian_writer::{BinaryWriter, EndianBinaryWriter};
    use crate::Endian;

    // Python lines 171-176: Get sound settings
    let (_sound_type, sound_format, channels, bits) = subsound
        .get_format()
        .map_err(|e| UnityError::Other(format!("Failed to get sound format: {:?}", e)))?;

    let sample_rate = subsound
        .get_defaults()
        .map_err(|e| UnityError::Other(format!("Failed to get defaults: {:?}", e)))?
        .0 as i32; // frequency

    let sound_data_length = subsound
        .get_length(TimeUnit::PCMBYTES)
        .map_err(|e| UnityError::Other(format!("Failed to get length: {:?}", e)))?;

    // Python lines 178-197: Determine audio format and convert settings
    let (audio_format, wav_data_length, final_bits, should_convert) = match sound_format {
        // Python lines 178-187: PCM integer formats
        SoundFormat::Pcm8 | SoundFormat::Pcm16 | SoundFormat::Pcm24 | SoundFormat::Pcm32 => {
            (1, sound_data_length, bits, false)
        }
        // Python lines 188-196: PCM float format
        SoundFormat::PcmFloat => {
            if convert_pcm_float {
                (1, sound_data_length / 2, 16, true)
            } else {
                (3, sound_data_length, bits, false)
            }
        }
        _ => {
            return Err(UnityError::Other(format!(
                "Sound format {:?} is not supported",
                sound_format
            )));
        }
    };

    // Python line 200: Create writer with little-endian
    let mut w = EndianBinaryWriter::new(Endian::Little);

    // Python lines 202-207: RIFF header
    w.write(b"RIFF")?;
    w.write_u32(wav_data_length + 36)?;
    w.write(b"WAVE")?;

    // Python lines 209-217: fmt chunk
    w.write(b"fmt ")?;
    w.write_u32(16)?;
    w.write_u16(audio_format as u16)?;
    w.write_u16(channels as u16)?;
    w.write_u32(sample_rate as u32)?;
    w.write_u32((sample_rate * channels * final_bits / 8) as u32)?;
    w.write_u16((channels * final_bits / 8) as u16)?;
    w.write_u16(final_bits as u16)?;

    // Python lines 219-221: data chunk header
    w.write(b"data")?;
    w.write_u32(wav_data_length)?;

    // Python lines 222-237: Lock and read PCM data
    let (ptr1, ptr2, len1, len2) = subsound
        .lock(0, sound_data_length)
        .map_err(|e| UnityError::Other(format!("Failed to lock sound: {:?}", e)))?;

    // Python line 224: for ptr, sound_data_length in lock:
    // Process all lock regions (typically 1, but can be 2 if wrapping)
    let lock_regions = vec![(ptr1, len1), (ptr2, len2)];

    for (ptr, len) in lock_regions {
        if len == 0 {
            continue; // Skip empty regions
        }

        // Python line 225: ptr_data = ctypes.string_at(ptr, sound_data_length.value)
        let pcm_data = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

        // Python lines 226-234: Convert PCM float to int16 if needed
        if should_convert {
            // Convert f32 to i16: multiply by 32768 (1 << 15)
            for chunk in pcm_data.chunks_exact(4) {
                let f32_val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let i16_val = (f32_val * 32768.0).clamp(-32768.0, 32767.0) as i16;
                w.write_i16(i16_val)?;
            }
        } else {
            // Python line 236: Write samples directly
            w.write(pcm_data)?;
        }
    }

    // Unlock the sound
    subsound
        .unlock(ptr1, ptr2, len1, len2)
        .map_err(|e| UnityError::Other(format!("Failed to unlock sound: {:?}", e)))?;

    // Python line 239: Return bytes
    Ok(w.to_bytes())
}

/// Clears the FMOD system instance cache to release memory
/// Note: Cache is no longer used to avoid FMOD resource limits
pub fn clear_fmod_cache() {
    // No-op since we no longer cache FMOD systems
    // Systems are created and released per audio clip to avoid resource limits
}
