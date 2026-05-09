//! QR code generation library.
//!
//! # Architecture
//!
//! ```text
//! Input -> Mode Select -> Reed-Solomon ECC -> Block Interleave
//!        -> Function Patterns -> Matrix -> Data Placement -> Masking
//!        -> Format Info -> SVG / PNG
//! ```

pub mod encoder;
pub mod error_correction;
pub mod matrix;
pub mod render;

pub mod qrcode;

// Public exports
pub use encoder::{Mode, Encoder};
pub use error_correction::ECCLevel;
pub use matrix::{QRMatrix, Module};
pub use qrcode::{QRCode, QRGenError};
