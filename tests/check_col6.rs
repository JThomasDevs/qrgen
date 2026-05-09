use qrgen::{QRCode, ECCLevel};

#[test]
fn check_col6() {
    let qr = QRCode::new("A", ECCLevel::M, None).unwrap();
    let size = qr.size();
    
    println!("Matrix size: {}", size);
    println!("Timing range should be: 8 to {}", size - 3);
    
    // Print col 6
    print!("Col 6: ");
    for j in 0..size {
        let ch = match qr.get_module(6, j) {
            qrgen::Module::Timing(_) => 'T',
            qrgen::Module::Finder(_) => 'F',
            qrgen::Module::Data(true) => '#',
            qrgen::Module::Data(false) => '.',
            _ => '?',
        };
        print!("{}", ch);
    }
    println!();
}
