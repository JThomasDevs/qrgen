//! Matrix module — constructs the QR module grid.

pub mod function_patterns;
pub mod data_placement;
pub mod masking;
pub mod format_info;
pub mod version_info;

pub use self::function_patterns::{QRMatrix, Module};
pub use crate::error_correction::ECCLevel;
