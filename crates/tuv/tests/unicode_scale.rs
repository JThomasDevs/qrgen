use tuv::QRCode;

#[test]
fn unicode_default_scale_doubles_module_width() {
    let qr = QRCode::from("Hello").generate().unwrap();
    let total_modules = qr.width() + 8;
    let text = qr.render().build_unicode();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0].chars().count(), total_modules * 2);
    assert_eq!(lines.len(), total_modules, "one terminal line per QR row");
}

#[test]
fn unicode_scale_sets_dimensions() {
    let qr = QRCode::from("Hello").generate().unwrap();
    let total_modules = qr.width() + 8;
    for scale in [1u32, 2, 3, 4] {
        let text = qr.render().unicode_scale(scale).build_unicode();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[0].chars().count(),
            total_modules * scale as usize,
            "scale {scale} width"
        );
        assert_eq!(
            lines.len(),
            total_modules,
            "scale {scale} height (1 line per QR row)"
        );
    }
}

#[test]
fn unicode_url_dimensions_at_default_scale() {
    let qr = QRCode::from("https://example.com").generate().unwrap();
    let total_modules = qr.width() + 8;
    let text = qr.render().build_unicode();
    let lines: Vec<&str> = text.lines().collect();
    let width = lines[0].chars().count();
    assert_eq!(width, total_modules * 2);
    assert_eq!(lines.len(), total_modules);
    assert!(width <= 80, "default scale should fit ~80-col terminals: {width}");
}
