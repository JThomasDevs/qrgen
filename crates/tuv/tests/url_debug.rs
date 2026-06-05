use tuv::{ECCLevel, QRCode};

#[test]
fn debug_matrix() {
    let qr = QRCode::new("A", Some(ECCLevel::M), None).unwrap();
    let debug = qr.debug_full_matrix();

    assert_eq!(qr.size(), 21);
    assert!(debug.starts_with("size: 21\n"));
    assert!(debug.contains("Module counts:"));
    assert!(debug.contains("Finder: 147"));
    assert!(debug.contains("Matrix:"));

    let matrix_rows: Vec<&str> = debug
        .lines()
        .skip_while(|line| *line != "Matrix:")
        .skip(1)
        .collect();
    assert_eq!(matrix_rows.len(), 21);
    assert_eq!(matrix_rows[0], "FFFFFFFsf#.#.sFFFFFFF");
}
