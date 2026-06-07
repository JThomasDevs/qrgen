//! Top-level orchestration of the encode pipeline.
//!
//! ```rust,ignore
//! use tuv::QRCode;
//!
//! let qr = QRCode::from("Hello, world!").generate()?;
//! let svg = qr.to_svg(true);
//! let png = qr.to_png(300, true);
//! ```

use std::borrow::Cow;
use std::ops::Index;

use crate::bits::Bits;
use crate::error_correction::{
    interleave, max_allowed_errors, split_into_blocks, total_data_codewords, ECCLevel,
};
use crate::matrix::{Module, QRMatrix};
use crate::render;
use crate::types::{Color, Version};

pub use crate::types::QRGenError;

/// Fluent builder for [`QRCode`].
#[derive(Debug)]
pub struct QRCodeBuilder<'a> {
    input: BuilderInput<'a>,
    ecc: Option<ECCLevel>,
    version: Option<Version>,
    mask_id: Option<u8>,
    micro: bool,
}

#[derive(Debug)]
enum BuilderInput<'a> {
    Text(&'a str),
    Bytes(&'a [u8]),
    PreEncoded(Bits),
}

#[derive(Debug, Clone)]
pub struct QRCode {
    version: Version,
    matrix: QRMatrix,
    colors: Vec<Color>,
    mask_id: u8,
    ecc: ECCLevel,
}

pub(crate) fn build_qrcode(
    version: Version,
    matrix: QRMatrix,
    mask_id: u8,
    ecc: ECCLevel,
) -> QRCode {
    let colors = colors_from_matrix(&matrix);
    QRCode {
        version,
        matrix,
        colors,
        mask_id,
        ecc,
    }
}

fn colors_from_matrix(matrix: &QRMatrix) -> Vec<Color> {
    let w = matrix.size;
    let mut out = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            out.push(if matrix.get(x, y).is_dark() {
                Color::Dark
            } else {
                Color::Light
            });
        }
    }
    out
}

fn input_bytes<'a>(input: &BuilderInput<'a>) -> Cow<'a, [u8]> {
    match input {
        BuilderInput::Text(s) => Cow::Borrowed(s.as_bytes()),
        BuilderInput::Bytes(b) => Cow::Borrowed(b),
        BuilderInput::PreEncoded(_) => Cow::Borrowed(&[]),
    }
}

/// Raw data + EC codewords in matrix interleave order (before zigzag placement).
fn interleaved_codewords(
    input: &[u8],
    ecc: ECCLevel,
    version: u8,
) -> Result<Vec<u8>, QRGenError> {
    let mut bits = Bits::new(Version::Normal(i16::from(version)));
    bits.push_optimal_data(input)?;
    interleaved_from_bits(bits, ecc, version)
}

fn interleaved_from_bits(
    mut bits: Bits,
    ecc: ECCLevel,
    version: u8,
) -> Result<Vec<u8>, QRGenError> {
    bits.push_terminator(ecc)?;
    let data_bytes = bits.into_bytes();
    let total = total_data_codewords(version, ecc);
    debug_assert_eq!(
        data_bytes.len(),
        total,
        "padded data length {} != capacity {}",
        data_bytes.len(),
        total
    );

    let blocks = split_into_blocks(&data_bytes, version, ecc);
    Ok(interleave(&blocks))
}

/// Build matrix with function patterns and placed data/EC bits, **before** mask selection.
fn matrix_before_mask(input: &[u8], ecc: ECCLevel, version: u8) -> Result<QRMatrix, QRGenError> {
    let interleaved = interleaved_codewords(input, ecc, version)?;
    let mut matrix = QRMatrix::new(version);
    matrix.place_function_patterns();
    let bit_stream = bytes_to_bits_msb_first(&interleaved);
    crate::matrix::data_placement::place_data(&mut matrix, &bit_stream);
    Ok(matrix)
}

fn encode_normal(
    input: &[u8],
    ecc: ECCLevel,
    version: u8,
    mask_id: Option<u8>,
) -> Result<QRCode, QRGenError> {
    let mut matrix = matrix_before_mask(input, ecc, version)?;
    let mask_id = crate::matrix::masking::apply_mask_selection(&mut matrix, ecc, mask_id);

    if version >= 7 {
        crate::matrix::version_info::place_version_info(&mut matrix, version);
    }

    let colors = colors_from_matrix(&matrix);

    Ok(QRCode {
        version: Version::Normal(i16::from(version)),
        matrix,
        colors,
        mask_id,
        ecc,
    })
}

fn encode(
    input: &[u8],
    ecc: ECCLevel,
    version: Version,
    mask_id: Option<u8>,
) -> Result<QRCode, QRGenError> {
    match version {
        Version::Normal(v) if (1..=40).contains(&v) => {
            encode_normal(input, ecc, v as u8, mask_id)
        }
        Version::Micro(_) => crate::micro::encode(input, ecc, version, mask_id),
        Version::Normal(v) => Err(QRGenError::InvalidVersion {
            version: v.max(0) as u8,
        }),
    }
}

impl QRCode {
    /// Start building a QR code from a UTF-8 string.
    pub fn from(input: &str) -> QRCodeBuilder<'_> {
        QRCodeBuilder {
            input: BuilderInput::Text(input),
            ecc: None,
            version: None,
            mask_id: None,
            micro: false,
        }
    }

    /// Start building from pre-encoded [`Bits`].
    pub fn from_bits(bits: Bits) -> QRCodeBuilder<'static> {
        QRCodeBuilder {
            input: BuilderInput::PreEncoded(bits),
            ecc: None,
            version: None,
            mask_id: None,
            micro: false,
        }
    }

    /// Start building a QR code from raw bytes (byte mode when optimal).
    pub fn from_bytes(input: &[u8]) -> QRCodeBuilder<'_> {
        QRCodeBuilder {
            input: BuilderInput::Bytes(input),
            ecc: None,
            version: None,
            mask_id: None,
            micro: false,
        }
    }

    /// Encode `input` into a QR code.
    #[deprecated(note = "use QRCode::from(input).with_ecc(ecc).with_version(version).generate()?")]
    pub fn new(input: &str, ecc: Option<ECCLevel>, version: Option<u8>) -> Result<Self, QRGenError> {
        let mut builder = QRCode::from(input);
        if let Some(ecc) = ecc {
            builder = builder.with_ecc(ecc);
        }
        if let Some(version) = version {
            builder = builder.with_version_number(version);
        }
        builder.generate()
    }

    pub fn to_svg(&self, quiet_zone: bool) -> String {
        self.render()
            .quiet_zone(quiet_zone)
            .module_dimensions(1, 1)
            .build_svg()
    }

    pub fn to_png(&self, size_px: u32, quiet_zone: bool) -> ::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>> {
        self.render()
            .quiet_zone(quiet_zone)
            .min_dimensions(size_px, size_px)
            .build_png()
    }

    /// Configure rendering (colors, dimensions, string/unicode/image output).
    pub fn render(&self) -> render::Renderer<'_> {
        render::Renderer::new(&self.colors, self.width(), self.version.is_micro())
    }

    /// Matrix width in modules (excluding any quiet zone in render output).
    #[inline]
    pub fn width(&self) -> usize {
        self.matrix.size
    }

    /// Dark or light after masking and format info.
    #[inline]
    pub fn module_is_dark(&self, col: usize, row: usize) -> bool {
        self.matrix.get(col, row).is_dark()
    }

    /// Color at `(x, y)`.
    #[inline]
    pub fn color_at(&self, x: usize, y: usize) -> Color {
        self.colors[y * self.width() + x]
    }

    /// Mask pattern id `0..=7` chosen during encoding.
    #[inline]
    pub fn mask_id(&self) -> u8 {
        self.mask_id
    }

    /// QR version used for this code.
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    /// Error correction level applied during encoding.
    #[inline]
    pub fn ecc(&self) -> ECCLevel {
        self.ecc
    }

    /// Alias for [`ecc`](Self::ecc).
    #[inline]
    pub fn error_correction_level(&self) -> ECCLevel {
        self.ecc
    }

    /// Maximum erratic modules that can be corrected before data is lost.
    pub fn max_allowed_errors(&self) -> usize {
        if let Some(v) = self.version.normal_number() {
            max_allowed_errors(v, self.ecc)
        } else if let Some(v) = self.version.micro_number() {
            crate::micro::max_allowed_errors(v, self.ecc)
        } else {
            0
        }
    }

    /// Whether `(x, y)` is a functional (non-data) module.
    pub fn is_functional(&self, x: usize, y: usize) -> bool {
        !self.matrix.get(x, y).is_data()
    }

    /// Human-readable matrix for debugging.
    pub fn to_debug_str(&self, on_char: char, off_char: char) -> String {
        self.render()
            .quiet_zone(false)
            .module_dimensions(1, 1)
            .dark_char(on_char)
            .light_char(off_char)
            .build_string()
    }

    pub fn to_vec(&self) -> Vec<bool> {
        self.colors.iter().map(|c| *c == Color::Dark).collect()
    }

    pub fn into_vec(self) -> Vec<bool> {
        self.colors.into_iter().map(|c| c == Color::Dark).collect()
    }

    pub fn to_colors(&self) -> Vec<Color> {
        self.colors.clone()
    }

    pub fn into_colors(self) -> Vec<Color> {
        self.colors
    }

    #[cfg(debug_assertions)]
    pub fn debug_matrix(&self) -> String {
        let mut s = format!("size: {}\n", self.matrix.size);
        for j in 0..self.matrix.size {
            for i in 0..self.matrix.size {
                let v = self.matrix.data_at(i, j);
                s.push(match v {
                    Some(true) => '#',
                    Some(false) => '.',
                    None => '?',
                });
            }
            s.push('\n');
        }
        s
    }

    #[cfg(debug_assertions)]
    pub fn get_module(&self, i: usize, j: usize) -> Module {
        self.matrix.get(i, j)
    }

    #[cfg(debug_assertions)]
    pub fn size(&self) -> usize {
        self.matrix.size
    }

    #[cfg(debug_assertions)]
    pub fn debug_full_matrix(&self) -> String {
        use std::collections::HashMap;
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for j in 0..self.matrix.size {
            for i in 0..self.matrix.size {
                let name = match self.matrix.get(i, j) {
                    Module::Data(false) => "Data(false)",
                    Module::Data(true) => "Data(true)",
                    Module::Finder(_) => "Finder",
                    Module::Separator => "Separator",
                    Module::Alignment(_) => "Alignment",
                    Module::Timing(_) => "Timing",
                    Module::FormatInfo(_) => "FormatInfo",
                    Module::VersionInfo(_) => "VersionInfo",
                };
                *counts.entry(name).or_insert(0) += 1;
            }
        }

        let mut s = format!("size: {}\n", self.matrix.size);
        s.push_str("Module counts:\n");
        for (k, v) in &counts {
            s.push_str(&format!("  {}: {}\n", k, v));
        }
        s.push_str("\nMatrix:\n");
        for j in 0..self.matrix.size {
            for i in 0..self.matrix.size {
                let ch = match self.matrix.get(i, j) {
                    Module::Data(false) => '.',
                    Module::Data(true) => '#',
                    Module::Finder(_) => 'F',
                    Module::Separator => 's',
                    Module::Alignment(_) => 'A',
                    Module::Timing(_) => 'T',
                    Module::FormatInfo(_) => 'f',
                    Module::VersionInfo(_) => 'v',
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }
}

impl Index<(usize, usize)> for QRCode {
    type Output = Color;

    fn index(&self, (x, y): (usize, usize)) -> &Color {
        &self.colors[y * self.width() + x]
    }
}

impl<'a> QRCodeBuilder<'a> {
    /// Set the error correction level explicitly.
    ///
    /// When omitted, ECC is auto-selected: together with version when both are
    /// unset (smallest symbol), or the lowest level that fits when version is fixed.
    pub fn with_ecc(self, ecc: ECCLevel) -> Self {
        Self { ecc: Some(ecc), ..self }
    }

    /// Set a specific QR or Micro QR version.
    pub fn with_version(self, version: Version) -> Self {
        Self {
            version: Some(version),
            ..self
        }
    }

    /// Set a Normal QR version (1–40). Convenience wrapper over [`with_version`](Self::with_version).
    pub fn with_version_number(self, version: u8) -> Self {
        self.with_version(Version::Normal(i16::from(version)))
    }

    /// Set a specific mask pattern (0-7). When omitted, the lowest-penalty mask is chosen.
    pub fn with_mask_id(self, mask_id: u8) -> Self {
        Self {
            mask_id: Some(mask_id),
            ..self
        }
    }

    /// Auto-select the smallest **Micro** QR version (v1–4) when version is omitted.
    ///
    /// Without this flag, version auto-selection considers Normal QR only.
    pub fn with_micro(self) -> Self {
        Self { micro: true, ..self }
    }

    /// Encode the QR code with the configured options.
    pub fn generate(self) -> Result<QRCode, QRGenError> {
        if let Some(mask_id) = self.mask_id {
            if mask_id > 7 {
                return Err(QRGenError::InvalidMaskId { mask_id });
            }
        }

        if let BuilderInput::PreEncoded(bits) = self.input {
            let version = match self.version {
                Some(v) => validate_version(v)?,
                None => validate_version(bits.version())?,
            };
            let ecc = match self.ecc {
                Some(ecc) => ecc,
                None => lowest_ecc_for_bits(&bits, version)
                    .ok_or(QRGenError::InputTooLong { ecc: ECCLevel::L })?,
            };
            return encode_from_bits(bits, ecc, version, self.mask_id);
        }

        let input = input_bytes(&self.input);

        let (version, ecc) = match (self.version, self.ecc) {
            (Some(v), Some(ecc)) => (validate_version(v)?, ecc),
            (Some(v), None) => {
                let version = validate_version(v)?;
                let ecc = lowest_ecc_for_version(&input, version)
                    .ok_or(QRGenError::InputTooLong { ecc: ECCLevel::L })?;
                (version, ecc)
            }
            (None, Some(ecc)) => {
                let version = if self.micro {
                    smallest_micro_version(&input, ecc)
                } else {
                    smallest_version(&input, ecc)
                }
                .ok_or(QRGenError::InputTooLong { ecc })?;
                (version, ecc)
            }
            (None, None) => smallest_version_and_ecc(&input, self.micro)
                .ok_or(QRGenError::InputTooLong { ecc: ECCLevel::L })?,
        };

        encode(&input, ecc, version, self.mask_id)
    }
}

fn encode_from_bits(
    bits: Bits,
    ecc: ECCLevel,
    version: Version,
    mask_id: Option<u8>,
) -> Result<QRCode, QRGenError> {
    match version {
        Version::Normal(v) if (1..=40).contains(&v) => {
            let interleaved = interleaved_from_bits(bits, ecc, v as u8)?;
            let mut matrix = QRMatrix::new(v as u8);
            matrix.place_function_patterns();
            let bit_stream = bytes_to_bits_msb_first(&interleaved);
            crate::matrix::data_placement::place_data(&mut matrix, &bit_stream);
            let mask_id = crate::matrix::masking::apply_mask_selection(&mut matrix, ecc, mask_id);
            if v >= 7 {
                crate::matrix::version_info::place_version_info(&mut matrix, v as u8);
            }
            let colors = colors_from_matrix(&matrix);
            Ok(QRCode {
                version,
                matrix,
                colors,
                mask_id,
                ecc,
            })
        }
        Version::Micro(_) => crate::micro::encode_from_bits(bits, ecc, version, mask_id),
        Version::Normal(v) => Err(QRGenError::InvalidVersion {
            version: v.max(0) as u8,
        }),
    }
}

fn validate_version(v: Version) -> Result<Version, QRGenError> {
    match v {
        Version::Normal(n) if (1..=40).contains(&n) => Ok(v),
        Version::Micro(n) if (1..=4).contains(&n) => Ok(v),
        Version::Normal(n) | Version::Micro(n) => Err(QRGenError::InvalidVersion {
            version: n.max(0) as u8,
        }),
    }
}

// ---- Helpers ----

const ECC_SEARCH_ORDER: [ECCLevel; 4] =
    [ECCLevel::L, ECCLevel::M, ECCLevel::Q, ECCLevel::H];

fn bytes_to_bits_msb_first(bytes: &[u8]) -> Vec<bool> {
    let mut out = Vec::with_capacity(bytes.len() * 8);
    for &b in bytes {
        for i in (0..8).rev() {
            out.push(((b >> i) & 1) != 0);
        }
    }
    out
}

fn smallest_version(input: &[u8], ecc: ECCLevel) -> Option<Version> {
    for v in 1u8..=40 {
        let version = Version::Normal(i16::from(v));
        if version_fits(input, ecc, version) {
            return Some(version);
        }
    }
    None
}

fn smallest_micro_version(input: &[u8], ecc: ECCLevel) -> Option<Version> {
    for v in 1u8..=4 {
        let version = Version::Micro(i16::from(v));
        if version_fits(input, ecc, version) {
            return Some(version);
        }
    }
    None
}

fn version_fits(input: &[u8], ecc: ECCLevel, version: Version) -> bool {
    let mut bits = Bits::new(version);
    bits.push_optimal_data(input).is_ok() && bits.push_terminator(ecc).is_ok()
}

fn bits_fits(bits: &Bits, ecc: ECCLevel) -> bool {
    let mut bits = bits.clone();
    bits.push_terminator(ecc).is_ok()
}

fn lowest_ecc_for_version(input: &[u8], version: Version) -> Option<ECCLevel> {
    ECC_SEARCH_ORDER
        .into_iter()
        .find(|&ecc| version_fits(input, ecc, version))
}

fn lowest_ecc_for_bits(bits: &Bits, _version: Version) -> Option<ECCLevel> {
    ECC_SEARCH_ORDER
        .into_iter()
        .find(|&ecc| bits_fits(bits, ecc))
}

fn smallest_version_and_ecc(input: &[u8], micro: bool) -> Option<(Version, ECCLevel)> {
    if micro {
        for v in 1u8..=4 {
            let version = Version::Micro(i16::from(v));
            for ecc in ECC_SEARCH_ORDER {
                if version_fits(input, ecc, version) {
                    return Some((version, ecc));
                }
            }
        }
    } else {
        for v in 1u8..=40 {
            let version = Version::Normal(i16::from(v));
            for ecc in ECC_SEARCH_ORDER {
                if version_fits(input, ecc, version) {
                    return Some((version, ecc));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod mask_score_debug {
    use super::*;

    #[test]
    fn interleaved_matches_qrcode_for_v1_digit_1() {
        use qrcode::bits::Bits;
        use qrcode::ec;
        use qrcode::types::{EcLevel, Version as RefVersion};

        let ours = interleaved_codewords(b"1", ECCLevel::M, 1).expect("ours");

        let mut bits = Bits::new(RefVersion::Normal(1));
        bits.push_optimal_data(b"1").unwrap();
        bits.push_terminator(EcLevel::M).unwrap();
        let raw = bits.into_bytes();
        let (data, eccv) = ec::construct_codewords(&raw, RefVersion::Normal(1), EcLevel::M).unwrap();
        let mut ref_interleaved = Vec::new();
        ref_interleaved.extend_from_slice(&data);
        ref_interleaved.extend_from_slice(&eccv);

        assert_eq!(ours, ref_interleaved, "interleaved codewords differ from qrcode");
    }
}

#[cfg(test)]
mod matrix_api_tests {
    use super::*;
    use crate::types::Color;

    #[test]
    fn from_bytes_non_utf8() {
        let bytes = [0xFF, 0xFE, 0x00];
        let qr = QRCode::from_bytes(&bytes)
            .with_ecc(ECCLevel::L)
            .with_version_number(1)
            .generate()
            .expect("non-UTF-8 bytes should encode in byte mode");
        assert_eq!(qr.version(), Version::Normal(1));
        assert_eq!(qr.width(), 21);
    }

    #[test]
    fn index_and_colors_match_module_is_dark() {
        let qr = QRCode::from("Hi")
            .with_ecc(ECCLevel::M)
            .with_version_number(1)
            .generate()
            .unwrap();
        for y in 0..qr.width() {
            for x in 0..qr.width() {
                assert_eq!(qr[(x, y)] == Color::Dark, qr.module_is_dark(x, y));
            }
        }
    }

    #[test]
    fn max_allowed_errors_v1_m() {
        let qr = QRCode::from("1")
            .with_ecc(ECCLevel::M)
            .with_version_number(1)
            .generate()
            .unwrap();
        assert_eq!(qr.max_allowed_errors(), 4);
    }

    #[test]
    fn is_functional_corners() {
        let qr = QRCode::from("1")
            .with_ecc(ECCLevel::M)
            .with_version_number(1)
            .generate()
            .unwrap();
        assert!(qr.is_functional(0, 0));
        assert!(!qr.is_functional(10, 10));
    }

    #[test]
    fn to_debug_str_uses_chars() {
        let qr = QRCode::from("1")
            .with_ecc(ECCLevel::M)
            .with_version_number(1)
            .generate()
            .unwrap();
        let s = qr.to_debug_str('#', '.');
        assert!(s.contains('#'));
        assert!(s.contains('.'));
    }

    #[test]
    fn auto_selects_lowest_ecc_for_short_payload() {
        let qr = QRCode::from("Hello").generate().unwrap();
        assert_eq!(qr.version(), Version::Normal(1));
        assert_eq!(qr.ecc(), ECCLevel::L);
    }

    #[test]
    fn auto_ecc_with_fixed_version_picks_lowest() {
        let qr = QRCode::from("1")
            .with_version_number(1)
            .generate()
            .unwrap();
        assert_eq!(qr.ecc(), ECCLevel::L);
    }

    #[test]
    fn explicit_ecc_is_respected() {
        let qr = QRCode::from("Hello")
            .with_ecc(ECCLevel::M)
            .generate()
            .unwrap();
        assert_eq!(qr.ecc(), ECCLevel::M);
    }

    #[test]
    fn auto_micro_selects_lowest_ecc() {
        let qr = QRCode::from("1")
            .with_micro()
            .generate()
            .unwrap();
        assert_eq!(qr.version(), Version::Micro(1));
        assert_eq!(qr.ecc(), ECCLevel::L);
    }
}
