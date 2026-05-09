use qrgen::{QRCode, ECCLevel};

#[test]
fn check_v1() {
    let qr = QRCode::new("A", ECCLevel::M, None).unwrap();
    println!("Matrix size: {}", qr.size());
    println!("Version (from size): {}", qr.size() / 4 + 13);
    println!("\n{}", qr.debug_full_matrix());
}
