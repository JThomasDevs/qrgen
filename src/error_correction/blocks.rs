//! Block splitting and interleaving per ISO/IEC 18004 §6.6 and Table 9.
//!
//! For each `(version, ecc)` combination the spec specifies up to two
//! "groups" of equal-size blocks. Group 2 blocks are exactly one byte
//! longer than Group 1 blocks (or Group 2 may be empty). All blocks share
//! the same ECC codeword count per block.
//!
//! After RS-encoding each block, data bytes are interleaved by reading the
//! n-th byte of every block in order (skipping blocks that are too short
//! at that index). ECC bytes are interleaved the same way.

/// A data block with its ECC codewords already computed.
#[derive(Debug, Clone)]
pub struct DataBlock {
    pub data: Vec<u8>,
    pub ecc: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct BlockSpec {
    g1_blocks: u8,
    g1_size: u16,
    g2_blocks: u8,
    g2_size: u16,
    ecc_per_block: u16,
}

const fn bs(g1c: u8, g1s: u16, g2c: u8, g2s: u16, e: u16) -> BlockSpec {
    BlockSpec { g1_blocks: g1c, g1_size: g1s, g2_blocks: g2c, g2_size: g2s, ecc_per_block: e }
}

/// `BLOCK_PARAMS[version - 1][ecc_index]` — ECC index: 0=L, 1=M, 2=Q, 3=H.
const BLOCK_PARAMS: [[BlockSpec; 4]; 40] = [
    // v1
    [bs(1, 19, 0, 0, 7), bs(1, 16, 0, 0, 10), bs(1, 13, 0, 0, 13), bs(1, 9, 0, 0, 17)],
    // v2
    [bs(1, 34, 0, 0, 10), bs(1, 28, 0, 0, 16), bs(1, 22, 0, 0, 22), bs(1, 16, 0, 0, 28)],
    // v3
    [bs(1, 55, 0, 0, 15), bs(1, 44, 0, 0, 26), bs(2, 17, 0, 0, 18), bs(2, 13, 0, 0, 22)],
    // v4
    [bs(1, 80, 0, 0, 20), bs(2, 32, 0, 0, 18), bs(2, 24, 0, 0, 26), bs(4, 9, 0, 0, 16)],
    // v5
    [bs(1, 108, 0, 0, 26), bs(2, 43, 0, 0, 24), bs(2, 15, 2, 16, 18), bs(2, 11, 2, 12, 22)],
    // v6
    [bs(2, 68, 0, 0, 18), bs(4, 27, 0, 0, 16), bs(4, 19, 0, 0, 24), bs(4, 15, 0, 0, 28)],
    // v7
    [bs(2, 78, 0, 0, 20), bs(4, 31, 0, 0, 18), bs(2, 14, 4, 15, 18), bs(4, 13, 1, 14, 26)],
    // v8
    [bs(2, 97, 0, 0, 24), bs(2, 38, 2, 39, 22), bs(4, 18, 2, 19, 22), bs(4, 14, 2, 15, 26)],
    // v9
    [bs(2, 116, 0, 0, 30), bs(3, 36, 2, 37, 22), bs(4, 16, 4, 17, 20), bs(4, 12, 4, 13, 24)],
    // v10
    [bs(2, 68, 2, 69, 18), bs(4, 43, 1, 44, 26), bs(6, 19, 2, 20, 24), bs(6, 15, 2, 16, 28)],
    // v11
    [bs(4, 81, 0, 0, 20), bs(1, 50, 4, 51, 30), bs(4, 22, 4, 23, 28), bs(3, 12, 8, 13, 24)],
    // v12
    [bs(2, 92, 2, 93, 24), bs(6, 36, 2, 37, 22), bs(4, 20, 6, 21, 26), bs(7, 14, 4, 15, 28)],
    // v13
    [bs(4, 107, 0, 0, 26), bs(8, 37, 1, 38, 22), bs(8, 20, 4, 21, 24), bs(12, 11, 4, 12, 22)],
    // v14
    [bs(3, 115, 1, 116, 30), bs(4, 40, 5, 41, 24), bs(11, 16, 5, 17, 20), bs(11, 12, 5, 13, 24)],
    // v15
    [bs(5, 87, 1, 88, 22), bs(5, 41, 5, 42, 24), bs(5, 24, 7, 25, 30), bs(11, 12, 7, 13, 24)],
    // v16
    [bs(5, 98, 1, 99, 24), bs(7, 45, 3, 46, 28), bs(15, 19, 2, 20, 24), bs(3, 15, 13, 16, 30)],
    // v17
    [bs(1, 107, 5, 108, 28), bs(10, 46, 1, 47, 28), bs(1, 22, 15, 23, 28), bs(2, 14, 17, 15, 28)],
    // v18
    [bs(5, 120, 1, 121, 30), bs(9, 43, 4, 44, 26), bs(17, 22, 1, 23, 28), bs(2, 14, 19, 15, 28)],
    // v19
    [bs(3, 113, 4, 114, 28), bs(3, 44, 11, 45, 26), bs(17, 21, 4, 22, 26), bs(9, 13, 16, 14, 26)],
    // v20
    [bs(3, 107, 5, 108, 28), bs(3, 41, 13, 42, 26), bs(15, 24, 5, 25, 30), bs(15, 15, 10, 16, 28)],
    // v21
    [bs(4, 116, 4, 117, 28), bs(17, 42, 0, 0, 26), bs(17, 22, 6, 23, 28), bs(19, 16, 6, 17, 30)],
    // v22
    [bs(2, 111, 7, 112, 28), bs(17, 46, 0, 0, 28), bs(7, 24, 16, 25, 30), bs(34, 13, 0, 0, 24)],
    // v23
    [bs(4, 121, 5, 122, 30), bs(4, 47, 14, 48, 28), bs(11, 24, 14, 25, 30), bs(16, 15, 14, 16, 30)],
    // v24
    [bs(6, 117, 4, 118, 30), bs(6, 45, 14, 46, 28), bs(11, 24, 16, 25, 30), bs(30, 16, 2, 17, 30)],
    // v25
    [bs(8, 106, 4, 107, 26), bs(8, 47, 13, 48, 28), bs(7, 24, 22, 25, 30), bs(22, 15, 13, 16, 30)],
    // v26
    [bs(10, 114, 2, 115, 28), bs(19, 46, 4, 47, 28), bs(28, 22, 6, 23, 28), bs(33, 16, 4, 17, 30)],
    // v27
    [bs(8, 122, 4, 123, 30), bs(22, 45, 3, 46, 28), bs(8, 23, 26, 24, 30), bs(12, 15, 28, 16, 30)],
    // v28
    [bs(3, 117, 10, 118, 30), bs(3, 45, 23, 46, 28), bs(4, 24, 31, 25, 30), bs(11, 15, 31, 16, 30)],
    // v29
    [bs(7, 116, 7, 117, 30), bs(21, 45, 7, 46, 28), bs(1, 23, 37, 24, 30), bs(19, 15, 26, 16, 30)],
    // v30
    [bs(5, 115, 10, 116, 30), bs(19, 47, 10, 48, 28), bs(15, 24, 25, 25, 30), bs(23, 15, 25, 16, 30)],
    // v31
    [bs(13, 115, 3, 116, 30), bs(2, 46, 29, 47, 28), bs(42, 24, 1, 25, 30), bs(23, 15, 28, 16, 30)],
    // v32
    [bs(17, 115, 0, 0, 30), bs(10, 46, 23, 47, 28), bs(10, 24, 35, 25, 30), bs(19, 15, 35, 16, 30)],
    // v33
    [bs(17, 115, 1, 116, 30), bs(14, 46, 21, 47, 28), bs(29, 24, 19, 25, 30), bs(11, 15, 46, 16, 30)],
    // v34
    [bs(13, 115, 6, 116, 30), bs(14, 46, 23, 47, 28), bs(44, 24, 7, 25, 30), bs(59, 16, 1, 17, 30)],
    // v35
    [bs(12, 121, 7, 122, 30), bs(12, 47, 26, 48, 28), bs(39, 24, 14, 25, 30), bs(22, 15, 41, 16, 30)],
    // v36
    [bs(6, 121, 14, 122, 30), bs(6, 47, 34, 48, 28), bs(46, 24, 10, 25, 30), bs(2, 15, 64, 16, 30)],
    // v37
    [bs(17, 122, 4, 123, 30), bs(29, 46, 14, 47, 28), bs(49, 24, 10, 25, 30), bs(24, 15, 46, 16, 30)],
    // v38
    [bs(4, 122, 18, 123, 30), bs(13, 46, 32, 47, 28), bs(48, 24, 14, 25, 30), bs(42, 15, 32, 16, 30)],
    // v39
    [bs(20, 117, 4, 118, 30), bs(40, 47, 7, 48, 28), bs(43, 24, 22, 25, 30), bs(10, 15, 67, 16, 30)],
    // v40
    [bs(19, 118, 6, 119, 30), bs(18, 47, 31, 48, 28), bs(34, 24, 34, 25, 30), bs(20, 15, 61, 16, 30)],
];

fn ecc_index(ecc: super::ECCLevel) -> usize {
    match ecc {
        super::ECCLevel::L => 0,
        super::ECCLevel::M => 1,
        super::ECCLevel::Q => 2,
        super::ECCLevel::H => 3,
    }
}

fn spec(version: u8, ecc: super::ECCLevel) -> BlockSpec {
    BLOCK_PARAMS[(version as usize) - 1][ecc_index(ecc)]
}

/// Total user data codewords across all blocks for a `(version, ecc)`.
pub fn total_data_codewords(version: u8, ecc: super::ECCLevel) -> usize {
    let s = spec(version, ecc);
    (s.g1_blocks as usize) * (s.g1_size as usize)
        + (s.g2_blocks as usize) * (s.g2_size as usize)
}

/// Number of ECC codewords per block for `(version, ecc)`.
pub fn ecc_codewords_per_block(version: u8, ecc: super::ECCLevel) -> usize {
    spec(version, ecc).ecc_per_block as usize
}

/// Split the byte-padded data into blocks per Table 9 and compute ECC.
pub fn split_into_blocks(
    data: &[u8],
    version: u8,
    ecc: super::ECCLevel,
) -> Vec<DataBlock> {
    let s = spec(version, ecc);
    let total = total_data_codewords(version, ecc);
    assert_eq!(
        data.len(),
        total,
        "data length {} != total data codewords {} for version {} ecc {:?}",
        data.len(),
        total,
        version,
        ecc
    );

    let mut blocks: Vec<DataBlock> = Vec::with_capacity((s.g1_blocks + s.g2_blocks) as usize);
    let mut offset = 0usize;
    for _ in 0..s.g1_blocks {
        let end = offset + s.g1_size as usize;
        let block_data = data[offset..end].to_vec();
        offset = end;
        let block_ecc = super::generate::generate_ecc(&block_data, s.ecc_per_block as usize);
        blocks.push(DataBlock { data: block_data, ecc: block_ecc });
    }
    for _ in 0..s.g2_blocks {
        let end = offset + s.g2_size as usize;
        let block_data = data[offset..end].to_vec();
        offset = end;
        let block_ecc = super::generate::generate_ecc(&block_data, s.ecc_per_block as usize);
        blocks.push(DataBlock { data: block_data, ecc: block_ecc });
    }
    blocks
}

/// Interleave blocks per ISO/IEC 18004 §6.6: read the j-th data byte
/// across all blocks (skipping shorter blocks at that index), then do the
/// same for ECC bytes.
pub fn interleave(blocks: &[DataBlock]) -> Vec<u8> {
    let max_data = blocks.iter().map(|b| b.data.len()).max().unwrap_or(0);
    let max_ecc = blocks.iter().map(|b| b.ecc.len()).max().unwrap_or(0);

    let mut out = Vec::with_capacity(
        blocks.iter().map(|b| b.data.len() + b.ecc.len()).sum(),
    );

    for j in 0..max_data {
        for block in blocks {
            if let Some(&byte) = block.data.get(j) {
                out.push(byte);
            }
        }
    }
    for j in 0..max_ecc {
        for block in blocks {
            if let Some(&byte) = block.ecc.get(j) {
                out.push(byte);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ECCLevel;

    #[test]
    fn v1_m_capacity() {
        assert_eq!(total_data_codewords(1, ECCLevel::M), 16);
        assert_eq!(ecc_codewords_per_block(1, ECCLevel::M), 10);
    }

    #[test]
    fn v2_m_capacity() {
        assert_eq!(total_data_codewords(2, ECCLevel::M), 28);
    }

    #[test]
    fn round_robin_interleave() {
        let blocks = vec![
            DataBlock { data: vec![1, 2, 3], ecc: vec![10, 11] },
            DataBlock { data: vec![4, 5, 6], ecc: vec![12, 13] },
        ];
        // Round-robin: 1,4,2,5,3,6 then 10,12,11,13.
        assert_eq!(interleave(&blocks), vec![1, 4, 2, 5, 3, 6, 10, 12, 11, 13]);
    }

    #[test]
    fn interleave_handles_uneven_blocks() {
        // Group 1: 1 block × 2 bytes; Group 2: 1 block × 3 bytes.
        let blocks = vec![
            DataBlock { data: vec![1, 2], ecc: vec![10] },
            DataBlock { data: vec![3, 4, 5], ecc: vec![11] },
        ];
        // Round-robin data: 1,3, 2,4, 5 (only group 2 has the third byte).
        assert_eq!(interleave(&blocks), vec![1, 3, 2, 4, 5, 10, 11]);
    }
}
