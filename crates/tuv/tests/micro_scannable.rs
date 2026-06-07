//! Micro QR structure and render tests (no external encoder dependency).
use tuv::{ECCLevel, Module, QRCode, Version};

#[test]
fn micro_v1_123_width_and_format_info() {
    let qr = QRCode::from("123")
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(1))
        .generate()
        .unwrap();

    assert_eq!(qr.version(), Version::Micro(1));
    assert_eq!(qr.width(), 11);

    let format_modules = (0..qr.width())
        .flat_map(|y| (0..qr.width()).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(qr.get_module(x, y), Module::FormatInfo(_)))
        .count();
    assert_eq!(format_modules, 15, "Micro QR exposes 15 format-info modules");

    let finder_modules = (0..qr.width())
        .flat_map(|y| (0..qr.width()).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(qr.get_module(x, y), Module::Finder(_)))
        .count();
    assert!(finder_modules > 0, "finder pattern should be present");
}

#[test]
fn micro_v1_png_uses_two_module_quiet_zone() {
    let qr = QRCode::from("123")
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(1))
        .generate()
        .unwrap();

    let png = qr.to_png(256, true);
    // 11 data modules + 2-module quiet zone on each side => 15 modules total.
    let modules_per_side = qr.width() as u32 + 4;
    let module_px = png.width() / modules_per_side;
    assert!(module_px > 0, "expected positive module pixel size");
    assert_eq!(png.width(), modules_per_side * module_px);
    assert_eq!(png.width(), png.height());

    // Top-left quiet-zone pixel must be light (white margin).
    let top_left = png.get_pixel(module_px / 2, module_px / 2);
    assert_eq!(top_left.0, [255, 255, 255, 255]);

    // First dark module in the symbol (top-left finder) sits after the margin.
    let finder_px = png.get_pixel(module_px * 2 + module_px / 2, module_px * 2 + module_px / 2);
    assert_eq!(finder_px.0[0], 0, "finder should render as dark");
}

#[test]
fn micro_auto_123_selects_v1() {
    let qr = QRCode::from_bytes(b"123")
        .with_micro()
        .with_ecc(ECCLevel::L)
        .generate()
        .unwrap();
    assert_eq!(qr.version(), Version::Micro(1));
    assert_eq!(qr.width(), 11);
}
