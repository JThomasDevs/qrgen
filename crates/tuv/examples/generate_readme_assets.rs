//! Generate PNG previews for README.md. Run from repo root:
//! `cargo run -p tuv --example generate_readme_assets`

use std::fs;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgba};

use tuv::bits::Bits;
use tuv::{ECCLevel, QRCode, Version};

const SIZE: u32 = 200;
/// Micro QR README previews target this total width/height (including extra border).
const MICRO_DISPLAY_PX: u32 = 220;
const MICRO_MODULE_MIN: u32 = 6;
const MICRO_EXTRA_BORDER_PX: u32 = 16;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = readme_assets_dir();
    fs::create_dir_all(&out_dir)?;

    // High-level builder
    save_png(
        &out_dir.join("builder-from-url.png"),
        QRCode::from("https://example.com").generate()?,
    )?;

    save_png(
        &out_dir.join("builder-from-bytes-raw.png"),
        QRCode::from_bytes(b"Hello")
            .with_ecc(ECCLevel::L)
            .generate()?,
    )?;

    save_png(
        &out_dir.join("builder-from-bytes-kanji.png"),
        QRCode::from_bytes(b"\x82\xa0")
            .with_ecc(ECCLevel::M)
            .generate()?,
    )?;

    save_micro_png(
        &out_dir.join("builder-micro-v1.png"),
        QRCode::from("123")
            .with_ecc(ECCLevel::L)
            .with_version(Version::Micro(1))
            .generate()?,
    )?;

    save_micro_png(
        &out_dir.join("builder-micro-auto.png"),
        QRCode::from_bytes(b"123")
            .with_micro()
            .with_ecc(ECCLevel::L)
            .generate()?,
    )?;

    let url_qr = QRCode::from("https://example.com").generate()?;
    save_png(&out_dir.join("builder-to-png.png"), url_qr)?;

    // Render builder
    let render_qr = QRCode::from("https://example.com").generate()?;
    save_render_png(
        &out_dir.join("render-colors.png"),
        &render_qr,
        "#800000",
        "#ffff80",
    )?;

    let unicode_qr = QRCode::from("https://example.com").generate()?;
    save_png(&out_dir.join("render-unicode.png"), unicode_qr)?;

    let luma_qr = QRCode::from("https://example.com").generate()?;
    save_png(&out_dir.join("render-luma.png"), luma_qr)?;

    save_render_transparent_png(&out_dir.join("render-transparent.png"), &render_qr)?;

    // Bits API
    let mut bits = Bits::new(Version::Normal(1));
    bits.push_eci_designator(9)?;
    bits.push_byte_data(b"\xa1\xa2\xa3")?;
    bits.push_terminator(ECCLevel::L)?;
    save_png(
        &out_dir.join("bits-eci.png"),
        QRCode::from_bits(bits).with_ecc(ECCLevel::L).generate()?,
    )?;

    // Matrix introspection
    save_png(
        &out_dir.join("matrix-hi.png"),
        QRCode::from("Hi")
            .with_version(Version::Normal(1))
            .with_ecc(ECCLevel::L)
            .generate()?,
    )?;

    // CLI equivalents
    save_png(
        &out_dir.join("cli-hello.png"),
        QRCode::from("Hello").generate()?,
    )?;

    let hello = QRCode::from("Hello").generate()?;
    save_png(&out_dir.join("cli-hello-png.png"), hello)?;

    save_micro_png(
        &out_dir.join("cli-micro.png"),
        QRCode::from("123")
            .with_version(Version::Micro(1))
            .with_ecc(ECCLevel::L)
            .generate()?,
    )?;

    save_micro_png(
        &out_dir.join("cli-micro-auto.png"),
        QRCode::from_bytes(b"123")
            .with_micro()
            .with_ecc(ECCLevel::L)
            .generate()?,
    )?;

    save_png(
        &out_dir.join("cli-unicode.png"),
        QRCode::from("Hello").generate()?,
    )?;

    let hello_colored = QRCode::from("Hello").generate()?;
    save_render_png(
        &out_dir.join("cli-colored.png"),
        &hello_colored,
        "#800000",
        "#ffff80",
    )?;

    println!("Wrote README assets to {}", out_dir.display());
    Ok(())
}

fn readme_assets_dir() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../assets/readme")
}

fn save_png(path: &Path, qr: QRCode) -> Result<(), Box<dyn std::error::Error>> {
    let img = qr.to_png(SIZE, true);
    img.save(path)?;
    println!("  {}", path.display());
    Ok(())
}

fn save_render_png(
    path: &Path,
    qr: &QRCode,
    dark: &str,
    light: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = qr
        .render()
        .dark_color(dark)
        .light_color(light)
        .quiet_zone(true)
        .min_dimensions(SIZE, SIZE)
        .build_png();
    img.save(path)?;
    println!("  {}", path.display());
    Ok(())
}

fn save_render_transparent_png(path: &Path, qr: &QRCode) -> Result<(), Box<dyn std::error::Error>> {
    let img = qr
        .render()
        .transparent_background(true)
        .quiet_zone(true)
        .min_dimensions(SIZE, SIZE)
        .build_png();
    img.save(path)?;
    println!("  {}", path.display());
    Ok(())
}

fn save_micro_png(path: &Path, qr: QRCode) -> Result<(), Box<dyn std::error::Error>> {
    let modules_per_side = qr.width() as u32 + 4;
    let border_total = 2 * MICRO_EXTRA_BORDER_PX;
    let content_target = MICRO_DISPLAY_PX.saturating_sub(border_total);
    let module_px = (content_target + modules_per_side - 1) / modules_per_side;
    let module_px = module_px.max(MICRO_MODULE_MIN);

    let img = qr
        .render()
        .quiet_zone(true)
        .module_dimensions(module_px, module_px)
        .build_png();
    let img = add_white_border(&img, MICRO_EXTRA_BORDER_PX);
    img.save(path)?;
    println!("  {} ({}x{}, {}px/module)", path.display(), img.width(), img.height(), module_px);
    Ok(())
}

fn add_white_border(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    border_px: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let out_w = img.width() + 2 * border_px;
    let out_h = img.height() + 2 * border_px;
    let mut out = ImageBuffer::from_pixel(out_w, out_h, Rgba([255, 255, 255, 255]));
    for y in 0..img.height() {
        for x in 0..img.width() {
            out.put_pixel(x + border_px, y + border_px, *img.get_pixel(x, y));
        }
    }
    out
}
