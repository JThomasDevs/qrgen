//! Top-level orchestration of the encode pipeline.
//!
//! ```rust,ignore
//! let qr = QRCode::new("Hello, world!", ECCLevel::M, None)?;
//! let svg = qr.to_svg(true);
//! let png = qr.to_png(300, true);
//! ```

use crate::encoder::{best_mode, estimated_bit_length, EncodeBits, Encoder};
use crate::error_correction::{
    ecc_codewords_per_block, interleave, split_into_blocks, total_data_codewords, ECCLevel,
};
use crate::matrix::{Module, QRMatrix};
use crate::render;

#[derive(Debug, thiserror::Error)]
pub enum QRGenError {
    #[error("input too long for ECC level {ecc}: needs more than the largest QR can hold")]
    InputTooLong { ecc: ECCLevel },

    #[error("input too long for version {version} with ECC level {ecc}: needs {needed_bits} bits, capacity is {capacity_bits}")]
    InputTooLongForVersion {
        version: u8,
        ecc: ECCLevel,
        needed_bits: usize,
        capacity_bits: usize,
    },

    #[error("invalid version {version}: must be 1-40")]
    InvalidVersion { version: u8 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct QRCode {
    version: u8,
    matrix: QRMatrix,
    mask_id: u8,
    ecc: ECCLevel,
}

/// Raw data + EC codewords in matrix interleave order (before zigzag placement).
fn interleaved_codewords(input: &str, ecc: ECCLevel, version: u8) -> Result<Vec<u8>, QRGenError> {
    let total_codewords = total_data_codewords(version, ecc);
    let capacity_bits = total_codewords * 8;

    let mut bits = Encoder::encode(input, version)?;
    let needed_bits = bits.len();
    if needed_bits > capacity_bits {
        return Err(QRGenError::InputTooLongForVersion {
            version,
            ecc,
            needed_bits,
            capacity_bits,
        });
    }

    let terminator = (capacity_bits - bits.len()).min(4);
    for _ in 0..terminator {
        bits.push(false);
    }
    while bits.len() % 8 != 0 {
        bits.push(false);
    }

    let mut data_bytes = pack_bits_msb_first(&bits);
    let padding = [0xECu8, 0x11];
    let mut idx = 0;
    while data_bytes.len() < total_codewords {
        data_bytes.push(padding[idx % 2]);
        idx += 1;
    }
    debug_assert_eq!(data_bytes.len(), total_codewords);

    let _ = ecc_codewords_per_block(version, ecc);
    let blocks = split_into_blocks(&data_bytes, version, ecc);
    Ok(interleave(&blocks))
}

/// Build matrix with function patterns and placed data/EC bits, **before** mask selection.
fn matrix_before_mask(input: &str, ecc: ECCLevel, version: u8) -> Result<QRMatrix, QRGenError> {
    let interleaved = interleaved_codewords(input, ecc, version)?;
    let mut matrix = QRMatrix::new(version);
    matrix.place_function_patterns();
    let bit_stream = bytes_to_bits_msb_first(&interleaved);
    crate::matrix::data_placement::place_data(&mut matrix, &bit_stream);
    Ok(matrix)
}

impl QRCode {
    /// Encode `input` into a QR code.
    ///
    /// `version` selects a specific QR version (1-40). When `None`, the
    /// smallest version that holds the input at the requested ECC level is
    /// chosen automatically.
    pub fn new(input: &str, ecc: ECCLevel, version: Option<u8>) -> Result<Self, QRGenError> {
        let version = match version {
            Some(v) => {
                if !(1..=40).contains(&v) {
                    return Err(QRGenError::InvalidVersion { version: v });
                }
                v
            }
            None => smallest_version(input, ecc).ok_or(QRGenError::InputTooLong { ecc })?,
        };

        let mut matrix = matrix_before_mask(input, ecc, version)?;
        let mask_id = crate::matrix::masking::find_best_mask(&mut matrix, ecc);

        // Version info for v ≥ 7.
        if version >= 7 {
            crate::matrix::version_info::place_version_info(&mut matrix, version);
        }

        Ok(Self { version, matrix, mask_id, ecc })
    }

    pub fn to_svg(&self, quiet_zone: bool) -> String {
        render::render_svg(&self.matrix, quiet_zone)
    }

    pub fn to_png(&self, size_px: u32, quiet_zone: bool) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        render::render_png(&self.matrix, size_px, quiet_zone)
    }

    /// Matrix width in modules (excluding any quiet zone in render output).
    #[inline]
    pub fn width(&self) -> usize {
        self.matrix.size
    }

    /// Dark (`true`) or light (`false`) after masking and format info.
    #[inline]
    pub fn module_is_dark(&self, col: usize, row: usize) -> bool {
        self.matrix.get(col, row).is_dark()
    }

    /// Mask pattern id `0..=7` chosen during encoding.
    #[inline]
    pub fn mask_id(&self) -> u8 {
        self.mask_id
    }

    #[cfg(debug_assertions)]
    pub fn debug_matrix(&self) -> String {
        let mut s = format!("size: {}\n", self.matrix.size);
        for j in 0..self.matrix.size {
            for i in 0..self.matrix.size {
                let v = self.matrix.data_at(i, j);
                s.push(match v {
                    Some(true) => '#',
                    Some(false) => '.',
                    None => '?',
                });
            }
            s.push('\n');
        }
        s
    }

    #[cfg(debug_assertions)]
    pub fn get_module(&self, i: usize, j: usize) -> Module {
        self.matrix.get(i, j)
    }

    #[cfg(debug_assertions)]
    pub fn size(&self) -> usize {
        self.matrix.size
    }

    #[cfg(debug_assertions)]
    pub fn debug_full_matrix(&self) -> String {
        use std::collections::HashMap;
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for j in 0..self.matrix.size {
            for i in 0..self.matrix.size {
                let name = match self.matrix.get(i, j) {
                    Module::Data(false) => "Data(false)",
                    Module::Data(true) => "Data(true)",
                    Module::Finder(_) => "Finder",
                    Module::Separator => "Separator",
                    Module::Alignment(_) => "Alignment",
                    Module::Timing(_) => "Timing",
                    Module::FormatInfo(_) => "FormatInfo",
                    Module::VersionInfo(_) => "VersionInfo",
                };
                *counts.entry(name).or_insert(0) += 1;
            }
        }

        let mut s = format!("size: {}\n", self.matrix.size);
        s.push_str("Module counts:\n");
        for (k, v) in &counts {
            s.push_str(&format!("  {}: {}\n", k, v));
        }
        s.push_str("\nMatrix:\n");
        for j in 0..self.matrix.size {
            for i in 0..self.matrix.size {
                let ch = match self.matrix.get(i, j) {
                    Module::Data(false) => '.',
                    Module::Data(true) => '#',
                    Module::Finder(_) => 'F',
                    Module::Separator => 's',
                    Module::Alignment(_) => 'A',
                    Module::Timing(_) => 'T',
                    Module::FormatInfo(_) => 'f',
                    Module::VersionInfo(_) => 'v',
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }
}

// ---- Helpers ----

/// Pack bits to bytes (MSB-first within each byte). `bits` uses `Msb0` order.
fn pack_bits_msb_first(bits: &EncodeBits) -> Vec<u8> {
    let mut out = Vec::with_capacity((bits.len() + 7) / 8);
    let mut byte = 0u8;
    let mut filled = 0u8;
    for bit in bits.iter() {
        byte = (byte << 1) | if *bit { 1 } else { 0 };
        filled += 1;
        if filled == 8 {
            out.push(byte);
            byte = 0;
            filled = 0;
        }
    }
    if filled != 0 {
        byte <<= 8 - filled;
        out.push(byte);
    }
    out
}

/// Expand a byte stream into a flat bit-bool vector, MSB-first within each byte.
fn bytes_to_bits_msb_first(bytes: &[u8]) -> Vec<bool> {
    let mut out = Vec::with_capacity(bytes.len() * 8);
    for &b in bytes {
        for i in (0..8).rev() {
            out.push(((b >> i) & 1) != 0);
        }
    }
    out
}

/// Smallest version (1..40) that fits `input` at the given ECC level.
fn smallest_version(input: &str, ecc: ECCLevel) -> Option<u8> {
    for v in 1u8..=40 {
        let cap_bits = total_data_codewords(v, ecc) * 8;
        let mode = best_mode(input, v);
        let n = estimated_bit_length(input, mode, v);
        if n > cap_bits {
            continue;
        }
        let t = (cap_bits - n).min(4);
        let after_term = n + t;
        let pad_to_byte = (8 - (after_term % 8)) % 8;
        if after_term + pad_to_byte <= cap_bits {
            return Some(v);
        }
    }
    None
}

#[cfg(test)]
mod mask_score_debug {
    use super::*;

    #[test]
    fn interleaved_matches_qrcode_for_v1_digit_1() {
        use qrcode::bits::Bits;
        use qrcode::ec;
        use qrcode::types::{EcLevel, Version};

        let ours = interleaved_codewords("1", ECCLevel::M, 1).expect("ours");

        let mut bits = Bits::new(Version::Normal(1));
        bits.push_optimal_data(b"1").unwrap();
        bits.push_terminator(EcLevel::M).unwrap();
        let raw = bits.into_bytes();
        let (data, eccv) = ec::construct_codewords(&raw, Version::Normal(1), EcLevel::M).unwrap();
        let mut ref_interleaved = Vec::new();
        ref_interleaved.extend_from_slice(&data);
        ref_interleaved.extend_from_slice(&eccv);

        assert_eq!(ours, ref_interleaved, "interleaved codewords differ from qrcode");
    }
}
