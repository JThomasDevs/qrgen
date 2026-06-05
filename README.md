# Tuv

Pure-Rust QR code encoder with SVG and PNG output.

TUV, for QRs

## Install

```toml
[dependencies]
tuv = "0.1.0"
```

## Usage

```rust
use tuv::QRCode;

let qr = QRCode::new("https://example.com", None, None)?;
let svg = qr.to_svg(true);
let png = qr.to_png(300, true);
```

- `ecc: None` defaults to ECC level M.
- `version: None` auto-selects the smallest QR version that fits the input.

## CLI

An optional command-line tool lives in `crates/tuv-cli`:

```bash
cargo run -p tuv-cli -- "Hello" -o hello.svg
cargo run -p tuv-cli -- "Hello" --png -o hello.png
```

## License

MIT
