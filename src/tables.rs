// ======================================================================
// FUNCTIONS - CRATE

#[inline(always)]
pub(crate) fn update_inverted_crc32(inverted_crc: u32, byte: u8) -> u32 {
    CRC32[0][(inverted_crc as u8 ^ byte) as usize] ^ (inverted_crc >> 8)
}

/// This is equivalent to calling `update_inverted_crc32` for each byte, but faster.
#[cfg(not(feature = "crc32c"))]
#[inline(always)]
pub(crate) fn update_inverted_crc32_8bytes(mut inverted_crc: u32, data: &[u8]) -> u32 {
    debug_assert_eq!(data.len(), 8);

    inverted_crc ^= data[0] as u32
        | ((data[1] as u32) << 8)
        | ((data[2] as u32) << 16)
        | ((data[3] as u32) << 24);

    // This order `CRC32[0] .. CRC32[7]`
    // gives ~40% better benchmark result for `DualCrc::checksum32`.
    CRC32[0][data[7] as usize]
        ^ CRC32[1][data[6] as usize]
        ^ CRC32[2][data[5] as usize]
        ^ CRC32[3][data[4] as usize]
        ^ CRC32[4][(inverted_crc >> 24) as usize]
        ^ CRC32[5][(inverted_crc >> 16) as usize & 0xFF]
        ^ CRC32[6][(inverted_crc >> 8) as usize & 0xFF]
        ^ CRC32[7][inverted_crc as usize & 0xFF]
}

#[inline(always)]
pub(crate) fn update_inverted_crc64(inverted_crc: u64, byte: u8) -> u64 {
    CRC64[0][(inverted_crc as u8 ^ byte) as usize] ^ (inverted_crc >> 8)
}

/// This is equivalent to calling `update_inverted_crc64` for each byte, but faster.
#[inline(always)]
pub(crate) fn update_inverted_crc64_8bytes(mut inverted_crc: u64, data: &[u8]) -> u64 {
    debug_assert_eq!(data.len(), 8);

    inverted_crc ^= data[0] as u64
        | ((data[1] as u64) << 8)
        | ((data[2] as u64) << 16)
        | ((data[3] as u64) << 24)
        | ((data[4] as u64) << 32)
        | ((data[5] as u64) << 40)
        | ((data[6] as u64) << 48)
        | ((data[7] as u64) << 56);

    // This order `CRC64[7] .. CRC64[0]`
    // gives ~10% better benchmark result for `DualCrc::checksum64`.
    CRC64[7][inverted_crc as usize & 0xFF]
        ^ CRC64[6][(inverted_crc >> 8) as usize & 0xFF]
        ^ CRC64[5][(inverted_crc >> 16) as usize & 0xFF]
        ^ CRC64[4][(inverted_crc >> 24) as usize & 0xFF]
        ^ CRC64[3][(inverted_crc >> 32) as usize & 0xFF]
        ^ CRC64[2][(inverted_crc >> 40) as usize & 0xFF]
        ^ CRC64[1][(inverted_crc >> 48) as usize & 0xFF]
        ^ CRC64[0][(inverted_crc >> 56) as usize]
}

// ======================================================================
// STATIC - PRIVATE / CRATE

// see `build.rs`
include! {env!("GENERATED_TABLES_RS")}

// ======================================================================
// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    use crate::DualCrc;

    #[test]
    fn crc32_table_checksums() {
        let mut crc = DualCrc::new();
        for inner in CRC32 {
            for x in inner {
                crc.update(&x.to_le_bytes());
            }
        }
        // These values have been confirmed with `crc` crate
        assert_eq!(crc.get(), (0x3F85CEA3, 0xC9BC02D60DD946D2));
    }

    #[test]
    fn crc64_table_checksums() {
        let mut crc = DualCrc::new();
        for inner in CRC64 {
            for x in inner {
                crc.update(&x.to_le_bytes());
            }
        }
        // These values have been confirmed with `crc` crate
        assert_eq!(crc.get(), (0x3D345BF2, 0x014ED9B63590C55E));
    }

    #[test]
    fn pow256_32_table_checksums() {
        let mut crc = DualCrc::new();
        for x in POW256_32 {
            crc.update(&x.to_le_bytes());
        }
        // These values have been confirmed with `crc` crate
        assert_eq!(crc.get(), (0xB3683DC1, 0xAB7DB56545FE470F));
    }

    #[test]
    fn pow256_64_table_checksums() {
        let mut crc = DualCrc::new();
        for x in POW256_64 {
            crc.update(&x.to_le_bytes());
        }
        // These values have been confirmed with `crc` crate
        assert_eq!(crc.get(), (0x49BABB74, 0x0F7DE3B7F5984AEF));
    }
}
