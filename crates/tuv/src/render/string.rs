//! Character-based terminal rendering.

use crate::types::Color;

pub fn render_chars(
    content: &[Color],
    modules_count: usize,
    quiet_zone_modules: usize,
    has_quiet_zone: bool,
    dark: char,
    light: char,
    module_size: (u32, u32),
) -> String {
    let qz = if has_quiet_zone {
        quiet_zone_modules
    } else {
        0
    };
    let (mw, _mh) = module_size;
    let total_modules = modules_count + 2 * qz;
    let mw = mw as usize;

    let mut result = String::new();
    let mut i = 0usize;
    for y in 0..total_modules {
        if y > 0 {
            result.push('\n');
        }
        for x in 0..total_modules {
            let ch = if qz <= x && x < modules_count + qz && qz <= y && y < modules_count + qz {
                let c = content[i];
                i += 1;
                c.select(dark, light)
            } else {
                light
            };
            for _ in 0..mw {
                result.push(ch);
            }
        }
    }
    result
}
