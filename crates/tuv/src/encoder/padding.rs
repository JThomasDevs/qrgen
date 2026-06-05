//! Padding and structure final message.
//!
//! After encoding data bits, the bit stream is "structured" into bytes:
//! 1. Append terminator bits (0x00, up to 4 bits)
//! 2. If bit stream is not byte-aligned, pad with 0s to align
//! 3. If still not full: append padding codewords alternating 0xEC and 0x11
//! 4. Split into blocks for interleaving (handled in error_correction module)

use crate::encoder::mode::EncodeBits;

/// Pad a bit vector to byte alignment with zeros.
pub fn pad_to_byte_boundary(bits: &mut EncodeBits) {
    let remainder = bits.len() % 8;
    if remainder != 0 {
        // Push zeros by pushing false bits
        for _ in 0..(8 - remainder) {
            bits.push(false);
        }
    }
}

/// Calculate total data capacity in bytes for a given version and ECC level.
// TODO: implement capacity table for versions 2-40
pub fn data_capacity(version: u8, ecc: crate::error_correction::ECCLevel) -> Option<usize> {
    use crate::error_correction::ECCLevel;
    debug_assert!(version == 1, "data_capacity: unsupported version");
    match (version, ecc) {
        (1, ECCLevel::L) => Some(17),
        (1, ECCLevel::M) => Some(14),
        (1, ECCLevel::Q) => Some(11),
        (1, ECCLevel::H) => Some(7),
        _ => None,
    }
}

/// Fill remaining capacity with padding codewords 0xEC, 0x11 alternating.
pub fn fill_with_padding(bits: &mut Vec<u8>, target_bytes: usize) {
    let padding = [0xEC, 0x11];
    let mut idx = 0;
    while bits.len() < target_bytes {
        bits.push(padding[idx % 2]);
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_capacity_table() {
        assert_eq!(data_capacity(1, crate::error_correction::ECCLevel::L), Some(17));
    }
}
