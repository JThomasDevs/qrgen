//! Format information encoding and placement.
//!
//! Format info encodes the ECC level (2 bits) and the mask pattern ID (3 bits)
//! into a 15-bit codeword via a (15, 5) BCH code, then XOR-masks the result
//! with `0x5412` to avoid all-zero / all-one codes. The same 15 bits are
//! placed in two places around the finder markers for redundancy.
//!
//! Bit ordering: bit 14 is the MSB of the codeword and lands in TL position 0;
//! bit 0 (LSB) lands in TL position 14.

use super::{QRMatrix, Module};
use crate::error_correction::ECCLevel;

/// ISO/IEC 18004 Annex C Table C.1 — masked format codewords.
///
/// `FORMAT_CODEWORDS[ecc_index][mask_id]` is the 15-bit value to place in
/// the matrix (i.e. already XORed with the format mask `0x5412`).
const FORMAT_CODEWORDS: [[u16; 8]; 4] = [
    // L
    [0x77C4, 0x72F3, 0x7DAA, 0x789D, 0x662F, 0x6318, 0x6C41, 0x6976],
    // M
    [0x5412, 0x5125, 0x5E7C, 0x5B4B, 0x45F9, 0x40CE, 0x4F97, 0x4AA0],
    // Q
    [0x355F, 0x3068, 0x3F31, 0x3A06, 0x24B4, 0x2183, 0x2EDA, 0x2BED],
    // H
    [0x1689, 0x13BE, 0x1CE7, 0x19D0, 0x0762, 0x0255, 0x0D0C, 0x083B],
];

fn ecc_index(ecc: ECCLevel) -> usize {
    match ecc {
        ECCLevel::L => 0,
        ECCLevel::M => 1,
        ECCLevel::Q => 2,
        ECCLevel::H => 3,
    }
}

/// Look up the masked 15-bit format codeword for `(ecc, mask_id)`.
pub fn encode_format_info(ecc: ECCLevel, mask_id: u8) -> u16 {
    FORMAT_CODEWORDS[ecc_index(ecc)][mask_id as usize]
}

/// All matrix cells that carry format information (union of both copies).
/// Used to reserve cells before data placement so bits are not written there.
pub fn all_format_info_positions(s: usize) -> Vec<(usize, usize)> {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    for k in 0..15 {
        let (c, r) = tl_position(k);
        set.insert((c, r));
        let (c, r) = other_position(k, s);
        set.insert((c, r));
    }
    set.into_iter().collect()
}

/// Place the 15-bit format codeword into the two redundant locations.
///
/// The MSB (bit 14) of the codeword is placed at TL position 0, and the
/// remaining bits follow in order. The same bits are mirrored into the
/// TR + BL block.
pub fn place_format_info(matrix: &mut QRMatrix, ecc: ECCLevel, mask_id: u8) {
    let code = encode_format_info(ecc, mask_id);
    let s = matrix.size;

    for k in 0..15 {
        let bit_val = ((code >> (14 - k)) & 1) != 0;
        let (tl_col, tl_row) = tl_position(k);
        matrix.set(tl_col, tl_row, Module::FormatInfo(bit_val));

        let (other_col, other_row) = other_position(k, s);
        matrix.set(other_col, other_row, Module::FormatInfo(bit_val));
    }

    // ISO / `qrcode`: single always-dark module left of bottom-left finder
    // (`put(8, -8)` with negative coordinate wrapping in the reference crate).
    matrix.set(8, s - 8, Module::FormatInfo(true));
}

/// 15 (col, row) positions of the format-info bits in the top-left block.
/// `k` runs 0..15 from MSB-first order.
pub fn tl_position(k: usize) -> (usize, usize) {
    match k {
        0 => (0, 8),
        1 => (1, 8),
        2 => (2, 8),
        3 => (3, 8),
        4 => (4, 8),
        5 => (5, 8),
        6 => (7, 8),
        7 => (8, 8),
        8 => (8, 7),
        9 => (8, 5),
        10 => (8, 4),
        11 => (8, 3),
        12 => (8, 2),
        13 => (8, 1),
        14 => (8, 0),
        _ => unreachable!(),
    }
}

/// 15 (col, row) positions in the top-right + bottom-left block, given
/// the matrix size.
fn other_position(k: usize, s: usize) -> (usize, usize) {
    match k {
        0 => (8, s - 1),
        1 => (8, s - 2),
        2 => (8, s - 3),
        3 => (8, s - 4),
        4 => (8, s - 5),
        5 => (8, s - 6),
        6 => (8, s - 7),
        7 => (s - 8, 8),
        8 => (s - 7, 8),
        9 => (s - 6, 8),
        10 => (s - 5, 8),
        11 => (s - 4, 8),
        12 => (s - 3, 8),
        13 => (s - 2, 8),
        14 => (s - 1, 8),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codewords() {
        // Spot-check entries from Annex C.
        assert_eq!(encode_format_info(ECCLevel::L, 0), 0x77C4);
        assert_eq!(encode_format_info(ECCLevel::M, 5), 0x40CE);
        assert_eq!(encode_format_info(ECCLevel::H, 7), 0x083B);
    }
}
