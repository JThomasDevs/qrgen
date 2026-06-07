//! Micro QR code encoding (v1–4).

mod blocks;
mod codeword_debug;
mod data_placement;
mod format_info;
mod masking;
mod matrix;

use crate::bits::Bits;
use crate::error_correction::ECCLevel;
use crate::qrcode::QRCode;
use crate::types::{QRGenError, Version};

pub fn encode(
    input: &[u8],
    ecc: ECCLevel,
    version: Version,
    mask_id: Option<u8>,
) -> Result<QRCode, QRGenError> {
    let mut bits = Bits::new(version);
    bits.push_optimal_data(input)?;
    encode_from_bits(bits, ecc, version, mask_id)
}

pub fn encode_from_bits(
    mut bits: Bits,
    ecc: ECCLevel,
    version: Version,
    mask_id: Option<u8>,
) -> Result<QRCode, QRGenError> {
    let v = micro_version(version)?;
    validate_mask_id(mask_id)?;

    bits.push_terminator(ecc)?;
    let data_bytes = bits.into_bytes();

    let (data, ec) = blocks::construct_codewords(&data_bytes, v, ecc)?;

    let mut matrix = matrix::new_matrix(v);
    matrix::place_function_patterns(&mut matrix);
    data_placement::place_data(
        &mut matrix,
        &data,
        &ec,
        data_placement::data_ends_with_half_codeword(v, ecc),
    );

    let mask_id = masking::apply_mask_selection(&mut matrix, v, ecc, mask_id);

    Ok(crate::qrcode::build_qrcode(version, matrix, mask_id, ecc))
}

/// Maximum erratic modules correctable before data loss (matches `qrcode` `ec.rs`).
pub fn max_allowed_errors(version: u8, ecc: ECCLevel) -> usize {
    let ver = Version::Micro(i16::from(version));
    let p = match (version, ecc) {
        (2, ECCLevel::L) => 3,
        (_, ECCLevel::L) | (2, ECCLevel::M) => 2,
        _ => 0,
    };

    let ec_bytes_per_block = ver.fetch(ecc, &blocks::EC_BYTES_PER_BLOCK).unwrap_or(0);
    let (_, count1, _, count2) = ver
        .fetch(ecc, &blocks::DATA_BYTES_PER_BLOCK)
        .unwrap_or((0, 0, 0, 0));
    let ec_bytes = (count1 + count2) * ec_bytes_per_block;

    (ec_bytes.saturating_sub(p)) / 2
}

fn micro_version(version: Version) -> Result<u8, QRGenError> {
    match version {
        Version::Micro(v @ 1..=4) => Ok(v as u8),
        Version::Micro(v) | Version::Normal(v) => Err(QRGenError::InvalidVersion {
            version: v.max(0) as u8,
        }),
    }
}

fn validate_mask_id(mask_id: Option<u8>) -> Result<(), QRGenError> {
    if let Some(id) = mask_id {
        if id > 7 {
            return Err(QRGenError::InvalidMaskId { mask_id: id });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrcode::{Color, EcLevel, QrCode};
    use qrcode::types::Version as RefVersion;

    fn ref_matrix(data: &[u8], version: RefVersion, ecc: EcLevel) -> Vec<bool> {
        let code = QrCode::with_version(data, version, ecc).unwrap();
        let w = code.width();
        let mut v = Vec::with_capacity(w * w);
        for y in 0..w {
            for x in 0..w {
                v.push(code[(x, y)] == Color::Dark);
            }
        }
        v
    }

    #[test]
    fn micro_v1_123_premask_matrix_matches_reference() {
        use qrcode::canvas::Canvas;
        use qrcode::types::{EcLevel, Version as RefVersion};

        let mut bits = Bits::new(Version::Micro(1));
        bits.push_optimal_data(b"123").unwrap();
        bits.push_terminator(ECCLevel::L).unwrap();
        let raw = bits.into_bytes();
        let (data, ec_bytes) = blocks::construct_codewords(&raw, 1, ECCLevel::L).unwrap();

        let mut matrix = super::matrix::new_matrix(1);
        super::matrix::place_function_patterns(&mut matrix);
        super::data_placement::place_data(
            &mut matrix,
            &data,
            &ec_bytes,
            super::data_placement::data_ends_with_half_codeword(1, ECCLevel::L),
        );

        let mut c = Canvas::new(RefVersion::Micro(1), EcLevel::L);
        c.draw_all_functional_patterns();
        c.draw_data(&data, &ec_bytes);

        let w = matrix.size;
        for y in 0..w {
            for x in 0..w {
                let ours = matrix.get(x, y).is_dark();
                let reference = c.get(x as i16, y as i16).is_dark();
                assert_eq!(
                    ours, reference,
                    "pre-mask mismatch at ({x},{y}): ours={ours} ref={reference}"
                );
            }
        }
    }

    #[test]
    fn micro_v1_123_l_matches_reference() {
        let ours = QRCode::from_bytes(b"123")
            .with_ecc(ECCLevel::L)
            .with_version(Version::Micro(1))
            .generate()
            .expect("encode");
        let reference = ref_matrix(b"123", RefVersion::Micro(1), EcLevel::L);
        let mut ours_bits = Vec::new();
        for y in 0..ours.width() {
            for x in 0..ours.width() {
                ours_bits.push(ours.module_is_dark(x, y));
            }
        }
        assert_eq!(ours_bits, reference);
    }
}
