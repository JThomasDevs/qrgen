#[cfg(test)]
mod codeword_debug {
    use crate::bits::Bits;
    use crate::error_correction::ECCLevel;
    use crate::micro::blocks;
    use crate::types::Version;
    use qrcode::bits::Bits as RefBits;
    use qrcode::ec;
    use qrcode::types::{EcLevel, Version as RefVersion};

    #[test]
    fn micro_v1_123_raw_and_codewords_match_reference() {
        let mut bits = Bits::new(Version::Micro(1));
        bits.push_optimal_data(b"123").unwrap();
        bits.push_terminator(ECCLevel::L).unwrap();
        let ours_raw = bits.into_bytes();

        let mut ref_bits = RefBits::new(RefVersion::Micro(1));
        ref_bits.push_optimal_data(b"123").unwrap();
        ref_bits.push_terminator(EcLevel::L).unwrap();
        let ref_raw = ref_bits.into_bytes();

        assert_eq!(ours_raw, ref_raw, "raw padded bytes");

        let (ours_data, ours_ec) = blocks::construct_codewords(&ours_raw, 1, ECCLevel::L).unwrap();
        let (ref_data, ref_ec) =
            ec::construct_codewords(&ref_raw, RefVersion::Micro(1), EcLevel::L).unwrap();

        assert_eq!(ours_data, ref_data, "data codewords");
        assert_eq!(ours_ec, ref_ec, "ec codewords");
    }
}
