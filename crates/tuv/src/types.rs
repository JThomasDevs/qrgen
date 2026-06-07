//! Shared types mirroring the reference `qrcode` crate API.

use std::fmt;
use std::ops::Not;

use crate::error_correction::ECCLevel;

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

    #[error("invalid version {version}: must be 1-40 for Normal QR, 1-4 for Micro QR")]
    InvalidVersion { version: u8 },

    #[error("invalid mask id {mask_id}: must be 0-7")]
    InvalidMaskId { mask_id: u8 },

    #[error("some characters in the data cannot be supported by the provided QR code version")]
    UnsupportedCharacterSet,

    #[error("a character not belonging to the character set is found")]
    InvalidCharacter,

    #[error("invalid ECI designator {designator}: must be 0-999999")]
    InvalidEciDesignator { designator: u32 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenient alias for QR generation results.
pub type QrResult<T> = Result<T, QRGenError>;

/// The color of a module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    Light,
    Dark,
}

impl Color {
    /// Select a value according to module color.
    pub fn select<T>(self, dark: T, light: T) -> T {
        match self {
            Color::Light => light,
            Color::Dark => dark,
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Color::Light => Color::Dark,
            Color::Dark => Color::Light,
        }
    }
}

impl From<bool> for Color {
    fn from(dark: bool) -> Self {
        if dark {
            Color::Dark
        } else {
            Color::Light
        }
    }
}

/// QR symbol version (Normal v1–40 or Micro v1–4).
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Version {
    Normal(i16),
    Micro(i16),
}

impl Version {
    /// Matrix width/height in modules (excluding quiet zone).
    pub fn width(self) -> usize {
        match self {
            Version::Normal(v) => (v as usize) * 4 + 17,
            Version::Micro(v) => (v as usize) * 2 + 9,
        }
    }

    /// Whether this is a Micro QR version.
    pub fn is_micro(self) -> bool {
        matches!(self, Version::Micro(_))
    }

    /// Mode indicator bit width for this version.
    pub fn mode_bits_count(self) -> usize {
        if let Version::Micro(a) = self {
            (a - 1).max(0) as usize
        } else {
            4
        }
    }

    /// Normal QR version number (1–40), if applicable.
    pub fn normal_number(self) -> Option<u8> {
        match self {
            Version::Normal(v) if (1..=40).contains(&v) => Some(v as u8),
            _ => None,
        }
    }

    /// Micro QR version number (1–4), if applicable.
    pub fn micro_number(self) -> Option<u8> {
        match self {
            Version::Micro(v) if (1..=4).contains(&v) => Some(v as u8),
            _ => None,
        }
    }

    /// Fetch from a 44×4 table (40 Normal + 4 Micro rows, 4 ECC columns).
    pub fn fetch<T>(self, ecc: ECCLevel, table: &[[T; 4]]) -> QrResult<T>
    where
        T: PartialEq + Default + Copy,
    {
        let ecc_idx = match ecc {
            ECCLevel::L => 0,
            ECCLevel::M => 1,
            ECCLevel::Q => 2,
            ECCLevel::H => 3,
        };

        match self {
            Version::Normal(v @ 1..=40) => Ok(table[(v - 1) as usize][ecc_idx]),
            Version::Micro(v @ 1..=4) => {
                let obj = table[(v + 39) as usize][ecc_idx];
                if obj != T::default() {
                    Ok(obj)
                } else {
                    Err(QRGenError::InvalidVersion {
                        version: v as u8,
                    })
                }
            }
            Version::Normal(v) => Err(QRGenError::InvalidVersion {
                version: v.max(0) as u8,
            }),
            Version::Micro(v) => Err(QRGenError::InvalidVersion {
                version: v.max(0) as u8,
            }),
        }
    }
}

impl From<u8> for Version {
    fn from(v: u8) -> Self {
        Version::Normal(i16::from(v))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Normal(v) => write!(f, "Normal({v})"),
            Version::Micro(v) => write!(f, "Micro({v})"),
        }
    }
}
