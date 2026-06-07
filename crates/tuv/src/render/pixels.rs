//! Generic image pixel rendering.

use image::{ImageBuffer, Luma, Rgba};
use crate::types::Color;

pub fn render_luma(
    content: &[Color],
    modules_count: usize,
    quiet_zone_modules: usize,
    has_quiet_zone: bool,
    module_size: (u32, u32),
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    render_with_pixels(
        content,
        modules_count,
        quiet_zone_modules,
        has_quiet_zone,
        module_size,
        Luma([0u8]),
        Luma([255u8]),
    )
}

pub fn render_rgba(
    content: &[Color],
    modules_count: usize,
    quiet_zone_modules: usize,
    has_quiet_zone: bool,
    module_size: (u32, u32),
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    render_with_pixels(
        content,
        modules_count,
        quiet_zone_modules,
        has_quiet_zone,
        module_size,
        Rgba([0, 0, 0, 255]),
        Rgba([255, 255, 255, 255]),
    )
}

fn render_with_pixels<P>(
    content: &[Color],
    modules_count: usize,
    quiet_zone_modules: usize,
    has_quiet_zone: bool,
    module_size: (u32, u32),
    dark: P,
    light: P,
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: image::Pixel + 'static,
    P::Subpixel: image::Primitive,
{
    let qz = if has_quiet_zone {
        quiet_zone_modules
    } else {
        0
    };
    let (mw, mh) = module_size;
    let total_modules = modules_count + 2 * qz;
    let width_px = total_modules as u32 * mw;
    let height_px = total_modules as u32 * mh;

    let mut img = ImageBuffer::from_pixel(width_px, height_px, light);
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
                }
                i += 1;
            }
        }
    }
    img
}
