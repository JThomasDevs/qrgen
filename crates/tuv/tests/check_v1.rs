use tuv::{QRCode, ECCLevel};

#[test]
fn check_v1() {
    let qr = QRCode::new("A", Some(ECCLevel::M), None).unwrap();
    println!("Matrix size: {}", qr.size());
    println!("Version (from size): {}", (qr.size() - 17) / 4);
    println!("\n{}", qr.debug_full_matrix());
}
