//! PNG rendering.
//!
//! PNG is raster output — fixed pixel dimensions, universally supported.

use image::{ImageBuffer, Rgba};
use super::super::{QRMatrix, Module};

#[inline]
fn is_dark(module: &Module) -> bool {
    module.is_dark()
}

/// Render a QR matrix as an RGBA PNG image.
///
/// `size_px` — width and height in pixels (output is square).
/// `quiet_zone` — if true, adds 4-module white margin around the QR.
pub fn render_png(matrix: &QRMatrix, size_px: u32, quiet_zone: bool) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let margin = if quiet_zone { 4 } else { 0 };
    let matrix_size = matrix.size() as u32;
    let total_size = matrix_size + margin * 2;

    let module_size = size_px / total_size;

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
        size_px,
        size_px,
        Rgba([255, 255, 255, 255]),
    );

    for j in 0..matrix_size {
        for i in 0..matrix_size {
            if is_dark(&matrix.get(i as usize, j as usize)) {
                let x_offset = margin as u32 * module_size;
                let y_offset = margin as u32 * module_size;

                for dy in 0..module_size {
                    for dx in 0..module_size {
                        let px = x_offset + i * module_size + dx;
                        let py = y_offset + j * module_size + dy;
                        img.put_pixel(px, py, Rgba([0, 0, 0, 255]));
                    }
                }
            }
        }
    }

    img
}
