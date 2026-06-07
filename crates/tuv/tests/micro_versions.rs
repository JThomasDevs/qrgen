//! Independent integration tests for Micro QR versions 2–4 (no reference crate).

use tuv::{ECCLevel, QRCode, QRGenError, Version};

fn micro_width(n: i16) -> usize {
    Version::Micro(n).width()
}

fn count_modules(qr: &QRCode) -> (usize, usize) {
    let w = qr.width();
    let mut functional = 0usize;
    let mut data = 0usize;
    for y in 0..w {
        for x in 0..w {
            if qr.is_functional(x, y) {
                functional += 1;
            } else {
                data += 1;
            }
        }
    }
    (functional, data)
}

fn encode_micro(payload: &str, version: i16, ecc: ECCLevel) -> QRCode {
    QRCode::from(payload)
        .with_ecc(ecc)
        .with_version(Version::Micro(version))
        .generate()
        .unwrap_or_else(|e| panic!("Micro({version}) should encode {payload:?}: {e}"))
}

#[test]
fn micro_auto_selects_smallest_version() {
    let qr = QRCode::from_bytes(b"123")
        .with_micro()
        .with_ecc(ECCLevel::L)
        .generate()
        .expect("short numeric payload should fit Micro v1");

    assert_eq!(qr.version(), Version::Micro(1));
    assert_eq!(qr.width(), micro_width(1));
    assert_eq!(qr.width(), 11);
}

#[test]
fn micro_v2_encodes_numeric_payload() {
    let qr = encode_micro("1234567890", 2, ECCLevel::L);
    assert_eq!(qr.version(), Version::Micro(2));
    assert_eq!(qr.width(), micro_width(2));
    assert_eq!(qr.width(), 13);

    let (functional, data) = count_modules(&qr);
    assert!(functional > 0, "expected functional modules");
    assert!(data > 0, "expected data modules");
    assert_eq!(functional + data, qr.width() * qr.width());

    let svg = qr.to_svg(true);
    assert!(!svg.is_empty());
    assert!(svg.contains("<svg"));
}

#[test]
fn micro_v3_encodes_alphanumeric_payload() {
    let qr = encode_micro("HELLOWORLD", 3, ECCLevel::L);
    assert_eq!(qr.version(), Version::Micro(3));
    assert_eq!(qr.width(), micro_width(3));
    assert_eq!(qr.width(), 15);

    let (functional, data) = count_modules(&qr);
    assert!(functional > 0, "expected functional modules");
    assert!(data > 0, "expected data modules");

    let text = qr.to_debug_str('#', '.');
    assert!(!text.is_empty());
    assert_eq!(text.lines().count(), qr.width());
}

#[test]
fn micro_v4_encodes_byte_payload() {
    let payload = b"hello.example";
    let qr = QRCode::from_bytes(payload)
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(4))
        .generate()
        .expect("Micro v4 should encode byte payload");
    assert_eq!(qr.version(), Version::Micro(4));
    assert_eq!(qr.width(), micro_width(4));
    assert_eq!(qr.width(), 17);

    let (functional, data) = count_modules(&qr);
    assert!(functional > 0);
    assert!(data > 0);

    let png = qr.to_png(64, true);
    assert!(png.width() >= 64);
    assert!(png.height() >= 64);
}

#[test]
fn micro_v2_rejects_payload_too_long() {
    let err = QRCode::from("12345678901")
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(2))
        .generate()
        .unwrap_err();
    assert!(
        matches!(
            err,
            QRGenError::InputTooLong { .. } | QRGenError::InputTooLongForVersion { .. }
        ),
        "expected InputTooLong, got {err:?}"
    );
}

#[test]
fn micro_v3_rejects_payload_too_long() {
    let payload = "1".repeat(24);
    let err = QRCode::from(&payload)
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(3))
        .generate()
        .unwrap_err();
    assert!(
        matches!(
            err,
            QRGenError::InputTooLong { .. } | QRGenError::InputTooLongForVersion { .. }
        ),
        "expected InputTooLong, got {err:?}"
    );
}

#[test]
fn micro_v4_rejects_payload_too_long() {
    let payload = "A".repeat(40);
    let err = QRCode::from(&payload)
        .with_ecc(ECCLevel::L)
        .with_version(Version::Micro(4))
        .generate()
        .unwrap_err();
    assert!(
        matches!(
            err,
            QRGenError::InputTooLong { .. } | QRGenError::InputTooLongForVersion { .. }
        ),
        "expected InputTooLong, got {err:?}"
    );
}
