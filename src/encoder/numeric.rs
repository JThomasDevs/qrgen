//! Numeric mode encoding.
//!
//! Numeric mode encodes decimal digits. The spec groups digits in triples:
//! - 3 digits → 10 bits (values 0-999)
//! - 2 digits → 7 bits (values 0-99)
//! - 1 digit → 4 bits (values 0-9)
//!
//! Groups are processed left-to-right. The final group uses the smallest
//! representation if it has fewer than 3 digits.

use super::mode::EncodeBits;

/// Number of bits needed to encode a numeric string.
pub fn bit_length(input: &str) -> usize {
    let n = input.chars().count();
    let full_triplets = n / 3;
    let remainder = n % 3;

    let mut bits = 0;
    for _ in 0..full_triplets {
        bits += 10; // each full triplet is 10 bits
    }

    match remainder {
        0 => bits,
        1 => bits + 4,  // single digit = 4 bits
        2 => bits + 7,  // two digits = 7 bits
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

/// Encode a numeric string into the bit vector.
pub fn encode(input: &str, bits: &mut EncodeBits) {
    let digits: Vec<u8> = input
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();

    let n = digits.len();
    let full_triplets = n / 3;
    let remainder = n % 3;

    // Process full triplets (3 digits → 10 bits)
    for i in 0..full_triplets {
        let idx = i * 3;
        let value = (digits[idx] as u16) * 100 + (digits[idx + 1] as u16) * 10 + (digits[idx + 2] as u16);
        push_bits(bits, value as u32, 10);
    }

    // Process remainder
    match remainder {
        0 => {}
        1 => {
            // Single digit: 4 bits
            push_bits(bits, digits[n - 1] as u32, 4);
        }
        2 => {
            // Two digits: 7 bits
            let value = (digits[n - 2] as u16) * 10 + (digits[n - 1] as u16);
            push_bits(bits, value as u32, 7);
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_length() {
        assert_eq!(bit_length("123"), 10);   // exactly 3 digits = 10 bits
        assert_eq!(bit_length("1234"), 14);  // 1 triplet (10) + 1 digit (4) = 14
        assert_eq!(bit_length("12"), 7);     // exactly 2 digits = 7 bits
        assert_eq!(bit_length("1"), 4);      // exactly 1 digit = 4 bits
        assert_eq!(bit_length("12345"), 17); // 1 triplet (10) + 2 digits (7) = 24
    }

    #[test]
    fn test_encode_triplets() {
        let mut bits = EncodeBits::new();
        encode("123", &mut bits);
        assert_eq!(bits.len(), 10);
    }

    #[test]
    fn test_encode_remainder() {
        let mut bits = EncodeBits::new();
        encode("12", &mut bits);
        assert_eq!(bits.len(), 7);
    }
}
