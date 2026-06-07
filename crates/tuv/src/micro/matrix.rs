//! Micro QR function patterns — single finder, timing on row/col 0, no alignment.

use crate::matrix::{Module, QRMatrix};

/// Create an empty Micro QR matrix (`version` 1–4).
pub fn new_matrix(version: u8) -> QRMatrix {
    let size = (version as usize) * 2 + 9;
    let modules = vec![vec![Module::Data(false); size]; size];
    QRMatrix {
        version,
        size,
        modules,
    }
}

/// Place finder, reserved format info, and timing patterns.
pub fn place_function_patterns(matrix: &mut QRMatrix) {
    place_finder(matrix);
    reserve_format_info(matrix);
    place_timing_patterns(matrix);
}

fn place_finder(matrix: &mut QRMatrix) {
    let cx = 3_i16;
    let cy = 3_i16;
    for j in -3..=4 {
        for i in -3..=4 {
            let dark = match (i, j) {
                (4 | -4, _) | (_, 4 | -4) => false,
                (3 | -3, _) | (_, 3 | -3) => true,
                (2 | -2, _) | (_, 2 | -2) => false,
                _ => true,
            };
            let x = (cx + i) as usize;
            let y = (cy + j) as usize;
            let module = if matches!((i, j), (4 | -4, _) | (_, 4 | -4)) {
                Module::Separator
            } else {
                Module::Finder(dark)
            };
            matrix.set(x, y, module);
        }
    }
}

fn reserve_format_info(matrix: &mut QRMatrix) {
    for (col, row) in super::format_info::all_format_info_positions() {
        matrix.set(col, row, Module::FormatInfo(false));
    }
}

fn place_timing_patterns(matrix: &mut QRMatrix) {
    let s = matrix.size;
    for x in 8..s {
        let dark = x % 2 == 0;
        matrix.set(x, 0, Module::Timing(dark));
    }
    for y in 8..s {
        let dark = y % 2 == 0;
        matrix.set(0, y, Module::Timing(dark));
    }
}
