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
//! # Defaults
//!
//! With no builder options, [`QRCode::from`](QRCode::from) auto-selects the smallest
//! Normal QR version and the lowest ECC level (L → M → Q → H) that fit the input.
//! Use [`QRCodeBuilder::with_ecc`](QRCodeBuilder::with_ecc) or
//! [`QRCodeBuilder::with_version`](QRCodeBuilder::with_version) to fix either field.
//!
//! # Text vs bytes vs Kanji
//!
//! [`QRCode::from`] accepts UTF-8 strings and encodes them in **byte mode** (one byte
//! per character code unit). It does **not** switch to Kanji mode for Japanese text.
//!
//! True **Kanji mode** (Shift JIS double-byte characters) requires raw Shift JIS bytes:
//! use [`QRCode::from_bytes`] or [`Bits::push_kanji_data`](crate::bits::Bits::push_kanji_data)
//! on the low-level [`Bits`](crate::bits::Bits) API.
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
pub mod micro;
pub mod render;
pub mod types;

pub mod qrcode;
pub mod bits;

pub use encoder::{Encoder, Mode};
pub use error_correction::ECCLevel;
pub use matrix::{Module, QRMatrix};
pub use types::{Color, QRGenError, QrResult, Version};
pub use qrcode::{QRCode, QRCodeBuilder};
