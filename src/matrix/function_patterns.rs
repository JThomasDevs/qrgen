//! Function patterns — finder markers, alignment patterns, timing patterns.
//! 
//! Function patterns are the fixed structural elements of a QR matrix.
//! They are immune to masking (placed before masking, never XOR'd).
//! 
//! ## Finder Markers (3 total, always in corners)
//! 
//! Each is a 7×7 pattern with a distinctive "target" appearance:
//! - Outer 7×7 border: all dark
//! - Inner 5×5 border: all dark  
//! - Core 3×3: dark corners, light center (forming a ring)
//! - Plus 1-module white separator (all around)
//! 
//! ## Alignment Patterns (version ≥ 2)
//! 
//! Smaller 5×5 patterns used to help scanners correct for rotation.
//! Positioned according to a version-specific alignment grid.
//! 
//! ## Timing Patterns
//! 
//! Alternating dark/light 1×N strips that help scanners sync their clock.
//! Row 6 (0-indexed) connecting finders horizontally, column 6 vertically.

// QRMatrix and Module are defined in this file — no need to import from super

/// Module type — identifies what's stored at each matrix position.
///
/// Every variant that actually renders carries its dark/light value so that
/// rendering, masking and scoring can all consult a single source of truth.
/// Function-pattern variants are immune to data masking by construction —
/// `apply_mask` only touches `Module::Data`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    /// Free data cell. `true` = dark.
    Data(bool),
    /// Cell inside a 7×7 finder marker. `true` = dark.
    Finder(bool),
    /// 1-module-wide white border around each finder. Always light.
    Separator,
    /// Cell inside a 5×5 alignment pattern. `true` = dark.
    Alignment(bool),
    /// Cell on a timing strip. `true` = dark.
    Timing(bool),
    /// Reserved/encoded format-info bit. `true` = dark.
    FormatInfo(bool),
    /// Reserved/encoded version-info bit. `true` = dark.
    VersionInfo(bool),
}

impl Module {
    /// Returns the rendered dark/light value for this module.
    pub fn is_dark(&self) -> bool {
        match self {
            Module::Data(v)
            | Module::Finder(v)
            | Module::Alignment(v)
            | Module::Timing(v)
            | Module::FormatInfo(v)
            | Module::VersionInfo(v) => *v,
            Module::Separator => false,
        }
    }

    /// True if this cell is available for data placement / data masking.
    pub fn is_data(&self) -> bool {
        matches!(self, Module::Data(_))
    }
}

/// The QR module matrix.
#[derive(Debug, Clone)]
pub struct QRMatrix {
    pub version: u8,
    pub size: usize,
    pub modules: Vec<Vec<Module>>,
}

impl QRMatrix {
    /// Create a new empty matrix for the given version.
    /// All modules start as Data(false).
    pub fn new(version: u8) -> Self {
        let size = (version as usize) * 4 + 17;
        let modules = vec![vec![Module::Data(false); size]; size];
        Self { version, size, modules }
    }

    /// Set a module at (col, row).
    pub fn set(&mut self, i: usize, j: usize, module: Module) {
        if i >= self.size || j >= self.size {
            return;
        }
        self.modules[j][i] = module;
    }

    /// Get the module at (i, j).
    pub fn get(&self, i: usize, j: usize) -> Module {
        if i >= self.size || j >= self.size {
            return Module::Data(false);
        }
        self.modules[j][i]
    }

    /// Get the boolean value at (i, j) for data modules.
    pub fn data_at(&self, i: usize, j: usize) -> Option<bool> {
        match self.get(i, j) {
            Module::Data(v) => Some(v),
            _ => None,
        }
    }

    /// Get size (modules per side).
    pub fn size(&self) -> usize {
        self.size
    }

    /// Place all function patterns for this version, plus reserve the
    /// format-info (and, for v ≥ 7, version-info) areas so that data
    /// placement skips over them.
    pub fn place_function_patterns(&mut self) {
        self.place_finder_marker(0, 0);
        self.place_finder_marker(self.size - 7, 0);
        self.place_finder_marker(0, self.size - 7);

        self.place_separators();
        self.place_timing_patterns();

        if self.version >= 2 {
            self.place_alignment_patterns();
        }

        self.reserve_format_info_area();

        if self.version >= 7 {
            self.reserve_version_info_area();
        }
    }

    /// Mark only the cells that will receive format information (see
    /// `format_info::all_format_info_positions`) so data placement skips them.
    fn reserve_format_info_area(&mut self) {
        for (c, r) in crate::matrix::format_info::all_format_info_positions(self.size) {
            self.set(c, r, Module::FormatInfo(false));
        }
        let s = self.size;
        // Same cell as `qrcode` `Canvas::draw_format_info_patterns_with_number` dark module.
        self.set(8, s - 8, Module::FormatInfo(true));
    }

    /// For v ≥ 7, reserve the two 3×6 version-info blocks.
    fn reserve_version_info_area(&mut self) {
        let s = self.size;
        for row in 0..6 {
            for col in 0..3 {
                self.set(s - 11 + col, row, Module::VersionInfo(false));
                self.set(row, s - 11 + col, Module::VersionInfo(false));
            }
        }
    }

    fn place_finder_marker(&mut self, ox: usize, oy: usize) {
        for dy in 0..7 {
            for dx in 0..7 {
                self.set(ox + dx, oy + dy, Module::Finder(FINDER_PATTERN[dy][dx]));
            }
        }
    }

    fn place_separators(&mut self) {
        let s = self.size;
        for i in 0..8 {
            self.set(i, 7, Module::Separator);
            self.set(7, i, Module::Separator);

            self.set(s - 8 + i, 7, Module::Separator);
            self.set(s - 8, i, Module::Separator);

            self.set(i, s - 8, Module::Separator);
            self.set(7, s - 8 + i, Module::Separator);
        }
    }

    /// Timing patterns are alternating dark/light strips on row 6 and column 6,
    /// running between the finder separators.
    fn place_timing_patterns(&mut self) {
        let s = self.size;
        for i in 8..=s - 9 {
            let dark = i % 2 == 0;
            self.set(i, 6, Module::Timing(dark));
            self.set(6, i, Module::Timing(dark));
        }
    }

    fn place_alignment_patterns(&mut self) {
        let positions = alignment_positions(self.version);
        for &(cx, cy) in &positions {
            // Skip any centre that overlaps a finder marker.
            if matches!(self.get(cx, cy), Module::Finder(_)) {
                continue;
            }
            let ox = cx - 2;
            let oy = cy - 2;
            for dy in 0..5 {
                for dx in 0..5 {
                    self.set(ox + dx, oy + dy, Module::Alignment(ALIGNMENT_PATTERN[dy][dx]));
                }
            }
        }
    }
}

/// Finder pattern — 7×7 grid. true=dark, false=white.
const FINDER_PATTERN: [[bool; 7]; 7] = [
    [true,  true,  true,  true,  true,  true,  true ],
    [true,  false, false, false, false, false, true ],
    [true,  false, true,  true,  true,  false, true ],
    [true,  false, true,  true,  true,  false, true ],
    [true,  false, true,  true,  true,  false, true ],
    [true,  false, false, false, false, false, true ],
    [true,  true,  true,  true,  true,  true,  true ],
];

/// Alignment pattern — 5×5 grid.
const ALIGNMENT_PATTERN: [[bool; 5]; 5] = [
    [true,  true,  true,  true,  true ],
    [true,  false, false, false, true ],
    [true,  false, true,  false, true ],
    [true,  false, false, false, true ],
    [true,  true,  true,  true,  true ],
];

/// Alignment pattern centre coordinates per ISO/IEC 18004 Annex E Table E.1.
/// Index into `ALIGNMENT_CENTRES` is `version - 1`. Empty for version 1.
const ALIGNMENT_CENTRES: [&[usize]; 40] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

/// All alignment-pattern centres for a given version, excluding the three
/// centres that would overlap a finder marker.
fn alignment_positions(version: u8) -> Vec<(usize, usize)> {
    let centres = ALIGNMENT_CENTRES[(version as usize) - 1];
    if centres.is_empty() {
        return Vec::new();
    }
    let last = *centres.last().unwrap();
    let mut out = Vec::new();
    for &cy in centres {
        for &cx in centres {
            // Skip the three centres that coincide with finder markers.
            let on_finder = (cx == 6 && cy == 6)
                || (cx == 6 && cy == last)
                || (cx == last && cy == 6);
            if !on_finder {
                out.push((cx, cy));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_size_version_1() {
        let m = QRMatrix::new(1);
        assert_eq!(m.size, 21);
    }

    #[test]
    fn test_matrix_size_version_10() {
        let m = QRMatrix::new(10);
        assert_eq!(m.size, 57);
    }

    #[test]
    fn test_finder_marker_at_topleft() {
        let mut mat = QRMatrix::new(1);
        mat.place_function_patterns();
        // Centre of top-left finder is dark.
        assert_eq!(mat.get(3, 3), Module::Finder(true));
        // Inside the white ring of the finder.
        assert_eq!(mat.get(1, 1), Module::Finder(false));
    }

    #[test]
    fn test_separator_placed() {
        let mut mat = QRMatrix::new(1);
        mat.place_function_patterns();
        assert_eq!(mat.get(0, 7), Module::Separator);
        assert_eq!(mat.get(7, 0), Module::Separator);
    }

    #[test]
    fn test_timing_alternates() {
        let mut mat = QRMatrix::new(2);
        mat.place_function_patterns();
        // Row 6, columns 8..=size-9 should alternate dark/light.
        assert_eq!(mat.get(8, 6), Module::Timing(true));
        assert_eq!(mat.get(9, 6), Module::Timing(false));
        assert_eq!(mat.get(10, 6), Module::Timing(true));
    }

    #[test]
    fn format_reserve_matches_placement_count() {
        let s = 25;
        let cells = crate::matrix::format_info::all_format_info_positions(s);
        assert_eq!(cells.len(), 30);
    }
}
