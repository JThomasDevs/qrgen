use tuv::{QRCode, ECCLevel};

#[test]
fn check_finders() {
    let qr = QRCode::new("A", Some(ECCLevel::M), None).unwrap();
    
    // Check finder patterns at key positions
    // Top-left finder: positions (0,0) through (6,6)
    // Top-right finder: positions (14,0) through (20,6) for version 1
    // Bottom-left finder: positions (0,14) through (6,20)
    
    println!("Version: {}", qr.size() / 4 + 13);
    
    // Check a few key positions
    let checks = [
        (0, 0, "top-left finder corner"),
        (3, 3, "top-left finder center"),
        (6, 0, "top-left finder right edge"),
        (0, 6, "top-left finder bottom edge"),
        (7, 0, "separator top"),
        (6, 7, "separator left"),
    ];
    
    for (i, j, desc) in checks {
        let m = qr.get_module(i, j);
        println!("({}, {}) {}: {:?}", i, j, desc, m);
    }
}
