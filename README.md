# TUV

Pure-Rust QR and Micro QR code encoder with SVG, PNG, and terminal output.

TUV, for QRs

## Install

```toml
[dependencies]
tuv = "0.1.0"
```

## Usage

### High-level builder (primary API)

```rust
use tuv::{QRCode, Version, ECCLevel};

// UTF-8 string (byte mode — not Kanji mode)
let qr = QRCode::from("https://example.com").generate()?;

// Raw bytes (byte mode, non-UTF-8 OK)
let qr = QRCode::from_bytes(b"\xca\xfe").with_ecc(ECCLevel::L).generate()?;

// Shift JIS Kanji mode (raw double-byte bytes, not UTF-8)
let qr = QRCode::from_bytes(b"\x82\xa0").with_ecc(ECCLevel::M).generate()?;

// Normal or Micro QR version
let qr = QRCode::from("123")
    .with_ecc(ECCLevel::L)
    .with_version(Version::Micro(1))
    .generate()?;

// Micro QR with auto-selected smallest version
let qr = QRCode::from_bytes(b"123")
    .with_micro()
    .with_ecc(ECCLevel::L)
    .generate()?;

// Backward-compatible render helpers
let svg = qr.to_svg(true);
let png = qr.to_png(300, true);
```

### Render builder

```rust
let svg = qr.render()
    .dark_color("#800000")
    .light_color("#ffff80")
    .min_dimensions(200, 200)
    .quiet_zone(true)
    .build_svg();

let terminal = qr.render().build_unicode();
let text = qr.render().dark_char('#').light_char('.').build_string();
let luma = qr.render().build_image_luma();
```

### Low-level Bits API

```rust
use tuv::bits::Bits;
use tuv::{QRCode, Version, ECCLevel};

let mut bits = Bits::new(Version::Normal(1));
bits.push_eci_designator(9)?;
bits.push_byte_data(b"\xa1\xa2\xa3")?;
bits.push_terminator(ECCLevel::L)?;
let qr = QRCode::from_bits(bits).with_ecc(ECCLevel::L).generate()?;
```

### Matrix introspection

```rust
use tuv::Color;

let dark = qr[(x, y)] == Color::Dark;
let max_err = qr.max_allowed_errors();
let functional = qr.is_functional(x, y);
let debug = qr.to_debug_str('#', '.');
```

## Defaults

- When `with_ecc` is omitted, version and ECC are **co-optimized** for the smallest symbol (try each version from smallest upward, and at each version try ECC L → M → Q → H).
- When `with_version` is set but ECC is omitted, the **lowest** ECC level that fits at that version is chosen.
- When `with_ecc` is set but version is omitted, only version is auto-selected (unchanged).
- Call [`with_micro()`](https://docs.rs/tuv/latest/tuv/struct.QRCodeBuilder.html#method.with_micro) to search Micro QR versions (v1–4) instead of Normal QR (v1–40).
- [`QRCode::from`](https://docs.rs/tuv/latest/tuv/struct.QRCode.html#method.from) / UTF-8 strings use **byte mode**. True **Kanji mode** requires Shift JIS bytes via [`from_bytes`](https://docs.rs/tuv/latest/tuv/struct.QRCode.html#method.from_bytes) or [`Bits::push_kanji_data`](https://docs.rs/tuv/latest/tuv/bits/struct.Bits.html#method.push_kanji_data).
- Mask auto-selects the lowest-penalty pattern when `with_mask_id` is omitted.
- Micro QR uses a 2-module quiet zone; Normal QR uses 4.

## CLI

An optional command-line tool lives in `crates/tuv-cli`:

```bash
cargo run -p tuv-cli -- "Hello" -o hello.svg
cargo run -p tuv-cli -- "Hello" --png -o hello.png
cargo run -p tuv-cli -- "123" --micro 1 --ecc L -o micro.svg
cargo run -p tuv-cli -- "123" --micro --ecc L -o micro-auto.svg
cargo run -p tuv-cli -- "Hello" --unicode
cargo run -p tuv-cli -- "Hello" --dark-color "#800000" --light-color "#ffff80" -o colored.svg
```

## Parity

Encoding and matrices are tested against [qrcode 0.14.1](https://docs.rs/qrcode/0.14.1/qrcode/) (dev-dependency). PIC output is intentionally excluded.

## License

MIT

## Other

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img src="assets/vondal-dev.svg" alt="Personal website" width="120" />
</p>
<!-- markdownlint-enable MD033 -->
