//! Mode selection and indicator encoding.
//!
//! The QR spec defines 4 encoding modes. We implement 3:
//! - **Numeric** — digits 0-9
//! - **Alphanumeric** — 0-9, A-Z, and 9 special chars: +%,.-/:
//! - **Byte** — arbitrary UTF-8 (most flexible, used as fallback)
//!
//! Auto-selection picks the mode that produces the shortest encoded bit sequence.

use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use crate::qrcode::QRGenError;

/// Bit stream for QR payload bits: first pushed bit is the MSB of the logical stream.
pub type EncodeBits = BitVec<u8, Msb0>;

/// Encoding mode, as defined by ISO/IEC 18004.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
}

impl Mode {
    /// Returns true if this mode can encode the given input.
    pub fn can_encode(&self, input: &str) -> bool {
        match self {
            Mode::Numeric => input.chars().all(|c| c.is_ascii_digit()),
            Mode::Alphanumeric => input.chars().all(Self::is_alphanumeric_char),
            Mode::Byte => true, // Byte handles everything
        }
    }

    /// Check if a char is in the alphanumeric special set.
    fn is_alphanumeric_char(c: char) -> bool {
        matches!(c, '0'..='9' | 'A'..='Z' | ' ' | '$' | '%' | '*' | '+' | '-' | '.' | '/' | ':')
    }

    /// 4-bit mode indicator per ISO/IEC 18004 Table 2.
    pub fn indicator_bits(&self) -> u8 {
        match self {
            Mode::Numeric => 0b0001,
            Mode::Alphanumeric => 0b0010,
            Mode::Byte => 0b0100,
        }
    }
}

/// Character count field bit lengths per ISO/IEC 18004 Table 3.
pub fn char_count_bits(mode: Mode, version: u8) -> usize {
    let version = version.max(1);
    let group: usize = match version {
        1..=9 => 0,
        10..=26 => 1,
        _ => 2,
    };

    match (mode, group) {
        (Mode::Numeric, 0) => 10,
        (Mode::Numeric, 1) => 12,
        (Mode::Numeric, 2) => 14,
        (Mode::Alphanumeric, 0) => 9,
        (Mode::Alphanumeric, 1) => 11,
        (Mode::Alphanumeric, 2) => 13,
        (Mode::Byte, 0) => 8,
        (Mode::Byte, 1) => 16,
        (Mode::Byte, 2) => 16,
        _ => unreachable!(),
    }
}

/// Push `width` low bits of `value` into the BitVec (MSB first).
fn push_bits(bits: &mut EncodeBits, value: u32, width: usize) {
    for i in 0..width {
        let bit = (value >> (width - 1 - i)) & 1;
        bits.push(bit != 0);
    }
}

/// Estimated total encoded bit length for a given mode (mode indicator +
/// char count + mode-specific data bits, *not* including terminator/padding).
pub fn estimated_bit_length(input: &str, mode: Mode, version: u8) -> usize {
    4 + char_count_bits(mode, version) + mode.data_bits_for_string(input)
}

/// How many data bits are needed to encode the full string in this mode.
impl Mode {
    pub fn data_bits_for_string(&self, input: &str) -> usize {
        match self {
            Mode::Numeric => crate::encoder::numeric::bit_length(input),
            Mode::Alphanumeric => crate::encoder::alphanumeric::bit_length(input),
            Mode::Byte => input.len() * 8,
        }
    }
}

/// Auto-select the best mode for the given input string.
pub fn best_mode(input: &str, version: u8) -> Mode {
    let candidates = [Mode::Numeric, Mode::Alphanumeric, Mode::Byte];
    let mut best = (Mode::Byte, usize::MAX);

    for mode in candidates {
        if !mode.can_encode(input) {
            continue;
        }
        let bits = estimated_bit_length(input, mode, version);
        if bits < best.1 {
            best = (mode, bits);
        }
    }

    best.0
}

/// Encoder — handles mode selection and bit stream generation.
pub struct Encoder;

impl Encoder {
    /// Encode a string with auto-selected best mode.
    pub fn encode(input: &str, version: u8) -> Result<EncodeBits, QRGenError> {
        let mode = best_mode(input, version);
        Self::encode_with_mode(input, mode, version)
    }

    /// Encode with a specific mode.
    pub fn encode_with_mode(input: &str, mode: Mode, version: u8) -> Result<EncodeBits, QRGenError> {
        let mut bits = EncodeBits::new();

        // Mode indicator (4 bits)
        push_bits(&mut bits, mode.indicator_bits() as u32, 4);

        // Character count
        let count = input.len() as u32;
        let cc_bits = char_count_bits(mode, version);
        push_bits(&mut bits, count, cc_bits);

        // Mode-specific data encoding
        match mode {
            Mode::Numeric => crate::encoder::numeric::encode(input, &mut bits),
            Mode::Alphanumeric => crate::encoder::alphanumeric::encode(input, &mut bits),
            Mode::Byte => crate::encoder::byte::encode(input, &mut bits),
        }

        Ok(bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_only() {
        assert!(Mode::Numeric.can_encode("123456"));
        assert!(!Mode::Numeric.can_encode("AB12"));
        assert!(Mode::Alphanumeric.can_encode("AB12"));
    }

    #[test]
    fn test_alphanumeric_special_chars() {
        assert!(Mode::Alphanumeric.can_encode("A+B"));
        assert!(Mode::Alphanumeric.can_encode("1-3"));
        assert!(Mode::Alphanumeric.can_encode("AB")); // plain letters
        assert!(!Mode::Alphanumeric.can_encode("a+b")); // lowercase not allowed
    }

    #[test]
    fn test_byte_catchall() {
        assert!(Mode::Byte.can_encode("anything"));
        assert!(Mode::Byte.can_encode("日本語"));
    }

    #[test]
    fn test_mode_indicator_bits() {
        assert_eq!(Mode::Numeric.indicator_bits(), 0b0001);
        assert_eq!(Mode::Alphanumeric.indicator_bits(), 0b0010);
        assert_eq!(Mode::Byte.indicator_bits(), 0b0100);
    }

    #[test]
    fn test_char_count_bits_by_version_group() {
        // Version 1-9: group 0
        assert_eq!(char_count_bits(Mode::Byte, 1), 8);
        assert_eq!(char_count_bits(Mode::Alphanumeric, 1), 9);
        assert_eq!(char_count_bits(Mode::Numeric, 1), 10);
        // Version 10-26: group 1
        assert_eq!(char_count_bits(Mode::Byte, 10), 16);
        assert_eq!(char_count_bits(Mode::Alphanumeric, 10), 11);
        assert_eq!(char_count_bits(Mode::Numeric, 10), 12);
        // Version 27-40: group 2
        assert_eq!(char_count_bits(Mode::Byte, 27), 16);
        assert_eq!(char_count_bits(Mode::Alphanumeric, 27), 13);
        assert_eq!(char_count_bits(Mode::Numeric, 27), 14);
    }
}
