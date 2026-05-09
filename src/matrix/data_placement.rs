//! Data module placement (ISO/IEC 18004 §6.7.3).
//!
//! Data is placed in pairs of columns, snaking up and down from the
//! bottom-right corner. Within each two-column band, bits go right-then-left
//! per row. Cells already occupied by function patterns or reserved areas
//! (anything that isn't `Module::Data(_)`) are skipped.
//!
//! Column 6 is the vertical timing strip; the band that would include it is
//! shifted left by one column instead.

use super::{Module, QRMatrix};

pub fn place_data(matrix: &mut QRMatrix, data_bits: &[bool]) {
    let size = matrix.size as isize;
    let mut bit_idx = 0usize;

    // Pair-of-columns walks from right edge to left, skipping the timing column (col 6).
    let mut col: isize = size - 1;
    while col > 0 {
        if col == 6 {
            col -= 1;
        }
        let going_up = ((size - 1 - col) / 2) % 2 == 0;
        for k in 0..size {
            let row = if going_up { size - 1 - k } else { k };
            for dx in 0..2 {
                let c = col - dx;
                if c < 0 {
                    break;
                }
                if bit_idx >= data_bits.len() {
                    return;
                }
                if let Module::Data(_) = matrix.get(c as usize, row as usize) {
                    let bit = data_bits[bit_idx];
                    matrix.set(c as usize, row as usize, Module::Data(bit));
                    bit_idx += 1;
                }
            }
        }
        col -= 2;
    }
}
