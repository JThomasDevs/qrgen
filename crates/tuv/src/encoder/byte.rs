//! Byte mode encoding.
//!
//! Byte mode encodes arbitrary byte sequences. The QR spec originally used
//! ISO 8859-1, but modern QR codes use UTF-8, which is what we implement.
//!
//! Each byte → 8 bits. Simple and universal.

use super::mode::EncodeBits;

/// Push `width` low bits of `value` into the BitVec (MSB first).
fn push_bits(bits: &mut EncodeBits, value: u32, width: usize) {
    for i in 0..width {
        let bit = (value >> (width - 1 - i)) & 1;
        bits.push(bit != 0);
    }
}

/// Encode a string as UTF-8 bytes into the bit vector.
pub fn encode(input: &str, bits: &mut EncodeBits) {
    for byte in input.bytes() {
        push_bits(bits, byte as u32, 8);
    }
}

/// Number of bits = bytes × 8
pub fn bit_length(input: &str) -> usize {
    input.len() * 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_length() {
        assert_eq!(bit_length("ABC"), 24);
    }

    #[test]
    fn test_encode_bytes() {
        let mut bits = EncodeBits::new();
        encode("AB", &mut bits);
        assert_eq!(bits.len(), 16);
    }
}
