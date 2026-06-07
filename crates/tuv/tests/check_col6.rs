use tuv::{QRCode, ECCLevel};

#[test]
fn check_col6() {
    let qr = QRCode::from("A").with_ecc(ECCLevel::M).generate().unwrap();
    let size = qr.size();
    
    println!("Matrix size: {}", size);
    println!("Timing range should be: 8 to {}", size - 3);
    
    // Print col 6
    print!("Col 6: ");
    for j in 0..size {
        let ch = match qr.get_module(6, j) {
            tuv::Module::Timing(_) => 'T',
            tuv::Module::Finder(_) => 'F',
            tuv::Module::Data(true) => '#',
            tuv::Module::Data(false) => '.',
            _ => '?',
        };
        print!("{}", ch);
    }
    println!();
}
