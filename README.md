# TUV

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

let qr = QRCode::from("https://example.com").generate()?;
let svg = qr.to_svg(true);
let png = qr.to_png(300, true);
```

- ECC defaults to level M when `with_ecc` is omitted.
- Version auto-selects the smallest QR version that fits the input when `with_version` is omitted.
- Mask auto-selects the lowest-penalty pattern when `with_mask_id` is omitted.

## CLI

An optional command-line tool lives in `crates/tuv-cli`:

```bash
cargo run -p tuv-cli -- "Hello" -o hello.svg
cargo run -p tuv-cli -- "Hello" --png -o hello.png
```

## License

MIT

## Other

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img src="assets/vondal-dev.svg" alt="Personal website" width="120" />
</p>
<!-- markdownlint-enable MD033 -->