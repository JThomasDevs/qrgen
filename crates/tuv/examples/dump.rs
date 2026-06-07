use tuv::{ECCLevel, QRCode};

fn main() {
    let qr = QRCode::from("https://vondal.dev")
        .with_ecc(ECCLevel::M)
        .generate()
        .unwrap();
    print!("{}", qr.debug_matrix());
}
