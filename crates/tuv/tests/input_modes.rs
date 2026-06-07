//! Integration tests for input modes and matrix API (numeric, byte, CJK).

use tuv::encoder::mode::{best_mode, best_mode_bytes, Mode};
use tuv::{Color, ECCLevel, QRCode, Version};

fn assert_generates(input: &str) {
    QRCode::from(input)
        .generate()
        .unwrap_or_else(|e| panic!("failed to encode {input:?}: {e}"));
}

fn assert_v1_m_encodes(input: &str) {
    let qr = QRCode::from(input)
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap_or_else(|e| panic!("failed to encode {input:?} at v1-M: {e}"));
    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
}

#[test]
fn numeric_input_selects_numeric_mode() {
    let input = "1234567890";
    assert_eq!(best_mode(input, 1), Mode::Numeric);
    assert_generates(input);
    assert_v1_m_encodes(input);
}

#[test]
fn alphanumeric_input_selects_alphanumeric_mode() {
    let input = "HELLO WORLD";
    assert_eq!(best_mode(input, 1), Mode::Alphanumeric);
    assert_generates(input);
    assert_v1_m_encodes(input);
}

#[test]
fn lowercase_ascii_selects_byte_mode() {
    let input = "hello";
    assert_eq!(best_mode(input, 1), Mode::Byte);
    assert_generates(input);
    assert_v1_m_encodes(input);
}

#[test]
fn byte_string_literal_input() {
    let bytes = b"byte string";
    let input = std::str::from_utf8(bytes).expect("test fixture must be valid UTF-8");
    assert_eq!(best_mode(input, 1), Mode::Byte);
    assert_eq!(input.as_bytes(), bytes);
    assert_generates(input);
    assert_v1_m_encodes(input);
}

#[test]
fn byte_string_literal_with_hex_escapes() {
    let bytes = b"\x48\x69";
    let input = std::str::from_utf8(bytes).expect("test fixture must be valid UTF-8");
    assert_eq!(input, "Hi");
    assert_eq!(best_mode(input, 1), Mode::Byte);
    assert_generates(input);
    assert_v1_m_encodes(input);
}

#[test]
fn raw_bytes_non_utf8_encodes_in_byte_mode() {
    let bytes: &[u8] = &[0xFF, 0xFE, 0xFD, 0x00, 0x01];
    assert_eq!(best_mode_bytes(bytes, 1), Mode::Byte);

    let qr = QRCode::from_bytes(bytes)
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(2))
        .generate()
        .expect("non-UTF-8 bytes should encode via byte mode");

    assert_eq!(qr.version(), Version::Normal(2));
    assert_eq!(qr.width(), 25);
}

#[test]
fn from_bytes_latin1_encodes() {
    let bytes = b"\xa1\xa2\xa3\xa4\xa5";
    let qr = QRCode::from_bytes(bytes)
        .with_ecc(ECCLevel::L)
        .with_version(Version::Normal(1))
        .generate()
        .expect("Latin-1 bytes should encode");
    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
}

#[test]
fn from_bytes_numeric_encodes() {
    let qr = QRCode::from_bytes(b"01234567")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .expect("numeric bytes should encode");
    assert_eq!(qr.width(), 21);
}

#[test]
fn kanji_input_selects_byte_mode() {
    let input = "日本語";
    assert_eq!(best_mode(input, 1), Mode::Byte);
    assert_generates(input);
}

#[test]
fn kanji_utf8_encodes() {
    let qr = QRCode::from("日本語")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .expect("UTF-8 Japanese should encode in byte mode");
    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
}

#[test]
fn mixed_kanji_and_ascii_encodes() {
    let input = "Hello世界";
    assert_eq!(best_mode(input, 1), Mode::Byte);
    assert_generates(input);
    assert_v1_m_encodes(input);
}

#[test]
fn kanji_auto_version_generates() {
    let qr = QRCode::from("漢字テスト")
        .with_ecc(ECCLevel::M)
        .generate()
        .expect("kanji should encode with auto-selected version");

    assert!(matches!(qr.version(), Version::Normal(v) if v >= 1));
    assert!(qr.width() > 0);
}

#[test]
fn matrix_index_returns_color() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();

    assert_eq!(qr[(0, 0)], Color::Dark);
    assert!(qr.is_functional(0, 0));
    assert!(!qr.is_functional(10, 10));
}

#[test]
fn matrix_to_vec_and_colors() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();

    let bools = qr.to_vec();
    let colors = qr.to_colors();

    assert_eq!(bools.len(), 21 * 21);
    assert_eq!(colors.len(), 21 * 21);

    for (b, c) in bools.iter().zip(colors.iter()) {
        assert_eq!(*b, *c == Color::Dark);
    }

    let consumed = qr.clone().into_vec();
    assert_eq!(consumed.len(), bools.len());

    let consumed_colors = qr.into_colors();
    assert_eq!(consumed_colors.len(), colors.len());
}

#[test]
fn max_allowed_errors_v1_m() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();

    assert_eq!(qr.max_allowed_errors(), 4);
    assert_eq!(qr.error_correction_level(), ECCLevel::M);
}

#[test]
fn to_debug_str_matches_dark_modules() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();

    let debug = qr.to_debug_str('#', '.');

    assert_eq!(debug.lines().count(), 21);
    assert!(debug.lines().all(|line| line.len() == 21));

    for y in 0..21 {
        for x in 0..21 {
            let ch = debug.lines().nth(y).unwrap().chars().nth(x).unwrap();
            let expected = if qr.module_is_dark(x, y) { '#' } else { '.' };
            assert_eq!(ch, expected, "mismatch at ({x},{y})");
        }
    }
}

#[test]
fn micro_v1_numeric_encodes() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(1))
        .generate()
        .expect("Micro v1 should encode");
    assert_eq!(qr.version(), Version::Micro(1));
    assert_eq!(qr.width(), 11);
}
