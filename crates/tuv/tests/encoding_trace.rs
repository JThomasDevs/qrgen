use tuv::{ECCLevel, Module, QRCode};

fn map_module_char(module: Module) -> char {
    match module {
        Module::Finder(_) => 'F',
        Module::Data(true) => '#',
        Module::Data(false) => '.',
        _ => '?',
    }
}

fn render_row(qr: &QRCode, y: usize) -> String {
    (0..qr.size())
        .map(|x| map_module_char(qr.get_module(x, y)))
        .collect()
}

#[test]
fn data_bits_trace() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(1)
        .generate()
        .unwrap();

    assert_eq!(qr.size(), 21);
    assert_eq!(render_row(&qr, 0), "FFFFFFF??#.#.?FFFFFFF");
    assert_eq!(render_row(&qr, 1), "FFFFFFF??#..#?FFFFFFF");
}
