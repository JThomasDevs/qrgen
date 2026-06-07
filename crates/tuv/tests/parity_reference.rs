//! Cross-crate parity tests against qrcode 0.14.1 (dev-dependency).

use qrcode::bits::Bits as RefBits;
use qrcode::{Color as RefColor, EcLevel, QrCode, Version as RefVersion};
use tuv::bits::Bits;
use tuv::{ECCLevel, QRCode, Version};

fn ref_matrix(data: &[u8], version: RefVersion, ecc: EcLevel) -> Vec<bool> {
    let code = QrCode::with_version(data, version, ecc).unwrap();
    let w = code.width();
    let mut v = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            v.push(code[(x, y)] == RefColor::Dark);
        }
    }
    v
}

fn ours_matrix_bytes(data: &[u8], version: Version, ecc: ECCLevel) -> Vec<bool> {
    let qr = QRCode::from_bytes(data)
        .with_ecc(ecc)
        .with_version(version)
        .generate()
        .unwrap();
    let w = qr.width();
    let mut v = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            v.push(qr.module_is_dark(x, y));
        }
    }
    v
}

fn ours_from_bits(bits: Bits, ecc: ECCLevel) -> Vec<bool> {
    let qr = QRCode::from_bits(bits).with_ecc(ecc).generate().unwrap();
    let w = qr.width();
    let mut v = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            v.push(qr.module_is_dark(x, y));
        }
    }
    v
}

fn assert_matrix_eq(label: &str, a: &[bool], b: &[bool], width: usize) {
    assert_eq!(a.len(), b.len(), "{label}: length mismatch");
    for y in 0..width {
        for x in 0..width {
            let i = y * width + x;
            assert_eq!(
                a[i], b[i],
                "{label}: mismatch at ({x},{y}): ref={} ours={}",
                a[i], b[i]
            );
        }
    }
}

#[test]
fn micro_v1_123_matches_reference() {
    let data = b"123";
    let a = ref_matrix(data, RefVersion::Micro(1), EcLevel::L);
    let b = ours_matrix_bytes(data, Version::Micro(1), ECCLevel::L);
    assert_matrix_eq("micro v1 123", &a, &b, 11);
}

#[test]
fn mixed_segment_abc123hello_matches_reference() {
    let data = b"ABC123hello";
    let a = ref_matrix(data, RefVersion::Normal(1), EcLevel::M);
    let b = ours_matrix_bytes(data, Version::Normal(1), ECCLevel::M);
    assert_matrix_eq("mixed ABC123hello", &a, &b, 21);
}

#[test]
fn eci_latin1_from_bits_matches_reference() {
    let payload = b"\xa1\xa2\xa3\xa4\xa5";
    let mut ref_bits = RefBits::new(RefVersion::Normal(1));
    ref_bits.push_eci_designator(9).unwrap();
    ref_bits.push_byte_data(payload).unwrap();
    ref_bits.push_terminator(EcLevel::L).unwrap();
    let ref_qr = QrCode::with_bits(ref_bits, EcLevel::L).unwrap();
    let w = ref_qr.width();
    let mut a = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            a.push(ref_qr[(x, y)] == RefColor::Dark);
        }
    }

    let mut bits = Bits::new(Version::Normal(1));
    bits.push_eci_designator(9).unwrap();
    bits.push_byte_data(payload).unwrap();
    bits.push_terminator(ECCLevel::L).unwrap();
    let b = ours_from_bits(bits, ECCLevel::L);
    assert_matrix_eq("ECI latin-1", &a, &b, w);
}

#[test]
fn render_svg_has_custom_colors() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();
    let svg = qr
        .render()
        .dark_color("#800000")
        .light_color("#ffff80")
        .min_dimensions(200, 200)
        .build_svg();
    assert!(svg.contains("#800000"));
    assert!(svg.contains("#ffff80"));
    // min_dimensions scales module size, not the SVG width exactly
    let width_start = svg.find("width=\"").unwrap() + 7;
    let width_end = svg[width_start..].find('"').unwrap() + width_start;
    let width: u32 = svg[width_start..width_end].parse().unwrap();
    assert!(width >= 200, "expected width >= 200, got {width}");
}

#[test]
fn render_unicode_nonempty() {
    let qr = QRCode::from("123")
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(2))
        .generate()
        .unwrap();
    let text = qr.render().build_unicode();
    assert!(text.contains('\n'));
    assert!(text.chars().any(|c| c != ' ' && c != '\n'));
}

#[test]
fn render_string_matches_debug_str() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();
    assert_eq!(
        qr.render()
            .quiet_zone(false)
            .module_dimensions(1, 1)
            .dark_char('#')
            .light_char('.')
            .build_string(),
        qr.to_debug_str('#', '.')
    );
}

#[test]
fn annex_i_normal_qr_debug_str() {
    let qr = QRCode::from_bytes(b"01234567")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();
    let ref_code = QrCode::with_version(b"01234567", RefVersion::Normal(1), EcLevel::M).unwrap();
    assert_eq!(
        qr.to_debug_str('#', '.'),
        ref_code.to_debug_str('#', '.')
    );
}
