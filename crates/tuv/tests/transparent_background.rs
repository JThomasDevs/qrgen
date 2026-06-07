use tuv::{Color, ECCLevel, QRCode, Version};

#[test]
fn png_transparent_light_and_quiet_zone_have_zero_alpha() {
    let qr = QRCode::from("Hi")
        .with_version(Version::Normal(1))
        .with_ecc(ECCLevel::L)
        .generate()
        .unwrap();

    let img = qr
        .render()
        .transparent_background(true)
        .quiet_zone(true)
        .module_dimensions(1, 1)
        .build_png();

    let qz = 4usize;
    let w = qr.width();
    for y in 0..img.height() {
        for x in 0..img.width() {
            let alpha = img.get_pixel(x, y).0[3];
            let in_matrix = qz <= x as usize
                && (x as usize) < w + qz
                && qz <= y as usize
                && (y as usize) < w + qz;
            if in_matrix {
                let mx = x as usize - qz;
                let my = y as usize - qz;
                if qr[(mx, my)] == Color::Dark {
                    assert_eq!(alpha, 255, "dark module at ({x}, {y})");
                } else {
                    assert_eq!(alpha, 0, "light module at ({x}, {y})");
                }
            } else {
                assert_eq!(alpha, 0, "quiet zone at ({x}, {y})");
            }
        }
    }
}

#[test]
fn png_opaque_default_fills_light_modules() {
    let qr = QRCode::from("Hi")
        .with_version(Version::Normal(1))
        .with_ecc(ECCLevel::L)
        .generate()
        .unwrap();

    let img = qr
        .render()
        .quiet_zone(false)
        .module_dimensions(1, 1)
        .build_png();

    let mut saw_light = false;
    for y in 0..qr.width() {
        for x in 0..qr.width() {
            let px = img.get_pixel(x as u32, y as u32);
            if qr[(x, y)] == Color::Light {
                assert_eq!(px.0[3], 255);
                saw_light = true;
            }
        }
    }
    assert!(saw_light);
}

#[test]
fn svg_transparent_omits_background_rect() {
    let qr = QRCode::from("Hi")
        .with_version(Version::Normal(1))
        .with_ecc(ECCLevel::L)
        .generate()
        .unwrap();

    let transparent = qr
        .render()
        .transparent_background(true)
        .module_dimensions(1, 1)
        .build_svg();
    assert!(
        !transparent.contains(r#"<rect width="100%""#),
        "transparent SVG must not include opaque background rect"
    );
    assert!(transparent.contains(r#"<path d=""#));

    let opaque = qr.render().module_dimensions(1, 1).build_svg();
    assert!(
        opaque.contains(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##),
        "opaque SVG must include background rect"
    );
}
