use tuv::{QRCode, ECCLevel};

#[test]
fn data_bits_trace() {
    let input = "1";
    let qr = QRCode::new(input, Some(ECCLevel::M), Some(1)).unwrap();
    
    eprintln!("QR matrix size: {}", qr.size());
    eprintln!("");
    
    eprintln!("Row 0:");
    for i in 0..qr.size() {
        let m = qr.get_module(i, 0);
        let ch = match m {
            tuv::Module::Finder(_) => 'F',
            tuv::Module::Data(true) => '#',
            tuv::Module::Data(false) => '.',
            _ => '?',
        };
        eprint!("{}", ch);
    }
    eprintln!();
    
    eprintln!("Row 1:");
    for i in 0..qr.size() {
        let m = qr.get_module(i, 1);
        let ch = match m {
            tuv::Module::Finder(_) => 'F',
            tuv::Module::Data(true) => '#',
            tuv::Module::Data(false) => '.',
            _ => '?',
        };
        eprint!("{}", ch);
    }
    eprintln!();
}
