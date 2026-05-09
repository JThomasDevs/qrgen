//! Version information encoding and placement (versions 7-40).
//!
//! The 6-bit version number is BCH(18,6)-encoded with generator polynomial
//! `x^12 + x^11 + x^10 + x^9 + x^8 + x^5 + x^2 + 1` (0x1F25), giving an
//! 18-bit codeword. Two copies are placed adjacent to the bottom-left and
//! top-right finder markers per ISO/IEC 18004 §6.10.

use super::{Module, QRMatrix};

/// 18-bit version-info codeword, with the LSB being bit 0.
fn version_codeword(version: u8) -> u32 {
    let mut rem: u32 = version as u32;
    for _ in 0..12 {
        rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
    }
    ((version as u32) << 12) | rem
}

pub fn place_version_info(matrix: &mut QRMatrix, version: u8) {
    if version < 7 {
        return;
    }
    let bits = version_codeword(version);
    let size = matrix.size;
    for i in 0u32..18 {
        let bit = ((bits >> i) & 1) != 0;
        let a = (size - 11) + (i as usize % 3);
        let b = (i / 3) as usize;
        matrix.set(a, b, Module::VersionInfo(bit));
        matrix.set(b, a, Module::VersionInfo(bit));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codewords() {
        // ISO/IEC 18004 Annex D, Table D.1 (sample).
        assert_eq!(version_codeword(7), 0x07C94);
        assert_eq!(version_codeword(40), 0x28C69);
    }
}
