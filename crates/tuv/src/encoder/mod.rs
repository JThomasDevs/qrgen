//! Encoder module — handles mode selection and data encoding.

pub mod bits;
pub mod mode;
pub mod numeric;
pub mod alphanumeric;
pub mod byte;
pub mod kanji;
pub mod optimize;
pub mod padding;

pub use mode::{
    Mode, Encoder, char_count_bits, best_mode, best_mode_bytes, estimated_bit_length,
    estimated_bit_length_bytes, EncodeBits,
};
pub use optimize::{Optimizer, Parser, Segment, total_encoded_len};
