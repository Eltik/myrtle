/// Fix Arknights' modified LZ4 format before standard decompression.
/// Swaps nibbles in sequence tokens and byte-swaps match offsets.
pub fn fix_lz4ak(data: &[u8], uncompressed_size: usize) -> Vec<u8> {
    let mut fixed = data.to_vec();
    let mut ip = 0;
    let mut op = 0;

    while ip < fixed.len() {
        // Swap nibbles: Arknights stores (match_len, literal_len)
        // but standard LZ4 expects (literal_len, match_len)
        let token = fixed[ip];
        let literal_len_nibble = token & 0x0F;
        let match_len_nibble = (token >> 4) & 0x0F;
        fixed[ip] = (literal_len_nibble << 4) | match_len_nibble;
        ip += 1;

        // Skip literal length extension bytes
        let mut literal_len = literal_len_nibble as usize;
        if literal_len == 15 {
            loop {
                if ip >= fixed.len() {
                    break;
                }
                let b = fixed[ip];
                literal_len += b as usize;
                ip += 1;
                if b != 0xFF {
                    break;
                }
            }
        }

        // Skip literal bytes
        ip += literal_len;
        op += literal_len;
        if op >= uncompressed_size {
            break;
        }

        // Byte-swap the 2-byte match offset (big-endian → little-endian)
        if ip + 1 >= fixed.len() {
            break;
        }
        fixed.swap(ip, ip + 1);
        ip += 2;

        // Skip match length extension bytes
        let mut match_len = match_len_nibble as usize;
        if match_len == 15 {
            loop {
                if ip >= fixed.len() {
                    break;
                }
                let b = fixed[ip];
                match_len += b as usize;
                ip += 1;
                if b != 0xFF {
                    break;
                }
            }
        }
        op += match_len + 4; // min match = 4
    }

    fixed
}
