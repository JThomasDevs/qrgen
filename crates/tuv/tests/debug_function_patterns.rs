use tuv::matrix::QRMatrix;

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
                tuv::Module::Finder(_) => finder += 1,
                tuv::Module::Timing(_) => timing += 1,
                tuv::Module::Data(true) => { data_true += 1; }
                tuv::Module::Data(false) => { data_false += 1; }
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
            tuv::Module::Data(false) => eprint!("."),
            tuv::Module::Data(true) => eprint!("#"),
            tuv::Module::Finder(_) => eprint!("F"),
            tuv::Module::Timing(_) => eprint!("T"),
            _ => eprint!("?"),
        }
    }
    eprintln!();

    // Print row 7
    eprintln!("\nRow 7:");
    for i in 0..m.size {
        match m.get(i, 7) {
            tuv::Module::Data(false) => eprint!("."),
            tuv::Module::Data(true) => eprint!("#"),
            tuv::Module::Finder(_) => eprint!("F"),
            tuv::Module::Timing(_) => eprint!("T"),
            _ => eprint!("?"),
        }
    }
    eprintln!();
}
