//! Mask selection and application.
//!
//! Mask coordinates follow ISO/IEC 18004: `i` = row, `j` = column. The
//! `qrcode` reference crate passes `(x, y)` to mask helpers as `(column, row)`,
//! which is the same pair order as `QRMatrix::get(col, row)`.
//!
//! Penalty scoring matches the widely deployed `qrcode` crate (which follows
//! ISO closely), including scoring **after** the candidate format string is
//! drawn — the same order as `Canvas::apply_mask` in that library.

use super::{Module, QRMatrix};
use crate::error_correction::ECCLevel;

/// True iff the mask flips position `(row, col)` for the given mask id.
#[inline]
fn mask(row: usize, col: usize, mask_id: u8) -> bool {
    let i = row as i16;
    let j = col as i16;
    match mask_id {
        0 => (i + j) % 2 == 0,
        1 => i % 2 == 0,
        2 => j % 3 == 0,
        3 => (i + j) % 3 == 0,
        4 => ((i / 2) + (j / 3)) % 2 == 0,
        5 => {
            let p = i as i32 * j as i32;
            (p % 2 + p % 3) == 0
        }
        6 => {
            let p = i as i32 * j as i32;
            (((p % 2) + (p % 3)) % 2) == 0
        }
        7 => {
            let p = i as i32 * j as i32;
            ((((i + j) % 2) as i32 + (p % 3)) % 2) == 0
        }
        _ => false,
    }
}

/// Apply mask `mask_id` in-place to all data modules.
pub fn apply_mask(matrix: &mut QRMatrix, mask_id: u8) {
    let size = matrix.size;
    for row in 0..size {
        for col in 0..size {
            if let Module::Data(val) = matrix.get(col, row) {
                if mask(row, col, mask_id) {
                    matrix.set(col, row, Module::Data(!val));
                }
            }
        }
    }
}

/// Pick the lowest-penalty mask, apply it, then draw format info for that mask.
pub fn find_best_mask(matrix: &mut QRMatrix, ecc: ECCLevel) -> u8 {
    let mut best_mask = 0u8;
    let mut best_score = u32::MAX;
    for mask_id in 0u8..8 {
        let mut trial = matrix.clone();
        apply_mask(&mut trial, mask_id);
        crate::matrix::format_info::place_format_info(&mut trial, ecc, mask_id);
        let score = total_penalty(&trial);
        if score < best_score {
            best_score = score;
            best_mask = mask_id;
        }
    }
    apply_mask(matrix, best_mask);
    crate::matrix::format_info::place_format_info(matrix, ecc, best_mask);
    best_mask
}

#[inline]
fn px(m: &QRMatrix, col: usize, row: usize) -> bool {
    m.get(col, row).is_dark()
}

/// Total penalty (same four terms as the `qrcode` crate), for tests / tooling.
pub(crate) fn total_penalty(m: &QRMatrix) -> u32 {
    let s1_a = adjacent_penalty(m, true);
    let s1_b = adjacent_penalty(m, false);
    let s2 = block_penalty(m);
    let s3_a = finder_penalty(m, true);
    let s3_b = finder_penalty(m, false);
    let s4 = balance_penalty(m);
    s1_a + s1_b + s2 + s3_a + s3_b + s4
}

/// Adjacent-module penalty (ISO 6.8.3.1), matching `qrcode`’s implementation.
fn adjacent_penalty(m: &QRMatrix, horizontal: bool) -> u32 {
    let w = m.size;
    let mut total = 0u32;
    for i in 0..w {
        let mut line: Vec<bool> = Vec::with_capacity(w + 1);
        for j in 0..w {
            line.push(if horizontal {
                px(m, j, i)
            } else {
                px(m, i, j)
            });
        }
        let last = *line.last().unwrap();
        line.push(!last);
        let mut run = 1u32;
        let mut prev = line[0];
        for d in line.iter().skip(1) {
            let d = *d;
            if d == prev {
                run += 1;
            } else {
                if run >= 5 {
                    total += run - 2;
                }
                prev = d;
                run = 1;
            }
        }
        if run >= 5 {
            total += run - 2;
        }
    }
    total
}

fn block_penalty(m: &QRMatrix) -> u32 {
    let w = m.size;
    let mut total = 0u32;
    for row in 0..w - 1 {
        for col in 0..w - 1 {
            if px(m, col, row)
                == px(m, col + 1, row)
                && px(m, col, row) == px(m, col, row + 1)
                && px(m, col, row) == px(m, col + 1, row + 1)
            {
                total += 3;
            }
        }
    }
    total
}

/// Finder-like pattern penalty (ISO 6.8.3.3), ported from `qrcode` crate.
///
/// The reference `qrcode` crate accumulates hits in `u16` and ends with
/// `total_score.wrapping_sub(360)` (not saturating). That matters when the raw
/// score is below 360: saturating would yield 0, but the reference wraps.
fn finder_penalty(m: &QRMatrix, horizontal: bool) -> u32 {
    const PAT: [bool; 7] = [true, false, true, true, true, false, true];
    let w = m.size;
    let mut total_score = 0u16;
    for i in 0..w {
        for j in 0..w.saturating_sub(6) {
            let mut ok = true;
            for k in 0..7 {
                let d = if horizontal {
                    px(m, j + k, i)
                } else {
                    px(m, i, j + k)
                };
                if d != PAT[k] {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            let dark_left = (j.saturating_sub(4)..j).any(|k| {
                if horizontal {
                    px(m, k, i)
                } else {
                    px(m, i, k)
                }
            });
            let dark_right = ((j + 7)..(j + 11).min(w)).any(|k| {
                if horizontal {
                    px(m, k, i)
                } else {
                    px(m, i, k)
                }
            });
            if !dark_left || !dark_right {
                total_score = total_score.wrapping_add(40);
            }
        }
    }
    u32::from(total_score.wrapping_sub(360))
}

fn balance_penalty(m: &QRMatrix) -> u32 {
    let w = m.size;
    let total = w * w;
    let dark = (0..w).flat_map(|row| (0..w).map(move |col| px(m, col, row)))
        .filter(|&d| d)
        .count();
    let ratio = (dark * 200 / total) as u32;
    if ratio >= 100 {
        ratio - 100
    } else {
        100 - ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask0_flips_even_diagonals() {
        assert!(mask(0, 0, 0));
        assert!(!mask(0, 1, 0));
        assert!(mask(1, 1, 0));
    }

    #[test]
    fn mask1_uses_row_index() {
        assert!(mask(0, 5, 1));
        assert!(!mask(1, 5, 1));
    }
}
