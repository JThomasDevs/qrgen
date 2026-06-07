//! Unicode Dense1x2 half-block terminal rendering.

use crate::types::Color;

const CODEPAGE: [&str; 4] = [" ", "\u{2584}", "\u{2580}", "\u{2588}"];

pub fn render_dense1x2(
    content: &[Color],
    modules_count: usize,
    quiet_zone_modules: usize,
    has_quiet_zone: bool,
    module_size: (u32, u32),
) -> String {
    let qz = if has_quiet_zone {
        quiet_zone_modules
    } else {
        0
    };
    let mw = module_size.0.max(1) as usize;
    let mh = module_size.1.max(1) as usize;
    let total_modules = modules_count + 2 * qz;
    let row_width = total_modules * mw;
    let pixel_h = total_modules * mh;

    let mut canvas = vec![0u8; row_width * pixel_h];
    let mut i = 0usize;
    for y in 0..total_modules {
        for x in 0..total_modules {
            if qz <= x && x < modules_count + qz && qz <= y && y < modules_count + qz {
                if content[i] == Color::Dark {
                    for dy in 0..mh {
                        for dx in 0..mw {
                            canvas[(y * mh + dy) * row_width + x * mw + dx] = 1;
                        }
                    }
                }
                i += 1;
            }
        }
    }

    canvas
        .chunks(row_width)
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|rows| {
            if rows.len() == 2 {
                rows[0]
                    .iter()
                    .zip(rows[1])
                    .map(|(top, bot)| CODEPAGE[(*top as usize) * 2 + (*bot as usize)])
                    .collect::<String>()
            } else {
                rows[0]
                    .iter()
                    .map(|top| CODEPAGE[(*top as usize) * 2])
                    .collect::<String>()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
