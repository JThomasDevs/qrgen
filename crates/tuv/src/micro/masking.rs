//! Micro QR masking — four patterns only; light-side penalty for auto selection.

use crate::error_correction::ECCLevel;
use crate::matrix::{Module, QRMatrix};

/// Mask patterns supported by Micro QR (QR mask ids 1, 4, 6, 7).
pub const MICRO_MASK_IDS: [u8; 4] = [1, 4, 6, 7];

fn mask_flip(row: usize, col: usize, mask_id: u8) -> bool {
    let i = row as i16;
    let j = col as i16;
    match mask_id {
        1 => i % 2 == 0,
        4 => ((i / 2) + (j / 3)) % 2 == 0,
        6 => {
            let p = i * j;
            ((p % 2) + (p % 3)) % 2 == 0
        }
        7 => {
            let p = i * j;
            (((i + j) % 2) + (p % 3)) % 2 == 0
        }
        _ => false,
    }
}

fn apply_mask(matrix: &mut QRMatrix, mask_id: u8) {
    let size = matrix.size;
    for row in 0..size {
        for col in 0..size {
            if let Module::Data(val) = matrix.get(col, row) {
                if mask_flip(row, col, mask_id) {
                    matrix.set(col, row, Module::Data(!val));
                }
            }
        }
    }
}

fn px(m: &QRMatrix, col: usize, row: usize) -> bool {
    m.get(col, row).is_dark()
}

/// Micro QR exclusive penalty: light modules on the bottom and right edges.
fn light_side_penalty(m: &QRMatrix) -> u32 {
    let w = m.size;
    let last = w - 1;
    let h = (1..w).filter(|&j| !px(m, j, last)).count();
    let v = (1..w).filter(|&j| !px(m, last, j)).count();
    (h + v + 15 * h.max(v)) as u32
}

fn total_penalty(m: &QRMatrix) -> u32 {
    light_side_penalty(m)
}

/// Apply explicit mask or pick lowest light-side penalty, then draw format info.
pub fn apply_mask_selection(
    matrix: &mut QRMatrix,
    version: u8,
    ecc: ECCLevel,
    mask_id: Option<u8>,
) -> u8 {
    match mask_id {
        Some(id) => {
            apply_mask(matrix, id);
            super::format_info::place_format_info(matrix, version, ecc, id);
            id
        }
        None => {
            let mut best_mask = MICRO_MASK_IDS[0];
            let mut best_score = u32::MAX;
            for &id in &MICRO_MASK_IDS {
                let mut trial = matrix.clone();
                apply_mask(&mut trial, id);
                super::format_info::place_format_info(&mut trial, version, ecc, id);
                let score = total_penalty(&trial);
                if score < best_score {
                    best_score = score;
                    best_mask = id;
                }
            }
            apply_mask(matrix, best_mask);
            super::format_info::place_format_info(matrix, version, ecc, best_mask);
            best_mask
        }
    }
}
