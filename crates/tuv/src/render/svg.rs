//! SVG rendering.
//!
//! SVG output is vector-based: infinitely scalable, small file size,
//! and easy to embed in documents or print.

use super::super::{QRMatrix, Module};

#[inline]
fn is_dark(module: &Module) -> bool {
    module.is_dark()
}

/// Render a QR matrix as an SVG string.
///
/// If `quiet_zone` is true, adds a 4-module white margin around the QR.
pub fn render_svg(matrix: &QRMatrix, quiet_zone: bool) -> String {
    let margin = if quiet_zone { 4 } else { 0 };
    let total_size = matrix.size() + margin * 2;

    let mut path_data = String::new();

    for j in 0..matrix.size() {
        for i in 0..matrix.size() {
            if is_dark(&matrix.get(i, j)) {
                let x = (i + margin) as f64;
                let y = (j + margin) as f64;
                path_data.push_str(&format!("M {:.0} {:.0} h 1 v 1 h -1 Z ", x, y));
            }
        }
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}">
  <rect width="100%" height="100%" fill="white"/>
  <path d="{path}" fill="black"/>
</svg>"#,
        size = total_size,
        path = path_data
    )
}
