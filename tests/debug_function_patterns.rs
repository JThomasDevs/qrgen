use qrgen::matrix::QRMatrix;

#[test]
fn inspect_function_patterns_only() {
    let mut m = QRMatrix::new(1);
    m.place_function_patterns();

    // Count module types
    let mut finder = 0usize;
    let mut timing = 0usize;
    let mut data_false = 0usize;
    let mut data_true = 0usize;

    for j in 0..m.size {
        for i in 0..m.size {
            match m.get(i, j) {
                qrgen::Module::Finder(_) => finder += 1,
                qrgen::Module::Timing(_) => timing += 1,
                qrgen::Module::Data(true) => { data_true += 1; }
                qrgen::Module::Data(false) => { data_false += 1; }
                _ => {}
            }
        }
    }

    eprintln!("After function patterns:");
    eprintln!("  Finder modules: {}", finder);
    eprintln!("  Timing modules: {}", timing);
    eprintln!("  Data(true): {}", data_true);
    eprintln!("  Data(false): {}", data_false);
    eprintln!("  Total: {}", finder + timing + data_true + data_false);

    // Print row 0
    eprintln!("\nRow 0:");
    for i in 0..m.size {
        match m.get(i, 0) {
            qrgen::Module::Data(false) => eprint!("."),
            qrgen::Module::Data(true) => eprint!("#"),
            qrgen::Module::Finder(_) => eprint!("F"),
            qrgen::Module::Timing(_) => eprint!("T"),
            _ => eprint!("?"),
        }
    }
    eprintln!();

    // Print row 7
    eprintln!("\nRow 7:");
    for i in 0..m.size {
        match m.get(i, 7) {
            qrgen::Module::Data(false) => eprint!("."),
            qrgen::Module::Data(true) => eprint!("#"),
            qrgen::Module::Finder(_) => eprint!("F"),
            qrgen::Module::Timing(_) => eprint!("T"),
            _ => eprint!("?"),
        }
    }
    eprintln!();
}
