//! Mode selection and indicator encoding.
//!
//! The QR spec defines 4 encoding modes:
//! - **Numeric**, **Alphanumeric**, **Byte**, and **Kanji** (Shift JIS).

use std::cmp::Ordering;

use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use crate::types::{QRGenError, Version};

use super::bits::push_bits;

/// Bit stream for QR payload bits: first pushed bit is the MSB of the logical stream.
pub type EncodeBits = BitVec<u8, Msb0>;

/// Encoding mode, as defined by ISO/IEC 18004.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}
impl Mode {
    /// Returns true if this mode can encode the given input.
    pub fn can_encode(&self, input: &str) -> bool {
        self.can_encode_bytes(input.as_bytes())
    }

    /// Returns true if this mode can encode the given byte slice.
    pub fn can_encode_bytes(&self, input: &[u8]) -> bool {
        match self {
            Mode::Numeric => input.iter().all(|b| b.is_ascii_digit()),
            Mode::Alphanumeric => input.iter().all(|&b| Self::is_alphanumeric_byte(b)),
            Mode::Byte => true,
            Mode::Kanji => input.len().is_multiple_of(2),
        }
    }

    fn is_alphanumeric_byte(b: u8) -> bool {
        matches!(
            b,
            b'0'..=b'9'
                | b'A'..=b'Z'
                | b' '
                | b'$'
                | b'%'
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'/'
                | b':'
        )
    }

    /// 4-bit mode indicator per ISO/IEC 18004 Table 2 (Normal QR).
    pub fn indicator_bits(&self) -> u8 {
        match self {
            Mode::Numeric => 0b0001,
            Mode::Alphanumeric => 0b0010,
            Mode::Byte => 0b0100,
            Mode::Kanji => 0b1000,
        }
    }

    pub fn length_bits_count(self, version: Version) -> usize {
        match version {
            Version::Micro(a) => {
                let a = a as usize;
                match self {
                    Mode::Numeric => 2 + a,
                    Mode::Alphanumeric | Mode::Byte => 1 + a,
                    Mode::Kanji => a,
                }
            }
            Version::Normal(1..=9) => match self {
                Mode::Numeric => 10,
                Mode::Alphanumeric => 9,
                Mode::Byte | Mode::Kanji => 8,
            },
            Version::Normal(10..=26) => match self {
                Mode::Numeric => 12,
                Mode::Alphanumeric => 11,
                Mode::Byte => 16,
                Mode::Kanji => 10,
            },
            Version::Normal(_) => match self {
                Mode::Numeric => 14,
                Mode::Alphanumeric => 13,
                Mode::Byte => 16,
                Mode::Kanji => 12,
            },
        }
    }

    pub fn data_bits_count(self, raw_data_len: usize) -> usize {
        match self {
            Mode::Numeric => (raw_data_len * 10 + 2) / 3,
            Mode::Alphanumeric => (raw_data_len * 11 + 1) / 2,
            Mode::Byte => raw_data_len * 8,
            Mode::Kanji => raw_data_len * 13,
        }
    }

    pub fn max(self, other: Self) -> Self {
        match self.partial_cmp(&other) {
            Some(Ordering::Greater) => self,
            Some(_) => other,
            None => Mode::Byte,
        }
    }
}

impl PartialOrd for Mode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (a, b) if a == b => Some(Ordering::Equal),
            (Mode::Numeric, Mode::Alphanumeric) | (_, Mode::Byte) => Some(Ordering::Less),
            (Mode::Alphanumeric, Mode::Numeric) | (Mode::Byte, _) => Some(Ordering::Greater),
            _ => None,
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
        (Mode::Kanji, 0) => 8,
        (Mode::Kanji, 1) => 10,
        (Mode::Kanji, 2) => 12,
        _ => unreachable!(),    }
}

/// Estimated total encoded bit length for a given mode (mode indicator +
/// char count + mode-specific data bits, *not* including terminator/padding).
pub fn estimated_bit_length(input: &str, mode: Mode, version: u8) -> usize {
    estimated_bit_length_bytes(input.as_bytes(), mode, version)
}

/// Estimated bit length for raw byte input.
pub fn estimated_bit_length_bytes(input: &[u8], mode: Mode, version: u8) -> usize {
    let v = Version::Normal(i16::from(version));
    v.mode_bits_count() + mode.length_bits_count(v) + mode.data_bits_for_bytes(input)
}
/// How many data bits are needed to encode the full string in this mode.
impl Mode {
    pub fn data_bits_for_string(&self, input: &str) -> usize {
        self.data_bits_for_bytes(input.as_bytes())
    }

    pub fn data_bits_for_bytes(&self, input: &[u8]) -> usize {
        match self {
            Mode::Numeric => {
                let n = input.len();
                let full_triplets = n / 3;
                let remainder = n % 3;
                let bits = full_triplets * 10;
                match remainder {
                    0 => bits,
                    1 => bits + 4,
                    2 => bits + 7,
                    _ => unreachable!(),
                }
            }
            Mode::Alphanumeric => {
                let chars = input.len();
                (chars / 2) * 11 + (chars % 2) * 6
            }
            Mode::Byte => crate::encoder::byte::bit_length_bytes(input),
            Mode::Kanji => crate::encoder::kanji::bit_length_bytes(input),
        }
    }
}
/// Auto-select the best mode for the given input string.
pub fn best_mode(input: &str, version: u8) -> Mode {
    best_mode_bytes(input.as_bytes(), version)
}

/// Auto-select the best single mode for raw byte input (legacy helper).
pub fn best_mode_bytes(input: &[u8], version: u8) -> Mode {
    use crate::encoder::optimize::{Optimizer, Parser};

    let v = Version::Normal(i16::from(version));
    let segments = Optimizer::new(Parser::new(input).optimize(v), v).collect::<Vec<_>>();
    segments.first().map(|s| s.mode).unwrap_or(Mode::Byte)
}
/// Encoder — handles mode selection and bit stream generation.
pub struct Encoder;

impl Encoder {
    /// Encode a string with auto-selected best mode.
    pub fn encode(input: &str, version: u8) -> Result<EncodeBits, QRGenError> {
        Self::encode_bytes(input.as_bytes(), version)
    }

    /// Encode raw bytes using the multi-segment optimizer.
    pub fn encode_bytes(input: &[u8], version: u8) -> Result<EncodeBits, QRGenError> {
        let mut bits = crate::bits::Bits::new(Version::Normal(i16::from(version)));
        bits.push_optimal_data(input)?;
        bits_to_encode_bits(&bits)
    }
    /// Encode with a specific mode.
    pub fn encode_with_mode(input: &str, mode: Mode, version: u8) -> Result<EncodeBits, QRGenError> {
        Self::encode_bytes_with_mode(input.as_bytes(), mode, version)
    }

    /// Encode raw bytes with a specific mode.
    pub fn encode_bytes_with_mode(
        input: &[u8],
        mode: Mode,
        version: u8,
    ) -> Result<EncodeBits, QRGenError> {
        let mut bits = EncodeBits::new();

        push_bits(&mut bits, mode.indicator_bits() as u32, 4);

        let count = match mode {
            Mode::Kanji => (input.len() / 2) as u32,
            _ => input.len() as u32,
        };
        let cc_bits = char_count_bits(mode, version);
        push_bits(&mut bits, count, cc_bits);

        match mode {
            Mode::Numeric => {
                let s = std::str::from_utf8(input).expect("numeric mode requires ASCII digits");
                crate::encoder::numeric::encode(s, &mut bits);
            }
            Mode::Alphanumeric => {
                let s = std::str::from_utf8(input)
                    .expect("alphanumeric mode requires ASCII subset bytes");
                crate::encoder::alphanumeric::encode(s, &mut bits);
            }
            Mode::Byte => crate::encoder::byte::encode_bytes(input, &mut bits),
            Mode::Kanji => crate::encoder::kanji::encode_bytes(input, &mut bits)?,
        }
        Ok(bits)
    }
}

fn bits_to_encode_bits(bits: &crate::bits::Bits) -> Result<EncodeBits, QRGenError> {
    let mut out = EncodeBits::new();
    let len = bits.len();
    for i in 0..len {
        out.push(bits.bit_at(i));
    }
    Ok(out)
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
