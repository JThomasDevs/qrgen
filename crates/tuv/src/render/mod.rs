//! Rendering — SVG, PNG, unicode, and luma image output.

pub mod pixels;
pub mod png;
pub mod svg;
pub mod unicode;

use std::cmp::max;

use ::image::{ImageBuffer, Luma, Rgba};

use crate::types::Color;

pub use crate::types::QRGenError;

/// Builder for rendering a QR code to various output formats.
pub struct Renderer<'a> {
    content: &'a [Color],
    modules_count: u32,
    quiet_zone_modules: u32,
    module_size: (u32, u32),
    unicode_scale: u32,
    dark_svg: String,
    light_svg: String,
    has_quiet_zone: bool,
    transparent_background: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(content: &'a [Color], modules_count: usize, is_micro: bool) -> Self {
        assert_eq!(modules_count * modules_count, content.len());
        Self {
            content,
            modules_count: modules_count as u32,
            quiet_zone_modules: if is_micro { 2 } else { 4 },
            module_size: (8, 8),
            unicode_scale: 2,
            dark_svg: "#000000".to_string(),
            light_svg: "#ffffff".to_string(),
            has_quiet_zone: true,
            transparent_background: false,
        }
    }

    pub fn dark_color(&mut self, color: impl Into<String>) -> &mut Self {
        self.dark_svg = color.into();
        self
    }

    pub fn light_color(&mut self, color: impl Into<String>) -> &mut Self {
        self.light_svg = color.into();
        self
    }

    pub fn quiet_zone(&mut self, has_quiet_zone: bool) -> &mut Self {
        self.has_quiet_zone = has_quiet_zone;
        self
    }

    pub fn transparent_background(&mut self, transparent: bool) -> &mut Self {
        self.transparent_background = transparent;
        self
    }

    pub fn module_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.module_size = (max(width, 1), max(height, 1));
        self
    }

    /// Terminal columns per QR module for [`Self::build_unicode`] (default `2`).
    ///
    /// Each module is rendered `scale` columns wide; height stays one terminal line per
    /// QR row via Dense1x2 half-block characters (2 pixel rows per module row).
    pub fn unicode_scale(&mut self, scale: u32) -> &mut Self {
        self.unicode_scale = max(scale, 1);
        self
    }

    pub fn min_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        let border_modules = if self.has_quiet_zone {
            2 * self.quiet_zone_modules
        } else {
            0
        };
        let width_in_modules = self.modules_count + border_modules;
        let unit_width = (width + width_in_modules - 1) / width_in_modules;
        let unit_height = (height + width_in_modules - 1) / width_in_modules;
        self.module_dimensions(unit_width, unit_height)
    }

    pub fn build_svg(&self) -> String {
        svg::render_colors(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            &self.dark_svg,
            &self.light_svg,
            self.module_size,
            self.transparent_background,
        )
    }

    pub fn build_png(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        png::render_colors(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            &self.dark_svg,
            &self.light_svg,
            self.module_size,
            self.transparent_background,
        )
    }

    pub fn build_unicode(&self) -> String {
        let scale = self.unicode_scale.max(1);
        // Dense1x2: scale affects width only; 2 pixel rows per QR row → 1 terminal line.
        unicode::render_dense1x2(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            (scale, 2),
        )
    }

    pub fn build_image_luma(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        pixels::render_luma(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            self.module_size,
        )
    }
}
