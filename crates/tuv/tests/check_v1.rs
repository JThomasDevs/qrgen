use tuv::{QRCode, ECCLevel};

#[test]
fn check_v1() {
    let qr = QRCode::from("A").with_ecc(ECCLevel::M).generate().unwrap();
    println!("Matrix size: {}", qr.size());
    println!("Version (from size): {}", (qr.size() - 17) / 4);
    println!("\n{}", qr.debug_full_matrix());
}
