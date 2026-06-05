//! Compare final module grid to the `qrcode` reference implementation.
use tuv::{ECCLevel, QRCode};
use qrcode::{Color, EcLevel, QrCode, Version};

fn ref_matrix(data: &[u8], version: Version, ecc: EcLevel) -> (usize, Vec<bool>) {
    let code = QrCode::with_version(data, version, ecc).unwrap();
    let w = code.width();
    let mut v = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            v.push(code[(x, y)] == Color::Dark);
        }
    }
    (w, v)
}

fn ours_matrix(input: &str, ecc: ECCLevel, version: Option<u8>) -> (usize, Vec<bool>) {
    let qr = QRCode::new(input, Some(ecc), version).unwrap();
    let w = qr.width();
    let mut v = Vec::with_capacity(w * w);
    for y in 0..w {
        for x in 0..w {
            v.push(qr.module_is_dark(x, y));
        }
    }
    (w, v)
}

#[test]
fn v1_byte_1_matches_reference() {
    let (w1, a) = ref_matrix(b"1", Version::Normal(1), EcLevel::M);
    let (w2, b) = ours_matrix("1", ECCLevel::M, Some(1));
    assert_eq!(w1, w2, "width mismatch");
    if a != b {
        for y in 0..w1 {
            for x in 0..w1 {
                let i = y * w1 + x;
                if a[i] != b[i] {
                    panic!("first mismatch at ({x},{y}): ref={} ours={}", a[i], b[i]);
                }
            }
        }
    }
}
