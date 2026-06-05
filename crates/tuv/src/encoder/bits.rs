//! Shared bit-stream helpers.

use super::mode::EncodeBits;

/// Push `width` low bits of `value` into the BitVec (MSB first).
pub fn push_bits(bits: &mut EncodeBits, value: u32, width: usize) {
    for i in 0..width {
        let bit = (value >> (width - 1 - i)) & 1;
        bits.push(bit != 0);
    }
}
