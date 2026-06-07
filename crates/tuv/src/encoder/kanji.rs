//! Shift JIS Kanji mode encoding (ISO/IEC 18004).

use crate::types::QRGenError;

use super::bits::push_bits;
use super::mode::EncodeBits;

/// Encode even-length Shift JIS double-byte data.
pub fn encode_bytes(data: &[u8], bits: &mut EncodeBits) -> Result<(), QRGenError> {
    if !data.len().is_multiple_of(2) {
        return Err(QRGenError::InvalidCharacter);
    }
    for kanji in data.chunks(2) {
        let cp = u16::from(kanji[0]) * 256 + u16::from(kanji[1]);
        let bytes = if cp < 0xe040 { cp - 0x8140 } else { cp - 0xc140 };
        let number = (bytes >> 8) * 0xc0 + (bytes & 0xff);
        push_bits(bits, number as u32, 13);
    }
    Ok(())
}

pub fn bit_length_bytes(data: &[u8]) -> usize {
    (data.len() / 2) * 13
}
