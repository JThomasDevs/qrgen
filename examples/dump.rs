use qrgen::{ECCLevel, QRCode};

fn main() {
    let qr = QRCode::new("https://vondal.dev", ECCLevel::M, None).unwrap();
    print!("{}", qr.debug_matrix());
}
