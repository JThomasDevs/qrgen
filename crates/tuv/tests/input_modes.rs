//! Integration tests for input modes and matrix API (numeric, byte, CJK).


use qrcode::{Color as RefColor, EcLevel, QrCode, Version as RefVersion};

use tuv::encoder::mode::{best_mode, best_mode_bytes, Mode};

use tuv::{Color, ECCLevel, QRCode, Version};



fn ref_matrix(data: &[u8], version: RefVersion, ecc: EcLevel) -> (usize, Vec<bool>) {

    let code = QrCode::with_version(data, version, ecc).unwrap();

    let w = code.width();

    let mut v = Vec::with_capacity(w * w);

    for y in 0..w {

        for x in 0..w {

            v.push(code[(x, y)] == RefColor::Dark);

        }

    }

    (w, v)

}



fn ours_matrix_str(input: &str, ecc: ECCLevel, version: Option<u8>) -> (usize, Vec<bool>) {

    let mut builder = QRCode::from(input).with_ecc(ecc);

    if let Some(version) = version {

        builder = builder.with_version(Version::Normal(i16::from(version)));

    }

    let qr = builder.generate().unwrap();

    matrix_bools(&qr)

}



fn ours_matrix_bytes(input: &[u8], ecc: ECCLevel, version: Option<u8>) -> (usize, Vec<bool>) {

    let mut builder = QRCode::from_bytes(input).with_ecc(ecc);

    if let Some(version) = version {

        builder = builder.with_version(Version::Normal(i16::from(version)));

    }

    let qr = builder.generate().unwrap();

    matrix_bools(&qr)

}



fn matrix_bools(qr: &QRCode) -> (usize, Vec<bool>) {

    let w = qr.width();

    let mut v = Vec::with_capacity(w * w);

    for y in 0..w {

        for x in 0..w {

            v.push(qr.module_is_dark(x, y));

        }

    }

    (w, v)

}



fn assert_matrices_match(input: &str, version: u8, ecc: ECCLevel) {

    let ec = match ecc {

        ECCLevel::L => EcLevel::L,

        ECCLevel::M => EcLevel::M,

        ECCLevel::Q => EcLevel::Q,

        ECCLevel::H => EcLevel::H,

    };

    let (w1, a) = ref_matrix(input.as_bytes(), RefVersion::Normal(i16::from(version)), ec);

    let (w2, b) = ours_matrix_str(input, ecc, Some(version));

    assert_eq!(w1, w2, "width mismatch for {input:?}");

    if a != b {

        for y in 0..w1 {

            for x in 0..w1 {

                let i = y * w1 + x;

                if a[i] != b[i] {

                    panic!(

                        "matrix mismatch for {input:?} at ({x},{y}): ref={} ours={}",

                        a[i], b[i]

                    );

                }

            }

        }

    }

}



fn assert_bytes_matrices_match(input: &[u8], version: u8, ecc: ECCLevel) {

    let ec = match ecc {

        ECCLevel::L => EcLevel::L,

        ECCLevel::M => EcLevel::M,

        ECCLevel::Q => EcLevel::Q,

        ECCLevel::H => EcLevel::H,

    };

    let (w1, a) = ref_matrix(input, RefVersion::Normal(i16::from(version)), ec);

    let (w2, b) = ours_matrix_bytes(input, ecc, Some(version));

    assert_eq!(w1, w2, "width mismatch for bytes {input:?}");

    if a != b {

        for y in 0..w1 {

            for x in 0..w1 {

                let i = y * w1 + x;

                if a[i] != b[i] {

                    panic!(

                        "matrix mismatch for bytes {input:?} at ({x},{y}): ref={} ours={}",

                        a[i], b[i]

                    );

                }

            }

        }

    }

}



fn assert_generates(input: &str) {

    QRCode::from(input)

        .generate()

        .unwrap_or_else(|e| panic!("failed to encode {input:?}: {e}"));

}



#[test]

fn numeric_input_selects_numeric_mode() {

    let input = "1234567890";

    assert_eq!(best_mode(input, 1), Mode::Numeric);

    assert_generates(input);

    assert_matrices_match(input, 1, ECCLevel::M);

}



#[test]

fn alphanumeric_input_selects_alphanumeric_mode() {

    let input = "HELLO WORLD";

    assert_eq!(best_mode(input, 1), Mode::Alphanumeric);

    assert_generates(input);

    assert_matrices_match(input, 1, ECCLevel::M);

}



#[test]

fn lowercase_ascii_selects_byte_mode() {

    let input = "hello";

    assert_eq!(best_mode(input, 1), Mode::Byte);

    assert_generates(input);

    assert_matrices_match(input, 1, ECCLevel::M);

}



#[test]

fn byte_string_literal_input() {

    let bytes = b"byte string";

    let input = std::str::from_utf8(bytes).expect("test fixture must be valid UTF-8");

    assert_eq!(best_mode(input, 1), Mode::Byte);

    assert_eq!(input.as_bytes(), bytes);

    assert_generates(input);

    assert_matrices_match(input, 1, ECCLevel::M);

}



#[test]

fn byte_string_literal_with_hex_escapes() {

    let bytes = b"\x48\x69";

    let input = std::str::from_utf8(bytes).expect("test fixture must be valid UTF-8");

    assert_eq!(input, "Hi");

    assert_eq!(best_mode(input, 1), Mode::Byte);

    assert_generates(input);

    assert_matrices_match(input, 1, ECCLevel::M);

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

fn from_bytes_matches_reference_for_latin1() {

    let bytes = b"\xa1\xa2\xa3\xa4\xa5";

    assert_bytes_matrices_match(bytes, 1, ECCLevel::L);

}



#[test]

fn from_bytes_numeric_matches_reference() {

    assert_bytes_matrices_match(b"01234567", 1, ECCLevel::M);

}



#[test]

fn kanji_input_selects_byte_mode() {

    let input = "日本語";

    assert_eq!(best_mode(input, 1), Mode::Byte);

    assert_generates(input);

}



#[test]

fn kanji_matches_reference() {

    assert_matrices_match("日本語", 1, ECCLevel::M);

}



#[test]

fn mixed_kanji_and_ascii_matches_reference() {

    let input = "Hello世界";

    assert_eq!(best_mode(input, 1), Mode::Byte);

    assert_generates(input);

    assert_matrices_match(input, 1, ECCLevel::M);

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

fn max_allowed_errors_matches_reference_v1() {

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


