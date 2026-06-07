//! Error correction module.
//!
//! QR codes use Reed-Solomon error correction over GF(2^8).
//! This module handles:
//! - Galois Field arithmetic
//! - ECC codeword generation
//! - Block splitting and interleaving

pub mod reed_solomon;
pub mod generate;
pub mod blocks;

pub use reed_solomon::GF256;
pub use generate::generate_ecc;
pub use blocks::{
    split_into_blocks, interleave, DataBlock, total_data_codewords, ecc_codewords_per_block,
    max_allowed_errors,
};

/// Error correction level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ECCLevel {
    L, // 7% recovery
    M, // 15% recovery
    Q, // 25% recovery
    H, // 30% recovery
}

impl std::fmt::Display for ECCLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ECCLevel::L => write!(f, "L"),
            ECCLevel::M => write!(f, "M"),
            ECCLevel::Q => write!(f, "Q"),
            ECCLevel::H => write!(f, "H"),
        }
    }
}

impl ECCLevel {
    /// 2-bit indicator per spec.
    pub fn indicator_bits(&self) -> u8 {
        match self {
            ECCLevel::L => 0b01,
            ECCLevel::M => 0b00,
            ECCLevel::Q => 0b11,
            ECCLevel::H => 0b10,
        }
    }
}
