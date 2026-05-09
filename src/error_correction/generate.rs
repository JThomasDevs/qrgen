//! Reed-Solomon ECC codeword generation.
//!
//! Uses the same GF(256) exp/log tables and generator polynomials as the
//! widely deployed [`qrcode`](https://crates.io/crates/qrcode) crate (ISO/IEC
//! 18004 Annex A), so codewords match reference encoders.

mod iso_tables {
    include!("qrcode_ecc_tables.rs");

    pub(super) fn create_error_correction_code(data: &[u8], ec_code_size: usize) -> Vec<u8> {
        let data_len = data.len();
        let log_den = GENERATOR_POLYNOMIALS[ec_code_size];

        let mut res = data.to_vec();
        res.resize(ec_code_size + data_len, 0);

        for i in 0..data_len {
            let lead_coeff = res[i] as usize;
            if lead_coeff == 0 {
                continue;
            }

            let log_lead_coeff = usize::from(LOG_TABLE[lead_coeff]);
            for (u, v) in res[i + 1..].iter_mut().zip(log_den.iter()) {
                *u ^= EXP_TABLE[(usize::from(*v) + log_lead_coeff) % 255];
            }
        }

        res.split_off(data_len)
    }
}

/// Generate `ecc_count` ECC codewords from `data`.
pub fn generate_ecc(data: &[u8], ecc_count: usize) -> Vec<u8> {
    assert!(
        ecc_code_size_ok(ecc_count),
        "ecc_count {} out of range for QR RS tables",
        ecc_count
    );
    iso_tables::create_error_correction_code(data, ecc_count)
}

#[inline]
fn ecc_code_size_ok(ec_code_size: usize) -> bool {
    ec_code_size > 0 && ec_code_size < 70
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_qrcode_poly_mod_1() {
        let res = generate_ecc(
            b" [\x0bx\xd1r\xdcMC@\xec\x11\xec\x11\xec\x11",
            10,
        );
        assert_eq!(&*res, b"\xc4#'w\xeb\xd7\xe7\xe2]\x17");
    }
}
