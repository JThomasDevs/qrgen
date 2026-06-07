use clap::Parser;
use std::path::PathBuf;
use tuv::{ECCLevel, QRCode, Version};

#[derive(Parser, Debug)]
#[command(author, version, about = "Pure-Rust QR code generator")]
struct Args {
    /// Input string to encode
    input: String,

    /// Output file path (.svg or .png). Omit when using --unicode.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output as PNG instead of SVG
    #[arg(long)]
    png: bool,

    /// Error correction level (L, M, Q, H). When omitted, ECC is auto-selected.
    #[arg(long)]
    ecc: Option<String>,

    /// Normal QR version (1-40, auto-selected if omitted)
    #[arg(long, conflicts_with = "micro")]
    qr_version: Option<u8>,

    /// Micro QR version (1-4, auto-selected if omitted)
    #[arg(long, value_name = "VERSION", num_args = 0..=1, conflicts_with = "qr_version")]
    micro: Option<Option<u8>>,

    /// PNG output size in pixels (ignored for SVG)
    #[arg(long, default_value = "300")]
    size: u32,

    /// Add quiet zone margin
    #[arg(long, default_value = "true")]
    quiet_zone: bool,

    /// Print Dense1x2 unicode output to stdout instead of writing a file
    #[arg(long)]
    unicode: bool,

    /// SVG/PNG dark module color (hex)
    #[arg(long, default_value = "#000000")]
    dark_color: String,

    /// SVG/PNG light module color (hex)
    #[arg(long, default_value = "#ffffff")]
    light_color: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut builder = QRCode::from(&args.input);
    if let Some(ref ecc_str) = args.ecc {
        let ecc = match ecc_str.to_uppercase().as_str() {
            "L" => ECCLevel::L,
            "M" => ECCLevel::M,
            "Q" => ECCLevel::Q,
            "H" => ECCLevel::H,
            _ => {
                return Err(format!(
                    "invalid ECC level: {} (must be L, M, Q, or H)",
                    ecc_str
                )
                .into());
            }
        };
        builder = builder.with_ecc(ecc);
    }
    if let Some(version) = args.qr_version {
        builder = builder.with_version(Version::Normal(i16::from(version)));
    }
    if let Some(micro) = args.micro {
        builder = builder.with_micro();
        if let Some(version) = micro {
            if !(1..=4).contains(&version) {
                return Err(format!("invalid micro version: {version} (must be 1-4)").into());
            }
            builder = builder.with_version(Version::Micro(i16::from(version)));
        }
    }
    let qr = builder.generate()?;

    if args.unicode {
        let text = qr.render().quiet_zone(args.quiet_zone).build_unicode();
        print!("{text}");
        return Ok(());
    }

    let output = args
        .output
        .ok_or("output path required unless --unicode is used")?;

    if args.png {
        let img = qr
            .render()
            .quiet_zone(args.quiet_zone)
            .min_dimensions(args.size, args.size)
            .build_png();
        img.save(&output)?;
    } else {
        let svg = qr
            .render()
            .dark_color(&args.dark_color)
            .light_color(&args.light_color)
            .quiet_zone(args.quiet_zone)
            .min_dimensions(args.size, args.size)
            .build_svg();
        std::fs::write(&output, svg)?;
    }

    println!("wrote {}", output.display());
    Ok(())
}
