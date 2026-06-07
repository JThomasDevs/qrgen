//! SVG rendering from color buffers.

use crate::types::Color;

pub fn render_svg(matrix: &crate::matrix::QRMatrix, quiet_zone: bool) -> String {
    let w = matrix.size();
    let mut colors = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            colors.push(if matrix.get(x, y).is_dark() {
                Color::Dark
            } else {
                Color::Light
            });
        }
    }
    let quiet_zone_modules = if matrix.is_micro() { 2 } else { 4 };
    render_colors(
        &colors,
        w,
        quiet_zone_modules,
        quiet_zone,
        "#000000",
        "#ffffff",
        (1, 1),
        false,
    )
}

pub fn render_colors(
    content: &[Color],
    modules_count: usize,
    quiet_zone_modules: usize,
    has_quiet_zone: bool,
    dark: &str,
    light: &str,
    module_size: (u32, u32),
    transparent_background: bool,
) -> String {
    let qz = if has_quiet_zone {
        quiet_zone_modules
    } else {
        0
    };
    let (mw, mh) = module_size;
    let total_modules = modules_count + 2 * qz;
    let width_px = total_modules as u32 * mw;
    let height_px = total_modules as u32 * mh;

    let mut path_data = String::new();
    let mut i = 0usize;
    for y in 0..total_modules {
        for x in 0..total_modules {
            if qz <= x && x < modules_count + qz && qz <= y && y < modules_count + qz {
                if content[i] == Color::Dark {
                    let px = x as f64 * mw as f64;
                    let py = y as f64 * mh as f64;
                    path_data.push_str(&format!(
                        "M {px} {py} h {mw} v {mh} h -{mw} Z "
                    ));
                }
                i += 1;
            }
        }
    }

    if transparent_background {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width_px}" height="{height_px}" viewBox="0 0 {width_px} {height_px}">
  <path d="{path}" fill="{dark}"/>
</svg>"#,
            width_px = width_px,
            height_px = height_px,
            dark = dark,
            path = path_data
        )
    } else {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width_px}" height="{height_px}" viewBox="0 0 {width_px} {height_px}">
  <rect width="100%" height="100%" fill="{light}"/>
  <path d="{path}" fill="{dark}"/>
</svg>"#,
            width_px = width_px,
            height_px = height_px,
            light = light,
            dark = dark,
            path = path_data
        )
    }
}
