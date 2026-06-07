//! Micro QR format information (15-bit BCH, single copy around finder).

use crate::error_correction::ECCLevel;
use crate::matrix::{Module, QRMatrix};

/// Masked format codewords from `qrcode` `FORMAT_INFOS_MICRO_QR`.
const FORMAT_INFOS_MICRO_QR: [u16; 32] = [
    0x4445, 0x4172, 0x4e2b, 0x4b1c, 0x55ae, 0x5099, 0x5fc0, 0x5af7, 0x6793, 0x62a4, 0x6dfd, 0x68ca,
    0x7678, 0x734f, 0x7c16, 0x7921, 0x06de, 0x03e9, 0x0cb0, 0x0987, 0x1735, 0x1202, 0x1d5b, 0x186c,
    0x2508, 0x203f, 0x2f66, 0x2a51, 0x34e3, 0x31d4, 0x3e8d, 0x3bba,
];

const FORMAT_INFO_COORDS: [(usize, usize); 15] = [
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (6, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 6),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
];

/// All cells reserved for format information before data placement.
pub fn all_format_info_positions() -> [(usize, usize); 15] {
    FORMAT_INFO_COORDS
}

fn encode_format_number(version: u8, ecc: ECCLevel, mask_id: u8) -> u16 {
    let micro_pattern_number = match mask_id {
        1 => 0b00,
        4 => 0b01,
        6 => 0b10,
        7 => 0b11,
        _ => 0b00,
    };
    let symbol_number = match (version, ecc) {
        (1, ECCLevel::L) => 0b000,
        (2, ECCLevel::L) => 0b001,
        (2, ECCLevel::M) => 0b010,
        (3, ECCLevel::L) => 0b011,
        (3, ECCLevel::M) => 0b100,
        (4, ECCLevel::L) => 0b101,
        (4, ECCLevel::M) => 0b110,
        (4, ECCLevel::Q) => 0b111,
        _ => 0,
    };
    let simple_format_number = (symbol_number << 2) | micro_pattern_number;
    FORMAT_INFOS_MICRO_QR[simple_format_number as usize]
}

/// Draw the 15-bit format codeword for `(version, ecc, mask_id)`.
pub fn place_format_info(matrix: &mut QRMatrix, version: u8, ecc: ECCLevel, mask_id: u8) {
    let code = encode_format_number(version, ecc, mask_id);
    for (k, &(col, row)) in FORMAT_INFO_COORDS.iter().enumerate() {
        let bit_val = ((code >> (14 - k)) & 1) != 0;
        matrix.set(col, row, Module::FormatInfo(bit_val));
    }
}
