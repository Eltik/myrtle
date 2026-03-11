use std::collections::HashMap;
use std::sync::LazyLock;

const CODEC_VORBIS: u32 = 0x0F;
const CHUNK_VORBISDATA: u32 = 0x0B;
const CHUNK_CHANNELS: u32 = 0x01;
const CHUNK_FREQUENCY: u32 = 0x02;

const FREQ_TABLE: [u32; 11] = [
    4000, 8000, 11000, 11025, 16000, 22050, 24000, 32000, 44100, 48000, 96000,
];

const VORBIS_HEADERS_BIN: &[u8] = include_bytes!("vorbis_headers.bin");

struct VorbisSetup {
    header_bytes: &'static [u8],
    seek_bit: u32,
}

static VORBIS_HEADERS: LazyLock<HashMap<u32, VorbisSetup>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    let data = VORBIS_HEADERS_BIN;
    let mut pos = 0;
    while pos + 12 <= data.len() {
        let setup_id = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let seek_bit =
            u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
        let len = u32::from_le_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]])
            as usize;
        pos += 12;
        if pos + len > data.len() {
            break;
        }
        map.insert(
            setup_id,
            VorbisSetup {
                header_bytes: &data[pos..pos + len],
                seek_bit,
            },
        );
        pos += len;
    }
    map
});

fn lookup_vorbis_setup(setup_id: u32) -> Option<&'static VorbisSetup> {
    VORBIS_HEADERS.get(&setup_id)
}

static OGG_CRC_TABLE: LazyLock<[u32; 256]> = LazyLock::new(|| {
    let mut table = [0u32; 256];
    for i in 0..256u32 {
        let mut crc = i << 24;
        for _ in 0..8 {
            if crc & 0x80000000 != 0 {
                crc = (crc << 1) ^ 0x04C11DB7;
            } else {
                crc <<= 1;
            }
        }
        table[i as usize] = crc;
    }
    table
});

fn ogg_crc32(data: &[u8]) -> u32 {
    let table = &*OGG_CRC_TABLE;
    let mut crc = 0u32;
    for &byte in data {
        crc = (crc << 8) ^ table[((crc >> 24) ^ byte as u32) as usize];
    }
    crc
}

fn write_ogg_page(
    out: &mut Vec<u8>,
    header_type: u8,
    granule: i64,
    serial: u32,
    page_seq: u32,
    packets: &[&[u8]],
) {
    let mut segments: Vec<u8> = Vec::new();
    for packet in packets {
        let len = packet.len();
        segments.extend(std::iter::repeat_n(255u8, len / 255));
        segments.push((len % 255) as u8);
    }

    let page_start = out.len();

    out.extend_from_slice(b"OggS");
    out.push(0);
    out.push(header_type);
    out.extend_from_slice(&granule.to_le_bytes());
    out.extend_from_slice(&serial.to_le_bytes());
    out.extend_from_slice(&page_seq.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.push(segments.len() as u8);
    out.extend_from_slice(&segments);

    for packet in packets {
        out.extend_from_slice(packet);
    }

    // Compute and write CRC
    let crc = ogg_crc32(&out[page_start..]);
    let crc_offset = page_start + 22;
    out[crc_offset..crc_offset + 4].copy_from_slice(&crc.to_le_bytes());
}

struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    fn seek(&mut self, bit_pos: usize) {
        self.bit_pos = bit_pos;
    }

    fn read_bit(&mut self) -> u8 {
        if self.bit_pos / 8 >= self.data.len() {
            return 0;
        }
        let byte = self.data[self.bit_pos / 8];
        let bit = (byte >> (self.bit_pos % 8)) & 1;
        self.bit_pos += 1;
        bit
    }

    fn read_bits(&mut self, n: usize) -> u32 {
        let mut val = 0u32;
        for i in 0..n {
            let bit = self.read_bit() as u32;
            if i < 32 {
                val |= bit << i;
            }
        }
        val
    }
}

/// Extract mode block flags from a Vorbis setup header.
fn extract_block_flags(header_bytes: &[u8], seek_bit: u32) -> Vec<bool> {
    let mut reader = BitReader::new(header_bytes);
    reader.seek(seek_bit as usize);

    let num_modes = reader.read_bits(6) + 1;
    let mut flags = Vec::with_capacity(num_modes as usize);
    for _ in 0..num_modes {
        let block_flag = reader.read_bit() != 0;
        // Skip: window_type(16) + transform_type(16) + mapping(8)
        reader.read_bits(16 + 16 + 8);
        flags.push(block_flag);
    }
    flags
}

/// Determine block size of a Vorbis audio packet.
fn get_packet_blocksize(packet: &[u8], block_flags: &[bool]) -> u32 {
    if packet.is_empty() {
        return 0;
    }
    let mut reader = BitReader::new(packet);
    if reader.read_bit() != 0 {
        return 0;
    }

    let mode = if block_flags.len() > 1 {
        let bits_needed = ilog(block_flags.len() as u32 - 1);
        reader.read_bits(bits_needed) as usize
    } else {
        0
    };

    if mode >= block_flags.len() {
        return 0;
    }

    if block_flags[mode] { 2048 } else { 256 }
}

/// Number of bits needed to represent value
fn ilog(value: u32) -> usize {
    if value == 0 {
        return 0;
    }
    32 - value.leading_zeros() as usize
}

struct Fsb5Sample {
    frequency: u32,
    channels: u8,
    data_offset: usize,
    num_samples: u32,
    vorbis_setup_id: u32,
}

/// Parse an FSB5 file and extract all Vorbis samples as OGG files.
pub fn fsb5_to_ogg(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    if data.len() < 60 || &data[0..4] != b"FSB5" {
        return Err("Not an FSB5 file".into());
    }

    let r32 =
        |off: usize| u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);

    let version = r32(4);
    let num_samples = r32(8);
    let sample_headers_size = r32(12);
    let name_table_size = r32(16);
    let _data_size = r32(20);
    let codec = r32(24);

    if codec != CODEC_VORBIS {
        return Err(format!(
            "Unsupported codec: {codec} (expected Vorbis/{CODEC_VORBIS})"
        ));
    }

    let base_header_size: usize = if version == 1 { 60 } else { 64 };

    let mut samples = Vec::with_capacity(num_samples as usize);
    let mut pos = base_header_size;

    for _ in 0..num_samples {
        if pos + 8 > data.len() {
            return Err("Truncated sample header".into());
        }
        let val = u64::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        pos += 8;

        let has_chunks = val & 1 != 0;
        let freq_idx = ((val >> 1) & 0x1F) as usize;
        let ch_bits = (val >> 6) & 0x3;
        let data_offset = (((val >> 8) & 0x3FFFFFF) as usize) * 32;
        let num_pcm_samples = ((val >> 34) & 0x3FFFFFFF) as u32;

        let mut frequency = if freq_idx < FREQ_TABLE.len() {
            FREQ_TABLE[freq_idx]
        } else {
            44100
        };
        let mut channels = match ch_bits {
            0 => 1u8,
            1 => 2,
            2 => 6,
            3 => 8,
            _ => 1,
        };
        let mut vorbis_setup_id = 0u32;

        if has_chunks {
            loop {
                if pos + 4 > data.len() {
                    return Err("Truncated chunk header".into());
                }
                let chunk_val = r32(pos);
                pos += 4;

                let has_next = chunk_val & 1 != 0;
                let chunk_size = ((chunk_val >> 1) & 0xFFFFFF) as usize;
                let chunk_type = (chunk_val >> 25) & 0x7F;

                if pos + chunk_size > data.len() {
                    return Err("Truncated chunk data".into());
                }

                match chunk_type {
                    CHUNK_VORBISDATA => {
                        if chunk_size >= 4 {
                            vorbis_setup_id = r32(pos);
                        }
                    }
                    CHUNK_FREQUENCY => {
                        if chunk_size >= 4 {
                            frequency = r32(pos);
                        }
                    }
                    CHUNK_CHANNELS => {
                        if chunk_size >= 1 {
                            channels = data[pos];
                        }
                    }
                    _ => {}
                }

                pos += chunk_size;
                if !has_next {
                    break;
                }
            }
        }

        samples.push(Fsb5Sample {
            frequency,
            channels,
            data_offset,
            num_samples: num_pcm_samples,
            vorbis_setup_id,
        });
    }

    let data_section_start =
        base_header_size + sample_headers_size as usize + name_table_size as usize;

    let mut results = Vec::new();
    for (i, sample) in samples.iter().enumerate() {
        let sample_start = data_section_start + sample.data_offset;
        let sample_end = if i + 1 < samples.len() {
            data_section_start + samples[i + 1].data_offset
        } else {
            data.len()
        };

        if sample_start >= data.len() || sample_end > data.len() {
            continue;
        }

        let sample_data = &data[sample_start..sample_end];
        match rebuild_ogg(
            sample_data,
            sample.frequency,
            sample.channels,
            sample.vorbis_setup_id,
            sample.num_samples,
        ) {
            Ok(ogg) => results.push((i.to_string(), ogg)),
            Err(e) => eprintln!("  FSB5 sample {i}: {e}"),
        }
    }

    if results.is_empty() {
        return Err("No samples could be decoded".into());
    }

    Ok(results)
}

/// Rebuild a valid OGG/Vorbis file from FSB5 sample data.
fn rebuild_ogg(
    sample_data: &[u8],
    frequency: u32,
    channels: u8,
    setup_id: u32,
    _total_samples: u32,
) -> Result<Vec<u8>, String> {
    let setup =
        lookup_vorbis_setup(setup_id).ok_or_else(|| format!("Unknown setup_id: {setup_id}"))?;

    let block_flags = extract_block_flags(setup.header_bytes, setup.seek_bit);

    // Vorbis identification header (30 bytes)
    let mut ident = Vec::with_capacity(30);
    ident.push(0x01);
    ident.extend_from_slice(b"vorbis");
    ident.extend_from_slice(&0u32.to_le_bytes()); // version
    ident.push(channels);
    ident.extend_from_slice(&frequency.to_le_bytes());
    ident.extend_from_slice(&0i32.to_le_bytes()); // bitrate_max
    ident.extend_from_slice(&0i32.to_le_bytes()); // bitrate_nominal
    ident.extend_from_slice(&0i32.to_le_bytes()); // bitrate_min
    ident.push(0xB8); // blocksizes: short=256 (2^8), long=2048 (2^11)
    ident.push(0x01); // framing

    // Vorbis comment header
    let mut comment = Vec::new();
    comment.push(0x03);
    comment.extend_from_slice(b"vorbis");
    comment.extend_from_slice(&0u32.to_le_bytes()); // vendor length
    comment.extend_from_slice(&0u32.to_le_bytes()); // comment count
    comment.push(0x01); // framing

    let setup_header = setup.header_bytes;

    let serial: u32 = 1;
    let mut out = Vec::new();
    let mut page_seq = 0u32;

    // Page 0: BOS
    write_ogg_page(&mut out, 0x02, 0, serial, page_seq, &[&ident]);
    page_seq += 1;

    // Page 1: comment + setup
    write_ogg_page(
        &mut out,
        0x00,
        0,
        serial,
        page_seq,
        &[&comment, setup_header],
    );
    page_seq += 1;

    // Audio pages
    let mut data_pos = 0usize;
    let mut granule_pos: i64 = 0;
    let mut prev_blocksize: u32 = 0;
    let mut page_packets: Vec<Vec<u8>> = Vec::new();
    let mut page_data_size = 0usize;
    let max_page_size = 65025;

    while data_pos + 2 <= sample_data.len() {
        let packet_size =
            u16::from_le_bytes([sample_data[data_pos], sample_data[data_pos + 1]]) as usize;
        data_pos += 2;

        if packet_size == 0 || packet_size == 0xFFFF {
            break;
        }
        if data_pos + packet_size > sample_data.len() {
            break;
        }

        let packet = &sample_data[data_pos..data_pos + packet_size];
        data_pos += packet_size;

        // Update granule position
        let blocksize = get_packet_blocksize(packet, &block_flags);
        if blocksize > 0 && prev_blocksize > 0 {
            granule_pos += ((blocksize + prev_blocksize) / 4) as i64;
        }
        if blocksize > 0 {
            prev_blocksize = blocksize;
        }

        page_data_size += packet_size;
        page_packets.push(packet.to_vec());

        if page_data_size >= max_page_size || page_packets.len() >= 255 {
            let refs: Vec<&[u8]> = page_packets.iter().map(|p| p.as_slice()).collect();
            write_ogg_page(&mut out, 0x00, granule_pos, serial, page_seq, &refs);
            page_seq += 1;
            page_packets.clear();
            page_data_size = 0;
        }
    }

    // Flush remaining as EOS page
    if !page_packets.is_empty() {
        let refs: Vec<&[u8]> = page_packets.iter().map(|p| p.as_slice()).collect();
        write_ogg_page(&mut out, 0x04, granule_pos, serial, page_seq, &refs);
    } else if page_seq >= 2 {
        write_ogg_page(&mut out, 0x04, granule_pos, serial, page_seq, &[]);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vorbis_headers_loaded() {
        assert!(
            !VORBIS_HEADERS.is_empty(),
            "Vorbis headers table should not be empty"
        );
        println!("Loaded {} vorbis setup headers", VORBIS_HEADERS.len());
        assert!(
            lookup_vorbis_setup(3072374402).is_some(),
            "Should find setup_id from CN_004"
        );
    }

    #[test]
    fn test_extract_block_flags() {
        let setup = lookup_vorbis_setup(3072374402).unwrap();
        let flags = extract_block_flags(setup.header_bytes, setup.seek_bit);
        assert!(!flags.is_empty(), "Should extract at least one block flag");
        println!("Block flags: {:?}", flags);
    }

    #[test]
    fn test_fsb5_to_ogg() {
        let data = match std::fs::read("test_output/audio/CN_004.bytes") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };

        let results = fsb5_to_ogg(&data).unwrap();
        assert!(!results.is_empty(), "Should extract at least one sample");

        for (name, ogg) in &results {
            println!("Sample {name}: {} bytes", ogg.len());
            assert_eq!(&ogg[0..4], b"OggS", "Should start with OggS");
            let path = format!("test_output/audio/CN_004_decoded_{name}.ogg");
            std::fs::write(&path, ogg).unwrap();
            println!("Wrote {path}");
        }
    }
}
