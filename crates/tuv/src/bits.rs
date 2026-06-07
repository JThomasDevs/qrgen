//! Low-level bit stream API mirroring the reference `qrcode` crate.

use std::cmp::min;

use crate::encoder::mode::Mode;
use crate::encoder::optimize::{Parser, Segment};
use crate::error_correction::ECCLevel;
use crate::types::{QRGenError, QrResult, Version};

/// Encoded payload bits for manual QR construction.
#[derive(Debug, Clone)]
pub struct Bits {
    data: Vec<u8>,
    bit_offset: usize,
    version: Version,
}

#[derive(Copy, Clone)]
pub(crate) enum ExtendedMode {
    Eci,
    Data(Mode),
    Fnc1First,
    Fnc1Second,
}

static DATA_LENGTHS: [[usize; 4]; 44] = [
    [152, 128, 104, 72],
    [272, 224, 176, 128],
    [440, 352, 272, 208],
    [640, 512, 384, 288],
    [864, 688, 496, 368],
    [1088, 864, 608, 480],
    [1248, 992, 704, 528],
    [1552, 1232, 880, 688],
    [1856, 1456, 1056, 800],
    [2192, 1728, 1232, 976],
    [2592, 2032, 1440, 1120],
    [2960, 2320, 1648, 1264],
    [3424, 2672, 1952, 1440],
    [3688, 2920, 2088, 1576],
    [4184, 3320, 2360, 1784],
    [4712, 3624, 2600, 2024],
    [5176, 4056, 2936, 2264],
    [5768, 4504, 3176, 2504],
    [6360, 5016, 3560, 2728],
    [6888, 5352, 3880, 3080],
    [7456, 5712, 4096, 3248],
    [8048, 6256, 4544, 3536],
    [8752, 6880, 4912, 3712],
    [9392, 7312, 5312, 4112],
    [10208, 8000, 5744, 4304],
    [10960, 8496, 6032, 4768],
    [11744, 9024, 6464, 5024],
    [12248, 9544, 6968, 5288],
    [13048, 10136, 7288, 5608],
    [13880, 10984, 7880, 5960],
    [14744, 11640, 8264, 6344],
    [15640, 12328, 8920, 6760],
    [16568, 13048, 9368, 7208],
    [17528, 13800, 9848, 7688],
    [18448, 14496, 10288, 7888],
    [19472, 15312, 10832, 8432],
    [20528, 15936, 11408, 8768],
    [21616, 16816, 12016, 9136],
    [22496, 17728, 12656, 9776],
    [23648, 18672, 13328, 10208],
    [20, 0, 0, 0],
    [40, 32, 0, 0],
    [84, 68, 0, 0],
    [128, 112, 80, 0],
];

fn alphanumeric_digit(character: u8) -> u16 {
    match character {
        b'0'..=b'9' => u16::from(character - b'0'),
        b'A'..=b'Z' => u16::from(character - b'A') + 10,
        b' ' => 36,
        b'$' => 37,
        b'%' => 38,
        b'*' => 39,
        b'+' => 40,
        b'-' => 41,
        b'.' => 42,
        b'/' => 43,
        b':' => 44,
        _ => 0,
    }
}

impl Bits {
    pub fn new(version: Version) -> Self {
        Self {
            data: Vec::new(),
            bit_offset: 0,
            version,
        }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn len(&self) -> usize {
        if self.bit_offset == 0 {
            self.data.len() * 8
        } else {
            (self.data.len().saturating_sub(1)) * 8 + self.bit_offset
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn bit_at(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_in_byte = 7 - (index % 8);
        if byte_index >= self.data.len() {
            return false;
        }
        ((self.data[byte_index] >> bit_in_byte) & 1) != 0
    }

    pub fn max_len(&self, ecc: ECCLevel) -> QrResult<usize> {
        self.version.fetch(ecc, &DATA_LENGTHS)
    }

    fn push_number(&mut self, n: usize, number: u16) {
        let b = self.bit_offset + n;
        let last_index = self.data.len().wrapping_sub(1);
        match (self.bit_offset, b) {
            (0, 0..=8) => {
                self.data.push((number << (8 - b)) as u8);
            }
            (0, _) => {
                self.data.push((number >> (b - 8)) as u8);
                self.data.push((number << (16 - b)) as u8);
            }
            (_, 0..=8) => {
                self.data[last_index] |= (number << (8 - b)) as u8;
            }
            (_, 9..=16) => {
                self.data[last_index] |= (number >> (b - 8)) as u8;
                self.data.push((number << (16 - b)) as u8);
            }
            _ => {
                self.data[last_index] |= (number >> (b - 8)) as u8;
                self.data.push((number >> (b - 16)) as u8);
                self.data.push((number << (24 - b)) as u8);
            }
        }
        self.bit_offset = b & 7;
    }

    fn push_number_checked(&mut self, n: usize, number: usize) -> QrResult<()> {
        if n > 16 || number >= (1 << n) {
            Err(QRGenError::InputTooLong {
                ecc: ECCLevel::M,
            })
        } else {
            self.push_number(n, number as u16);
            Ok(())
        }
    }

    fn reserve(&mut self, n: usize) {
        let extra_bytes = (n + (8 - self.bit_offset) % 8) / 8;
        self.data.reserve(extra_bytes);
    }

    pub(crate) fn push_mode_indicator(&mut self, mode: ExtendedMode) -> QrResult<()> {
        let number = match (self.version, mode) {
            (Version::Micro(1), ExtendedMode::Data(Mode::Numeric)) => return Ok(()),
            (Version::Micro(_), ExtendedMode::Data(Mode::Numeric)) => 0,
            (Version::Micro(_), ExtendedMode::Data(Mode::Alphanumeric)) => 1,
            (Version::Micro(_), ExtendedMode::Data(Mode::Byte)) => 0b10,
            (Version::Micro(_), ExtendedMode::Data(Mode::Kanji)) => 0b11,
            (Version::Micro(_), _) => return Err(QRGenError::UnsupportedCharacterSet),
            (_, ExtendedMode::Data(Mode::Numeric)) => 0b0001,
            (_, ExtendedMode::Data(Mode::Alphanumeric)) => 0b0010,
            (_, ExtendedMode::Data(Mode::Byte)) => 0b0100,
            (_, ExtendedMode::Data(Mode::Kanji)) => 0b1000,
            (_, ExtendedMode::Eci) => 0b0111,
            (_, ExtendedMode::Fnc1First) => 0b0101,
            (_, ExtendedMode::Fnc1Second) => 0b1001,
        };
        let bits = self.version.mode_bits_count();
        self.push_number_checked(bits, number)
            .or(Err(QRGenError::UnsupportedCharacterSet))
    }

    pub fn push_eci_designator(&mut self, eci_designator: u32) -> QrResult<()> {
        self.reserve(12);
        self.push_mode_indicator(ExtendedMode::Eci)?;
        match eci_designator {
            0..=127 => self.push_number(8, eci_designator as u16),
            128..=16383 => {
                self.push_number(2, 0b10);
                self.push_number(14, eci_designator as u16);
            }
            16384..=999_999 => {
                self.push_number(3, 0b110);
                self.push_number(5, (eci_designator >> 16) as u16);
                self.push_number(16, (eci_designator & 0xffff) as u16);
            }
            _ => {
                return Err(QRGenError::InvalidEciDesignator {
                    designator: eci_designator,
                })
            }
        }
        Ok(())
    }

    fn push_header(&mut self, mode: Mode, raw_data_len: usize) -> QrResult<()> {
        let length_bits = mode.length_bits_count(self.version);
        self.reserve(length_bits + 4 + mode.data_bits_count(raw_data_len));
        self.push_mode_indicator(ExtendedMode::Data(mode))?;
        self.push_number_checked(length_bits, raw_data_len)?;
        Ok(())
    }

    pub fn push_numeric_data(&mut self, data: &[u8]) -> QrResult<()> {
        self.push_header(Mode::Numeric, data.len())?;
        for chunk in data.chunks(3) {
            let number = chunk
                .iter()
                .map(|b| u16::from(*b - b'0'))
                .fold(0u16, |a, b| a * 10 + b);
            let length = chunk.len() * 3 + 1;
            self.push_number(length, number);
        }
        Ok(())
    }

    pub fn push_alphanumeric_data(&mut self, data: &[u8]) -> QrResult<()> {
        self.push_header(Mode::Alphanumeric, data.len())?;
        for chunk in data.chunks(2) {
            let number = chunk
                .iter()
                .map(|b| alphanumeric_digit(*b))
                .fold(0u16, |a, b| a * 45 + b);
            let length = chunk.len() * 5 + 1;
            self.push_number(length, number);
        }
        Ok(())
    }

    pub fn push_byte_data(&mut self, data: &[u8]) -> QrResult<()> {
        self.push_header(Mode::Byte, data.len())?;
        for &b in data {
            self.push_number(8, u16::from(b));
        }
        Ok(())
    }

    pub fn push_kanji_data(&mut self, data: &[u8]) -> QrResult<()> {
        self.push_header(Mode::Kanji, data.len() / 2)?;
        for kanji in data.chunks(2) {
            if kanji.len() != 2 {
                return Err(QRGenError::InvalidCharacter);
            }
            let cp = u16::from(kanji[0]) * 256 + u16::from(kanji[1]);
            let bytes = if cp < 0xe040 { cp - 0x8140 } else { cp - 0xc140 };
            let number = (bytes >> 8) * 0xc0 + (bytes & 0xff);
            self.push_number(13, number);
        }
        Ok(())
    }

    pub fn push_fnc1_first_position(&mut self) -> QrResult<()> {
        self.push_mode_indicator(ExtendedMode::Fnc1First)
    }

    pub fn push_fnc1_second_position(&mut self, application_indicator: u8) -> QrResult<()> {
        self.push_mode_indicator(ExtendedMode::Fnc1Second)?;
        self.push_number(8, u16::from(application_indicator));
        Ok(())
    }

    pub fn push_segments<I>(&mut self, data: &[u8], segments_iter: I) -> QrResult<()>
    where
        I: Iterator<Item = Segment>,
    {
        for segment in segments_iter {
            let slice = &data[segment.begin..segment.end];
            match segment.mode {
                Mode::Numeric => self.push_numeric_data(slice)?,
                Mode::Alphanumeric => self.push_alphanumeric_data(slice)?,
                Mode::Byte => self.push_byte_data(slice)?,
                Mode::Kanji => self.push_kanji_data(slice)?,
            }
        }
        Ok(())
    }

    pub fn push_optimal_data(&mut self, data: &[u8]) -> QrResult<()> {
        let segments = Parser::new(data).optimize(self.version);
        self.push_segments(data, segments)
    }

    pub fn push_terminator(&mut self, ecc: ECCLevel) -> QrResult<()> {
        let terminator_size = if let Version::Micro(a) = self.version {
            (a as usize) * 2 + 1
        } else {
            4
        };

        let cur_length = self.len();
        let data_length = self.max_len(ecc)?;
        if cur_length > data_length {
            return Err(QRGenError::InputTooLong { ecc });
        }

        let terminator_size = min(terminator_size, data_length - cur_length);
        if terminator_size > 0 {
            self.push_number(terminator_size, 0);
        }

        if self.len() < data_length {
            const PADDING_BYTES: &[u8] = &[0b1110_1100, 0b0001_0001];
            self.bit_offset = 0;
            let data_bytes_length = data_length / 8;
            let padding_bytes_count = data_bytes_length.saturating_sub(self.data.len());
            let padding = PADDING_BYTES.iter().copied().cycle().take(padding_bytes_count);
            self.data.extend(padding);
        }

        if self.len() < data_length {
            self.data.push(0);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrcode::bits::Bits as RefBits;
    use qrcode::types::{EcLevel, Version as RefVersion};

    #[test]
    fn eci_9_matches_reference() {
        let mut bits = Bits::new(Version::Normal(1));
        bits.push_eci_designator(9).unwrap();
        let mut ref_bits = RefBits::new(RefVersion::Normal(1));
        ref_bits.push_eci_designator(9).unwrap();
        assert_eq!(bits.into_bytes(), ref_bits.into_bytes());
    }

    #[test]
    fn optimal_data_mixed_matches_reference() {
        let data = b"ABC123hello";
        let mut bits = Bits::new(Version::Normal(1));
        bits.push_optimal_data(data).unwrap();
        bits.push_terminator(ECCLevel::M).unwrap();

        let mut ref_bits = RefBits::new(RefVersion::Normal(1));
        ref_bits.push_optimal_data(data).unwrap();
        ref_bits.push_terminator(EcLevel::M).unwrap();
        assert_eq!(bits.into_bytes(), ref_bits.into_bytes());
    }
}
