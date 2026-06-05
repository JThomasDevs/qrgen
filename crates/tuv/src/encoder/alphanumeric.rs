//! Alphanumeric mode encoding.
//!
//! Encodes a restricted character set: 0-9, A-Z, plus space, $, %, *, +, -, ., /, : (45 total).
//! Each character maps to a value 0-44 using the QR spec's encoding table.
//!
//! Chunking rules (per spec):
//! - Pairs of characters → 11 bits (each char contributes up to 6 bits: log2(45)=5.95≈6)
//! - Single remaining char → 6 bits
//!
//! The value calculation: (val1 * 45) + val2 for a pair.

use super::mode::EncodeBits;

/// QR spec alphanumeric encoding table: char → value (0-44)
const ALPHANUMERIC_TABLE: &[(&str, u8)] = &[
    ("0", 0), ("1", 1), ("2", 2), ("3", 3), ("4", 4),
    ("5", 5), ("6", 6), ("7", 7), ("8", 8), ("9", 9),
    ("A", 10), ("B", 11), ("C", 12), ("D", 13), ("E", 14),
    ("F", 15), ("G", 16), ("H", 17), ("I", 18), ("J", 19),
    ("K", 20), ("L", 21), ("M", 22), ("N", 23), ("O", 24),
    ("P", 25), ("Q", 26), ("R", 27), ("S", 28), ("T", 29),
    ("U", 30), ("V", 31), ("W", 32), ("X", 33), ("Y", 34),
    ("Z", 35), (" ", 36), ("$", 37), ("%", 38), ("*", 39),
    ("+", 40), ("-", 41), (".", 42), ("/", 43), (":", 44),
];

/// Look up a character's alphanumeric value. Panics if char not in set.
fn char_value(c: char) -> u8 {
    for (ch, val) in ALPHANUMERIC_TABLE {
        if ch.chars().next() == Some(c) {
            return *val;
        }
    }
    panic!("character '{}' not in alphanumeric set", c);
}

/// Push `width` low bits of `value` into the BitVec (MSB first).
fn push_bits(bits: &mut EncodeBits, value: u32, width: usize) {
    for i in 0..width {
        let bit = (value >> (width - 1 - i)) & 1;
        bits.push(bit != 0);
    }
}

/// Number of bits needed to encode an alphanumeric string.
pub fn bit_length(input: &str) -> usize {
    let chars = input.chars().count();
    let pairs = chars / 2;
    let remainder = chars % 2;
    (pairs * 11) + (remainder * 6)
}

/// Encode an alphanumeric string into the bit vector.
pub fn encode(input: &str, bits: &mut EncodeBits) {
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let pairs = n / 2;

    for i in 0..pairs {
        let idx = i * 2;
        let val1 = char_value(chars[idx]);
        let val2 = char_value(chars[idx + 1]);
        let combined = (val1 as u16) * 45 + (val2 as u16);
        push_bits(bits, combined as u32, 11);
    }

    if n % 2 == 1 {
        // Odd one out: 6 bits
        let val = char_value(chars[n - 1]);
        push_bits(bits, val as u32, 6);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_length() {
        assert_eq!(bit_length("AB"), 11);     // exactly 1 pair = 11 bits
        assert_eq!(bit_length("A"), 6);      // exactly 1 char = 6 bits
        assert_eq!(bit_length("ABC"), 17);   // 1 pair + 1 remainder: 11 + 6 = 17
    }

    #[test]
    fn test_encode_hello() {
        let mut bits = EncodeBits::new();
        encode("HELLO", &mut bits);
        assert_eq!(bits.len(), 28);
    }
}
