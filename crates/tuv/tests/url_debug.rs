use tuv::{QRCode, ECCLevel};

#[test]
fn debug_matrix() {
    let qr = QRCode::new("A", Some(ECCLevel::M), None).unwrap();
    let debug = qr.debug_full_matrix();
    eprintln!("Size: {}", qr.size());
    eprintln!("{}", debug);
}
