//! Independent tests for Kanji mode (Shift JIS bytes only — no reference crate).

use tuv::bits::Bits;
use tuv::{Color, ECCLevel, QRCode, Version};

/// Shift JIS encoding of hiragana あ (U+3042).
const HIRAGANA_A: &[u8] = b"\x82\xa0";

#[test]
fn shift_jis_bytes_via_from_bytes_encodes() {
    let qr = QRCode::from_bytes(HIRAGANA_A)
        .with_ecc(ECCLevel::M)
        .with_version_number(1)
        .generate()
        .expect("single Shift JIS kanji should fit in v1-M");

    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
    assert_eq!(qr[(0, 0)] == Color::Dark, qr.module_is_dark(0, 0));
}

#[test]
fn push_kanji_data_via_bits_api_encodes() {
    let mut bits = Bits::new(Version::Normal(1));
    bits.push_kanji_data(HIRAGANA_A).expect("valid Shift JIS pair");
    let qr = QRCode::from_bits(bits)
        .with_ecc(ECCLevel::M)
        .generate()
        .expect("Kanji segment should generate");

    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
    assert!(qr.is_functional(0, 0));
    assert!(!qr.is_functional(10, 10));
}

#[test]
fn utf8_japanese_uses_byte_mode_not_kanji() {
    // UTF-8 あ is 3 bytes — byte mode, not Kanji mode.
    let utf8 = "あ";
    assert_eq!(utf8.as_bytes().len(), 3);

    let qr = QRCode::from(utf8)
        .with_ecc(ECCLevel::M)
        .with_version_number(1)
        .generate()
        .expect("UTF-8 text should encode in byte mode");

    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
}
