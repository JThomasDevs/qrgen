//! QR code generation library.
//!
//! **Tuv** is named for the three letters after QRS in the alphabet — a nod to QR
//! codes and the sequential nature of encoding.
//!
//! # Quick start
//!
//! ```
//! use tuv::QRCode;
//!
//! let qr = QRCode::from("Hello, world!").generate().unwrap();
//! let svg = qr.to_svg(true);
//! let png = qr.to_png(300, true);
//! # let _ = (svg, png);
//! ```
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
pub use qrcode::{QRCode, QRCodeBuilder, QRGenError};
