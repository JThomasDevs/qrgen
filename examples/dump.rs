use qrgen::{ECCLevel, QRCode};

fn main() {
    let qr = QRCode::new("https://vondal.dev", Some(ECCLevel::M), None).unwrap();
    print!("{}", qr.debug_matrix());
}
