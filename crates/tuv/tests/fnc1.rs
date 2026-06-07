//! Independent tests for FNC1 mode indicators (no reference crate).

use tuv::bits::Bits;
use tuv::{Color, ECCLevel, QRCode, Version};

#[test]
fn fnc1_first_position_with_numeric_data() {
    let mut bits = Bits::new(Version::Normal(1));
    bits.push_fnc1_first_position()
        .expect("FNC1 first position indicator");
    bits.push_numeric_data(b"0123456789012")
        .expect("GS1 numeric payload");

    let qr = QRCode::from_bits(bits)
        .with_ecc(ECCLevel::M)
        .generate()
        .expect("FNC1 + numeric should generate");

    assert_eq!(qr.version(), Version::Normal(1));
    assert_eq!(qr.width(), 21);
    assert!(qr.is_functional(0, 0));
    assert!(!qr.is_functional(10, 10));

    for y in 0..qr.width() {
        for x in 0..qr.width() {
            assert_eq!(qr[(x, y)] == Color::Dark, qr.module_is_dark(x, y));
        }
    }
}

#[test]
fn fnc1_second_position_with_alphanumeric_data() {
    let mut bits = Bits::new(Version::Normal(2));
    bits.push_fnc1_second_position(b'A')
        .expect("FNC1 second position with application indicator");
    bits.push_alphanumeric_data(b"HELLO")
        .expect("alphanumeric payload");

    let qr = QRCode::from_bits(bits)
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(2))
        .generate()
        .expect("FNC1 second + alphanumeric should generate");

    assert_eq!(qr.version(), Version::Normal(2));
    assert_eq!(qr.width(), 25);
}
