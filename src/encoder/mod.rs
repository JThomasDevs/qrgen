//! Encoder module — handles mode selection and data encoding.

pub mod mode;
pub mod numeric;
pub mod alphanumeric;
pub mod byte;
pub mod padding;

pub use mode::{Mode, Encoder, char_count_bits, best_mode, estimated_bit_length, EncodeBits};
