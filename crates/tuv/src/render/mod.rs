//! Rendering module — converts the QR matrix to SVG or PNG.

pub mod svg;
pub mod png;

pub use svg::render_svg;
pub use png::render_png;

pub use crate::qrcode::QRGenError;
