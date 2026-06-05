use tuv::{QRCode, ECCLevel};

#[test]
fn debug_matrix() {
    let qr = QRCode::new("1", Some(ECCLevel::M), Some(1)).unwrap();
    let debug = qr.debug_matrix();
    eprintln!("{}", debug);
}
