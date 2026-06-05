use tuv::matrix::QRMatrix;

#[test]
fn inspect_function_patterns_only() {
    let mut m = QRMatrix::new(1);
    m.place_function_patterns();

    let mut finder = 0usize;
    let mut timing = 0usize;
    let mut data_false = 0usize;
    let mut data_true = 0usize;

    for j in 0..m.size {
        for i in 0..m.size {
            match m.get(i, j) {
                tuv::Module::Finder(_) => finder += 1,
                tuv::Module::Timing(_) => timing += 1,
                tuv::Module::Data(true) => data_true += 1,
                tuv::Module::Data(false) => data_false += 1,
                _ => {}
            }
        }
    }

    assert_eq!(finder, 147);
    assert_eq!(timing, 10);
    assert_eq!(data_true, 0);
    assert_eq!(data_false, 208);
    assert_eq!(finder + timing + data_true + data_false, 365);
}
