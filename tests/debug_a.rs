use qrgen::{QRCode, ECCLevel};

#[test]
fn debug_a() {
    let qr = QRCode::new("A", ECCLevel::M, None).unwrap();
    let debug = qr.debug_full_matrix();
    eprintln!("Size: {}", qr.size());
    eprintln!("{}", debug);
}
