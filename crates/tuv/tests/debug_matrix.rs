use tuv::{ECCLevel, QRCode, Version};

#[test]
fn debug_matrix() {
    let qr = QRCode::from("1")
        .with_ecc(ECCLevel::M)
        .with_version(Version::Normal(1))
        .generate()
        .unwrap();
    let debug = qr.debug_matrix();

    assert!(debug.starts_with("size: 21\n"));
    let rows: Vec<&str> = debug.lines().skip(1).collect();
    assert_eq!(rows.len(), 21);
    assert!(rows.iter().all(|row| row.len() == 21));
    assert!(rows.iter().all(|row| row.chars().all(|c| matches!(c, '#' | '.' | '?'))));
    assert_eq!(rows[0], "?????????#.#.????????");
}
