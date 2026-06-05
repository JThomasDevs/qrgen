use clap::Parser;
use std::path::PathBuf;
use tuv::{ECCLevel, QRCode};

#[derive(Parser, Debug)]
#[command(author, version, about = "Pure-Rust QR code generator")]
struct Args {
    /// Input string to encode
    input: String,

    /// Output file path (.svg or .png)
    #[arg(short, long)]
    output: PathBuf,

    /// Output as PNG instead of SVG
    #[arg(long)]
    png: bool,

    /// Error correction level (L, M, Q, H)
    #[arg(long, default_value = "M")]
    ecc: String,

    /// QR version (1-40, auto-selected if omitted)
    #[arg(long)]
    qr_version: Option<u8>,

    /// PNG output size in pixels (ignored for SVG)
    #[arg(long, default_value = "300")]
    size: u32,

    /// Add quiet zone (4-module margin)
    #[arg(long, default_value = "true")]
    quiet_zone: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let ecc = match args.ecc.to_uppercase().as_str() {
        "L" => ECCLevel::L,
        "M" => ECCLevel::M,
        "Q" => ECCLevel::Q,
        "H" => ECCLevel::H,
        _ => {
            eprintln!("invalid ECC level: {} (must be L, M, Q, or H)", args.ecc);
            std::process::exit(1);
        }
    };

    let qr = QRCode::new(&args.input, Some(ecc), args.qr_version)?;

    if args.png {
        let img = qr.to_png(args.size, args.quiet_zone);
        img.save(&args.output)?;
        println!("wrote {}", args.output.display());
    } else {
        let svg = qr.to_svg(args.quiet_zone);
        std::fs::write(&args.output, svg)?;
        println!("wrote {}", args.output.display());
    }

    Ok(())
}
