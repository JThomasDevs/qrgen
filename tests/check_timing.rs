use qrgen::{QRCode, ECCLevel};

#[test]
fn check_timing() {
    let qr = QRCode::new("A", Some(ECCLevel::M), None).unwrap();
    let size = qr.size();
    
    println!("Matrix size: {}", size);
    
    // Count timing modules in row 6 and col 6
    let mut row6_count = 0;
    let mut col6_count = 0;
    
    for i in 0..size {
        if matches!(qr.get_module(i, 6), qrgen::Module::Timing(_)) { row6_count += 1; }
        if matches!(qr.get_module(6, i), qrgen::Module::Timing(_)) { col6_count += 1; }
    }
    
    println!("Timing modules in row 6: {}", row6_count);
    println!("Timing modules in col 6: {}", col6_count);
    println!("Expected: 11 each (for v1, s=21)");
    
    // Print row 6
    print!("Row 6: ");
    for i in 0..size {
        let ch = match qr.get_module(i, 6) {
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
