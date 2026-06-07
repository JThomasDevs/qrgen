use tuv::{QRCode, ECCLevel};

#[test]
fn debug_a() {
    let qr = QRCode::from("A").with_ecc(ECCLevel::M).generate().unwrap();
    let debug = qr.debug_full_matrix();
    eprintln!("Size: {}", qr.size());
    eprintln!("{}", debug);
}
