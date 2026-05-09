use qrgen::{QRCode, ECCLevel};

#[test]
fn debug_matrix() {
    let qr = QRCode::new("1", ECCLevel::M, Some(1)).unwrap();
    let debug = qr.debug_matrix();
    eprintln!("{}", debug);
}
