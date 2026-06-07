//! PNG rendering from color buffers.

use image::{ImageBuffer, Rgba};
use crate::types::Color;

pub fn render_png(
    matrix: &crate::matrix::QRMatrix,
    size_px: u32,
    quiet_zone: bool,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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
    let mut renderer = super::Renderer::new(&colors, w, matrix.is_micro());
    renderer.quiet_zone(quiet_zone).min_dimensions(size_px, size_px);
    renderer.build_png()
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
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let dark = parse_hex_color(dark);
    let light = parse_hex_color(light);
    let qz = if has_quiet_zone {
        quiet_zone_modules
    } else {
        0
    };
    let (mw, mh) = module_size;
    let total_modules = modules_count + 2 * qz;
    let width_px = total_modules as u32 * mw;
    let height_px = total_modules as u32 * mh;

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = if transparent_background {
        ImageBuffer::from_pixel(width_px, height_px, Rgba([0, 0, 0, 0]))
    } else {
        ImageBuffer::from_pixel(width_px, height_px, light)
    };

    let mut i = 0usize;
    for y in 0..total_modules {
        for x in 0..total_modules {
            if qz <= x && x < modules_count + qz && qz <= y && y < modules_count + qz {
                if content[i] == Color::Dark {
                    for dy in 0..mh {
                        for dx in 0..mw {
                            img.put_pixel(x as u32 * mw + dx, y as u32 * mh + dy, dark);
                        }
                    }
                } else if !transparent_background {
                    for dy in 0..mh {
                        for dx in 0..mw {
                            img.put_pixel(x as u32 * mw + dx, y as u32 * mh + dy, light);
                        }
                    }
                }
                i += 1;
            }
        }
    }

    img
}

fn parse_hex_color(hex: &str) -> Rgba<u8> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Rgba([0, 0, 0, 255]);
    }
    let parse = |start: usize| u8::from_str_radix(&hex[start..start + 2], 16).unwrap_or(0);
    Rgba([parse(0), parse(2), parse(4), 255])
}
