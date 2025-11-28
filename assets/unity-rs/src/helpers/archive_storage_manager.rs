//! Archive Storage Manager for decrypting Unity China AssetBundles
//!
//! Based on: <https://github.com/Razmoth/PGRStudio/blob/master/AssetStudio/PGR/PGR.cs>
//!
//! This module provides decryption capabilities for encrypted Unity AssetBundle files,
//! specifically those using the Unity China encryption scheme.

use crate::streams::endian_reader::BinaryReader;
use std::fs;
use std::io;

use aes::Aes128;
use cipher::generic_array::GenericArray;
use cipher::{BlockEncrypt, KeyInit};
use once_cell::sync::Lazy;
use regex::bytes::Regex;
use std::sync::Mutex;

/// Unity3D China signature used for validation
const UNITY3D_SIGNATURE: &[u8; 16] = b"#$unity3dchina!@";

/// Global decryption key for AssetBundles
///
/// This is protected by a Mutex for thread-safe access.
/// Use `set_assetbundle_decrypt_key()` to set the key.
static DECRYPT_KEY: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

/// Sets the decryption key for AssetBundles
///
/// The key must be exactly 16 bytes long.
///
/// # Arguments
///
/// * `key` - The decryption key as either bytes or a string
///
/// # Errors
///
/// Returns an error if the key length is not 16 bytes.
///
/// # Examples
///
/// ```
/// use unity_rs::helpers::archive_storage_manager::set_assetbundle_decrypt_key;
///
/// // Using a byte array
/// set_assetbundle_decrypt_key(b"my16bytekey12345").unwrap();
///
/// // Using a string
/// set_assetbundle_decrypt_key("my16bytekey12345").unwrap();
/// ```
pub fn set_assetbundle_decrypt_key<K: AsRef<[u8]>>(key: K) -> Result<(), String> {
    let key_bytes = key.as_ref();

    if key_bytes.len() != 16 {
        return Err(format!(
            "AssetBundle Key length is wrong. It should be 16 bytes and now is {} bytes.",
            key_bytes.len()
        ));
    }

    let mut decrypt_key = DECRYPT_KEY.lock().unwrap();
    *decrypt_key = Some(key_bytes.to_vec());

    Ok(())
}

/// Reads a data/key vector pair from the reader
///
/// Reads 16 bytes of data, 16 bytes of key, then skips 1 byte.
///
/// # Arguments
///
/// * `reader` - A binary reader implementing the BinaryReader trait
///
/// # Returns
///
/// A tuple of (data, key) where each is 16 bytes
///
/// # Errors
///
/// Returns an error if reading fails
fn read_vector<R: BinaryReader>(reader: &mut R) -> Result<(Vec<u8>, Vec<u8>), io::Error> {
    let data = reader.read_bytes(0x10)?;
    let key = reader.read_bytes(0x10)?;

    // Skip 1 byte by advancing position
    let current_pos = reader.position();
    reader.set_position(current_pos + 1);

    Ok((data, key))
}

/// Decrypts a key using AES ECB encryption and XOR
///
/// # Arguments
///
/// * `key` - The key to encrypt (16 bytes)
/// * `data` - The data to XOR with (16 bytes)
/// * `keybytes` - The AES encryption key (16 bytes)
///
/// # Returns
///
/// The XORed result of encrypted key and data
pub fn decrypt_key(key: &[u8], data: &[u8], keybytes: &[u8]) -> Vec<u8> {
    let cipher = Aes128::new(GenericArray::from_slice(keybytes));

    let mut encrypted_key = GenericArray::clone_from_slice(key);
    cipher.encrypt_block(&mut encrypted_key);

    encrypted_key
        .iter()
        .zip(data.iter())
        .map(|(x, y)| x ^ y)
        .collect()
}

/// AES-128 ECB encryption
///
/// # Arguments
///
/// * `keybytes` - The encryption key (16 bytes)
/// * `data` - The data to encrypt (16 bytes)
///
/// # Returns
///
/// Encrypted data (16 bytes)
pub fn aes_encrypt(keybytes: &[u8], data: &[u8]) -> Vec<u8> {
    let key = GenericArray::from_slice(keybytes);
    let cipher = Aes128::new(key);

    let mut block = GenericArray::clone_from_slice(data);
    cipher.encrypt_block(&mut block);

    block.to_vec()
}

/// Brute-forces the decryption key by searching through a file
///
/// # Arguments
///
/// * `fp` - File path to search (typically global-metadata.dat or memory dump)
/// * `key_sig` - The key signature to decrypt
/// * `data_sig` - The data signature to decrypt
/// * `pattern` - Optional regex pattern (defaults to finding 16-byte sequences)
/// * `verbose` - Whether to print progress
///
/// # Returns
///
/// The found key, or None if no valid key was found
///
/// # Examples
///
/// ```no_run
/// use unity_rs::helpers::archive_storage_manager::brute_force_key;
///
/// let key_sig = b"1234567890123456";
/// let data_sig = b"abcdefghijklmnop";
///
/// if let Some(key) = brute_force_key(
///     "global-metadata.dat",
///     key_sig,
///     data_sig,
///     None,
///     true
/// ).unwrap() {
///     println!("Found key: {:?}", key);
/// }
/// ```
pub fn brute_force_key(
    fp: &str,
    key_sig: &[u8],
    data_sig: &[u8],
    pattern: Option<&str>,
    verbose: bool,
) -> Result<Option<Vec<u8>>, io::Error> {
    let data = fs::read(fp)?;

    let pattern_str = pattern.unwrap_or(r"(?=(\w{16}))");
    let re = Regex::new(pattern_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let matches: Vec<&[u8]> = re
        .captures_iter(&data)
        .filter_map(|cap| cap.get(1).map(|m| m.as_bytes()))
        .collect();

    for (i, key) in matches.iter().enumerate() {
        if verbose {
            println!("Trying {}/{} - {:?}", i + 1, matches.len(), key);
        }

        let signature = decrypt_key(key_sig, data_sig, key);
        if signature.as_slice() == UNITY3D_SIGNATURE {
            if verbose {
                println!("Found key: {:?}", key);
            }

            return Ok(Some(key.to_vec()));
        }
    }

    Ok(None)
}

/// Decryptor for Unity AssetBundle archive storage
///
/// Handles decryption of encrypted Unity AssetBundle data blocks
/// using a custom encryption scheme.
#[derive(Debug)]
pub struct ArchiveStorageDecryptor {
    #[allow(dead_code)] // Reserved field from Unity format
    unknown_1: u32,
    index: [u8; 16],
    substitute: [u8; 16],
}

impl ArchiveStorageDecryptor {
    /// Creates a new ArchiveStorageDecryptor from a binary reader
    ///
    /// # Arguments
    ///
    /// * `reader` - A binary reader positioned at the archive storage header
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No decryption key has been set
    /// - The signature validation fails
    /// - Reading from the reader fails
    pub fn new<R: BinaryReader>(reader: &mut R) -> Result<Self, String> {
        let unknown_1 = reader
            .read_u32()
            .map_err(|e| format!("Failed to read unknown_1: {}", e))?;

        let (data, key) =
            read_vector(reader).map_err(|e| format!("Failed to read data vector: {}", e))?;
        let (data_sig, key_sig) =
            read_vector(reader).map_err(|e| format!("Failed to read signature vector: {}", e))?;

        let key_guard = DECRYPT_KEY.lock().unwrap();
        let key_bytes = key_guard.as_ref().ok_or_else(|| {
            format!(
                "The BundleFile is encrypted, but no key was provided!\n\
                You can set the key via set_assetbundle_decrypt_key(key).\n\
                To try brute-forcing the key, use brute_force_key(fp, key_sig, data_sig)\n\
                with key_sig = {:?}, data_sig = {:?},\n\
                and fp being the path to global-metadata.dat or a memory dump.",
                key_sig, data_sig
            )
        })?;

        let signature = decrypt_key(&key_sig, &data_sig, key_bytes);
        if signature.as_slice() != UNITY3D_SIGNATURE {
            return Err(format!(
                "Invalid signature {:?} != {:?}",
                signature, UNITY3D_SIGNATURE
            ));
        }

        let decrypted_data = decrypt_key(&key, &data, key_bytes);

        let mut nibbles = Vec::with_capacity(decrypted_data.len() * 2);
        for byte in decrypted_data {
            nibbles.push(byte >> 4);
            nibbles.push(byte & 0xF);
        }

        let mut index = [0u8; 16];
        index.copy_from_slice(&nibbles[0..16]);

        let mut substitute = [0u8; 16];
        let mut idx = 0;
        for j in 0..4 {
            for i in 0..4 {
                substitute[idx] = nibbles[0x10 + i * 4 + j];
                idx += 1;
            }
        }

        Ok(ArchiveStorageDecryptor {
            unknown_1,
            index,
            substitute,
        })
    }

    /// Decrypts a block of data
    ///
    /// # Arguments
    ///
    /// * `data` - The data block to decrypt
    /// * `index` - The starting block index
    ///
    /// # Returns
    ///
    /// The decrypted data
    pub fn decrypt_block(&self, data: &[u8], mut index: usize) -> Vec<u8> {
        let mut offset = 0;
        let size = data.len();

        let mut decrypted_data = data.to_vec();

        while offset < decrypted_data.len() {
            let remaining = size - offset;
            let bytes_processed = self.decrypt(&mut decrypted_data[offset..], index, remaining);
            offset += bytes_processed;
            index += 1;
        }

        decrypted_data
    }

    /// Decrypts a single byte in-place
    ///
    /// # Arguments
    ///
    /// * `data` - Mutable slice containing the byte to decrypt
    /// * `offset` - Offset of the byte within the slice
    /// * `index` - Current decryption index
    ///
    /// # Returns
    ///
    /// Tuple of (decrypted_byte, new_offset, new_index)
    fn decrypt_byte(&self, data: &mut [u8], offset: usize, index: usize) -> (u8, usize, usize) {
        let b = (self.substitute[((index >> 2) & 3) + 4] as usize
            + self.substitute[index & 3] as usize
            + self.substitute[((index >> 4) & 3) + 8] as usize
            + self.substitute[((index % 256) >> 6) + 12] as usize) as u8;

        let original_byte = data[offset];
        // Python lines 118-121: calculate in larger type then % 256
        let low = (self.index[(original_byte & 0xF) as usize].wrapping_sub(b) as u32) & 0xF;
        let high =
            (0x10 as u32) * (self.index[(original_byte >> 4) as usize].wrapping_sub(b) as u32);
        data[offset] = ((low | high) % 256) as u8;

        let decrypted = data[offset];

        (decrypted, offset + 1, index + 1)
    }

    /// Decrypts a chunk of data
    ///
    /// # Arguments
    ///
    /// * `data` - Mutable slice to decrypt
    /// * `index` - Current decryption index
    /// * `remaining` - Number of remaining bytes in the full data block
    ///
    /// # Returns
    ///
    /// The number of bytes processed (offset advancement)
    fn decrypt(&self, data: &mut [u8], mut index: usize, remaining: usize) -> usize {
        let mut offset = 0;

        let (cur_byte, new_offset, new_index) = self.decrypt_byte(data, offset, index);
        offset = new_offset;
        index = new_index;

        let mut byte_high = (cur_byte >> 4) as usize;
        let byte_low = (cur_byte & 0xF) as usize;

        if byte_high == 0xF {
            let mut b = 0xFF;
            while b == 0xFF {
                let (decrypted, new_offset, new_index) = self.decrypt_byte(data, offset, index);
                b = decrypted;
                offset = new_offset;
                index = new_index;
                byte_high += b as usize;
            }
        }

        offset += byte_high;

        if offset < remaining {
            let (_, new_offset, new_index) = self.decrypt_byte(data, offset, index);
            offset = new_offset;
            index = new_index;

            let (_, new_offset, new_index) = self.decrypt_byte(data, offset, index);
            offset = new_offset;
            index = new_index;

            if byte_low == 0xF {
                let mut b = 0xFF;
                while b == 0xFF {
                    let (decrypted, new_offset, new_index) = self.decrypt_byte(data, offset, index);
                    b = decrypted;
                    offset = new_offset;
                    index = new_index;
                }
            }
        }

        offset
    }
}
