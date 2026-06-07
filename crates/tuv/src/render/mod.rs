//! Rendering — SVG, PNG, terminal string, and unicode output.

pub mod pixels;
pub mod png;
pub mod string;
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
    dark_svg: String,
    light_svg: String,
    dark_char: char,
    light_char: char,
    has_quiet_zone: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(content: &'a [Color], modules_count: usize, is_micro: bool) -> Self {
        assert_eq!(modules_count * modules_count, content.len());
        Self {
            content,
            modules_count: modules_count as u32,
            quiet_zone_modules: if is_micro { 2 } else { 4 },
            module_size: (8, 8),
            dark_svg: "#000000".to_string(),
            light_svg: "#ffffff".to_string(),
            dark_char: '\u{2588}',
            light_char: ' ',
            has_quiet_zone: true,
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

    pub fn dark_char(&mut self, ch: char) -> &mut Self {
        self.dark_char = ch;
        self
    }

    pub fn light_char(&mut self, ch: char) -> &mut Self {
        self.light_char = ch;
        self
    }

    pub fn quiet_zone(&mut self, has_quiet_zone: bool) -> &mut Self {
        self.has_quiet_zone = has_quiet_zone;
        self
    }

    pub fn module_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.module_size = (max(width, 1), max(height, 1));
        self
    }

    pub fn min_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        let qz = if self.has_quiet_zone { 2 } else { 0 } * self.quiet_zone_modules;
        let width_in_modules = self.modules_count + qz;
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
        )
    }

    pub fn build_png(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        png::render_colors(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            self.module_size,
        )
    }

    pub fn build_string(&self) -> String {
        string::render_chars(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            self.dark_char,
            self.light_char,
            self.module_size,
        )
    }

    pub fn build_unicode(&self) -> String {
        unicode::render_dense1x2(
            self.content,
            self.modules_count as usize,
            self.quiet_zone_modules as usize,
            self.has_quiet_zone,
            self.module_size,
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
