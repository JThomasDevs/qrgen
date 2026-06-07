//! Micro QR data module placement (`timing_pattern_column = 0`).

use crate::matrix::{Module, QRMatrix};

struct DataModuleIter {
    x: isize,
    y: isize,
    width: isize,
    timing_pattern_column: isize,
}

impl DataModuleIter {
    fn new(width: usize) -> Self {
        let w = width as isize;
        Self {
            x: w - 1,
            y: w - 1,
            width: w,
            timing_pattern_column: 0,
        }
    }

    fn next_coord(&mut self) -> Option<(usize, usize)> {
        let adjusted_ref_col = if self.x <= self.timing_pattern_column {
            self.x + 1
        } else {
            self.x
        };
        if adjusted_ref_col <= 0 {
            return None;
        }

        let res = (self.x as usize, self.y as usize);
        let column_type = (self.width - adjusted_ref_col) % 4;

        match column_type {
            2 if self.y > 0 => {
                self.y -= 1;
                self.x += 1;
            }
            0 if self.y < self.width - 1 => {
                self.y += 1;
                self.x += 1;
            }
            0 | 2 if self.x == self.timing_pattern_column + 1 => {
                self.x -= 2;
            }
            _ => {
                self.x -= 1;
            }
        }

        Some(res)
    }
}

fn place_codewords(
    matrix: &mut QRMatrix,
    codewords: &[u8],
    is_half_codeword_at_end: bool,
    coords: &mut DataModuleIter,
) {
    let last_word = if is_half_codeword_at_end {
        codewords.len().saturating_sub(1)
    } else {
        codewords.len()
    };

    for (i, &byte) in codewords.iter().enumerate() {
        let bits_end = if i == last_word { 4 } else { 0 };
        'outside: for j in (bits_end..=7).rev() {
            let bit = (byte & (1 << j)) != 0;
            loop {
                let Some((col, row)) = coords.next_coord() else {
                    return;
                };
                if let Module::Data(_) = matrix.get(col, row) {
                    matrix.set(col, row, Module::Data(bit));
                    continue 'outside;
                }
            }
        }
    }
}

/// Place data then EC codewords into empty data modules.
pub fn place_data(matrix: &mut QRMatrix, data: &[u8], ec: &[u8], half_data: bool) {
    let mut coords = DataModuleIter::new(matrix.size);
    place_codewords(matrix, data, half_data, &mut coords);
    place_codewords(matrix, ec, false, &mut coords);
}

/// Whether the last data codeword uses only 4 bits (Micro M1/L and M3/M).
pub fn data_ends_with_half_codeword(version: u8, ecc: crate::error_correction::ECCLevel) -> bool {
    matches!(
        (version, ecc),
        (1, crate::error_correction::ECCLevel::L) | (3, crate::error_correction::ECCLevel::M)
    )
}
