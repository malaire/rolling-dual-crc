use crate::{tables, Zeros};

// ======================================================================
// DualCrc - PUBLIC

/// Computes 32-bit `CRC-32C` and/or 64-bit `CRC-64/XZ` checksums in one go
/// or iteratively, with efficient handling of long `0u8` sequences using [`Zeros`].
///
/// # Examples
///
/// Use [`checksum`]/[`checksum32`]/[`checksum64`] to compute checksums in one go:
///
/// ```rust
/// use rolling_dual_crc::DualCrc;
///
/// assert_eq!(DualCrc::checksum32("Hello, world!"), 0xC8A106E5);
/// ```
///
/// Use [`new`] + [`update`]/[`update_with_zeros`] + [`get`]/[`get32`]/[`get64`]
/// to compute checksums iteratively:
///
/// ```rust
/// use rolling_dual_crc::DualCrc;
///
/// let mut crc = DualCrc::new();
/// crc.update("Hello");
/// crc.update(", world!");
/// assert_eq!(crc.get32(), 0xC8A106E5);
/// ```
///
/// See [`Zeros`] for [`update_with_zeros`] example of handling long `0u8` sequences.
///
/// [`Zeros`]: crate::Zeros
/// [`checksum`]: DualCrc::checksum
/// [`checksum32`]: DualCrc::checksum32
/// [`checksum64`]: DualCrc::checksum64
/// [`get`]: DualCrc::get
/// [`get32`]: DualCrc::get32
/// [`get64`]: DualCrc::get64
/// [`new`]: DualCrc::new
/// [`update`]: DualCrc::update
/// [`update_with_zeros`]: DualCrc::update_with_zeros
#[derive(Clone, Copy, Debug, Hash)]
pub struct DualCrc {
    inverted_crc32: u32,
    inverted_crc64: u64,
}

impl DualCrc {
    /// Computes 32-bit `CRC-32C` and 64-bit `CRC-64/XZ` checksums of given data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::DualCrc;
    ///
    /// assert_eq!(DualCrc::checksum("Hello, world!"), (0xC8A106E5, 0x8E59E143665877C4));
    /// ```
    pub fn checksum<T: AsRef<[u8]>>(data: T) -> (u32, u64) {
        (Self::checksum32(&data), Self::checksum64(&data))
    }

    /// Computes 32-bit `CRC-32C` checksum of given data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::DualCrc;
    ///
    /// assert_eq!(DualCrc::checksum32("Hello, world!"), 0xC8A106E5);
    /// ```
    pub fn checksum32<T: AsRef<[u8]>>(data: T) -> u32 {
        #[cfg(feature = "crc32c")]
        return crc32c::crc32c(data.as_ref());

        #[cfg(not(feature = "crc32c"))]
        {
            let mut inverted_crc: u32 = !0;

            let data = data.as_ref();
            let mut pos = 0;
            let mut remaining = data.len();

            while remaining >= 8 {
                inverted_crc =
                    tables::update_inverted_crc32_8bytes(inverted_crc, &data[pos..pos + 8]);
                pos += 8;
                remaining -= 8;
            }

            while remaining > 0 {
                inverted_crc = tables::update_inverted_crc32(inverted_crc, data[pos]);
                pos += 1;
                remaining -= 1;
            }

            !inverted_crc
        }
    }

    /// Computes 64-bit `CRC-64/XZ` checksum of given data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::DualCrc;
    ///
    /// assert_eq!(DualCrc::checksum64("Hello, world!"), 0x8E59E143665877C4);
    /// ```
    pub fn checksum64<T: AsRef<[u8]>>(data: T) -> u64 {
        #[cfg(feature = "crc64fast")]
        {
            let mut crc = crc64fast::Digest::new();
            crc.write(data.as_ref());
            crc.sum64()
        }

        #[cfg(not(feature = "crc64fast"))]
        {
            let mut inverted_crc: u64 = !0;

            let data = data.as_ref();
            let mut pos = 0;
            let mut remaining = data.len();

            while remaining >= 8 {
                inverted_crc =
                    tables::update_inverted_crc64_8bytes(inverted_crc, &data[pos..pos + 8]);
                pos += 8;
                remaining -= 8;
            }

            while remaining > 0 {
                inverted_crc = tables::update_inverted_crc64(inverted_crc, data[pos]);
                pos += 1;
                remaining -= 1;
            }

            !inverted_crc
        }
    }

    /// Returns 32-bit `CRC-32C` and 64-bit `CRC-64/XZ` checksums
    /// of the data processed so far.
    ///
    /// Checksums computation is not reset and can be continued with further data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::DualCrc;
    ///
    /// let mut crc = DualCrc::new();
    /// crc.update("Hello");
    /// // checksums of "Hello"
    /// assert_eq!(crc.get(), (0x81D90E1B, 0x51CF5C3BC87BACC8));
    /// crc.update(", world!");
    /// // checksums of "Hello, world!"
    /// assert_eq!(crc.get(), (0xC8A106E5, 0x8E59E143665877C4));
    /// ```
    #[inline(always)]
    pub fn get(&self) -> (u32, u64) {
        (!self.inverted_crc32, !self.inverted_crc64)
    }

    /// Returns 32-bit `CRC-32C` checksum of the data processed so far.
    ///
    /// Checksums computation is not reset and can be continued with further data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::DualCrc;
    ///
    /// let mut crc = DualCrc::new();
    /// crc.update("Hello");
    /// // checksum of "Hello"
    /// assert_eq!(crc.get32(), 0x81D90E1B);
    /// crc.update(", world!");
    /// // checksum of "Hello, world!"
    /// assert_eq!(crc.get32(), 0xC8A106E5);
    /// ```
    #[inline(always)]
    pub fn get32(&self) -> u32 {
        !self.inverted_crc32
    }

    /// Returns 64-bit `CRC-64/XZ` checksum of the data processed so far.
    ///
    /// Checksums computation is not reset and can be continued with further data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::DualCrc;
    ///
    /// let mut crc = DualCrc::new();
    /// crc.update("Hello");
    /// // checksum of "Hello"
    /// assert_eq!(crc.get64(), 0x51CF5C3BC87BACC8);
    /// crc.update(", world!");
    /// // checksum of "Hello, world!"
    /// assert_eq!(crc.get64(), 0x8E59E143665877C4);
    /// ```
    #[inline(always)]
    pub fn get64(&self) -> u64 {
        !self.inverted_crc64
    }

    /// Begins computation of 32-bit `CRC-32C` and 64-bit `CRC-64/XZ` checksums.
    ///
    /// See [`DualCrc`] for an example.
    pub fn new() -> Self {
        Self {
            inverted_crc32: !0,
            inverted_crc64: !0,
        }
    }

    /// Continues checksums computation with given data.
    ///
    /// See [`DualCrc`] for an example.
    pub fn update<T: AsRef<[u8]>>(&mut self, data: T) {
        #[cfg(feature = "crc32c")]
        {
            self.inverted_crc32 = !crc32c::crc32c_append(!self.inverted_crc32, data.as_ref());
        }

        let data = data.as_ref();
        let mut pos = 0;
        let mut remaining = data.len();

        while remaining >= 8 {
            #[cfg(not(feature = "crc32c"))]
            {
                self.inverted_crc32 =
                    tables::update_inverted_crc32_8bytes(self.inverted_crc32, &data[pos..pos + 8]);
            }

            self.inverted_crc64 =
                tables::update_inverted_crc64_8bytes(self.inverted_crc64, &data[pos..pos + 8]);

            pos += 8;
            remaining -= 8;
        }

        while remaining > 0 {
            #[cfg(not(feature = "crc32c"))]
            {
                self.inverted_crc32 = tables::update_inverted_crc32(self.inverted_crc32, data[pos]);
            }

            self.inverted_crc64 = tables::update_inverted_crc64(self.inverted_crc64, data[pos]);

            pos += 1;
            remaining -= 1;
        }
    }

    /// Continues checksums computation with `0u8` sequence
    /// represented by the given [`Zeros`].
    ///
    /// This is equivalent to [`update`]`(&[0u8; N])`
    /// but more efficient with long sequences.
    ///
    /// Complexity: `Θ(1)` time
    ///
    /// See [`Zeros`] for an example and more details.
    ///
    /// [`update`]: DualCrc::update
    /// [`Zeros`]: crate::Zeros
    #[inline(always)]
    pub fn update_with_zeros(&mut self, zeros: &Zeros) {
        self.inverted_crc32 = zeros.apply_to_inverted_crc32(self.inverted_crc32);
        self.inverted_crc64 = zeros.apply_to_inverted_crc64(self.inverted_crc64);
    }
}

// ======================================================================
// DualCrc - IMPL Default

impl Default for DualCrc {
    fn default() -> Self {
        DualCrc::new()
    }
}

// ======================================================================
// TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Zeros;

    // ============================================================
    // STATIC

    // These values have been confirmed with `crc` crate
    static TESTDATA_0_TO_15: [(&[u8], (u32, u64)); 16] = [
        (b"", (0x00000000, 0x0000000000000000)),
        (b"a", (0xC1D04330, 0x330284772E652B05)),
        (b"ab", (0xE2A22936, 0xBC6573200E84B046)),
        (b"abc", (0x364B3FB7, 0x2CD8094A1A277627)),
        (b"abcd", (0x92C80A31, 0x3C9D28596E5960BA)),
        (b"abcde", (0xC450D697, 0x040BDF58FB0895F2)),
        (b"abcdef", (0x53BCEFF1, 0xD08E9F8545A700F4)),
        (b"abcdefg", (0xE627F441, 0xEC20A3A8CC710E66)),
        (b"abcdefgh", (0x0A9421B7, 0x67B4F30A647A0C59)),
        (b"abcdefghi", (0x2DDC99FC, 0x9966F6C89D56EF8E)),
        (b"abcdefghij", (0xE6599437, 0x32093A2ECD5773F4)),
        (b"abcdefghijk", (0x4EFD1FC6, 0x60B3608067681C40)),
        (b"abcdefghijkl", (0x9B9A33D0, 0x688B14EE46F77982)),
        (b"abcdefghijklm", (0x5FDBF778, 0x82F32A2CBF759130)),
        (b"abcdefghijklmn", (0x64DDA821, 0x7EF7AA715AF9E92E)),
        (b"abcdefghijklmno", (0xBF1A2C62, 0xC84B31ADFD591E7E)),
    ];

    // ============================================================
    // checksum

    #[test]
    fn checksum() {
        for (input, expected) in TESTDATA_0_TO_15 {
            assert_eq!(DualCrc::checksum(input), expected);
        }
    }

    // ============================================================
    // checksum32

    #[test]
    fn checksum32() {
        for (input, expected) in TESTDATA_0_TO_15 {
            assert_eq!(DualCrc::checksum32(input), expected.0);
        }
    }

    // ============================================================
    // checksum64

    #[test]
    fn checksum64() {
        for (input, expected) in TESTDATA_0_TO_15 {
            assert_eq!(DualCrc::checksum64(input), expected.1);
        }
    }

    // ============================================================
    // new

    #[test]
    fn new() {
        let crc = DualCrc::new();
        assert_eq!(crc.get(), (0, 0));
        assert_eq!(crc.get32(), 0);
        assert_eq!(crc.get64(), 0);
    }

    // ============================================================
    // update / get / get32 / get64

    #[test]
    fn update_once_with_get_get32_get64() {
        for (input, expected) in TESTDATA_0_TO_15 {
            let mut crc = DualCrc::new();
            crc.update(input);
            assert_eq!(crc.get(), expected);
            assert_eq!(crc.get32(), expected.0);
            assert_eq!(crc.get64(), expected.1);
        }
    }

    #[test]
    fn update_twice_with_get_get32_get64() {
        let mut crc = DualCrc::new();
        crc.update(b"123456789");

        // "check" values from "Catalogue of parametrised CRC algorithms"
        assert_eq!(crc.get(), (0xE3069283, 0x995DC9BBDF1939FA));
        assert_eq!(crc.get32(), 0xE3069283);
        assert_eq!(crc.get64(), 0x995DC9BBDF1939FA);

        crc.update(b"abc");

        // These values have been confirmed with `crc` crate
        assert_eq!(crc.get(), (0x92A0541A, 0x5A062275250CB126));
        assert_eq!(crc.get32(), 0x92A0541A);
        assert_eq!(crc.get64(), 0x5A062275250CB126);
    }

    // ============================================================
    // update_with_zeros / Zeros

    #[test]
    fn update_with_zeros_mixed() {
        // All checksums here have been confirmed with `crc` crate

        let mut crc = DualCrc::new();
        crc.update(b"abc");
        assert_eq!(crc.get(), (0x364B3FB7, 0x2CD8094A1A277627));
        crc.update_with_zeros(&Zeros::new(123));
        assert_eq!(crc.get(), (0xCEC292F2, 0x6299C03F43E742BE));
        crc.update(b"def");
        assert_eq!(crc.get(), (0x11769AE8, 0xBF7EC305917854C5));
        crc.update_with_zeros(&Zeros::new(456));
        assert_eq!(crc.get(), (0x5B8D8166, 0xA9B8E3BFC470CB4D));
    }

    #[test]
    fn update_with_zeros_0_to_15() {
        // These values have been confirmed with `crc` crate
        const EXPECTED: [(u32, u64); 16] = [
            (0x00000000, 0x0000000000000000),
            (0x527D5351, 0x1FADA17364673F59),
            (0xF16177D2, 0x42104D97514A5A87),
            (0x6064A37A, 0xEAF95FC670D9DB46),
            (0x48674BC7, 0xF4A586351E1B9F4B),
            (0x45727635, 0xCBE4D2DFEE43E035),
            (0x572A7C8A, 0x513429D3B4F4D73E),
            (0xBB3E6A6D, 0xE1A504C8EC57235B),
            (0x8C28B28A, 0xB66A73654282CAC0),
            (0xBBE568A3, 0xB2C1B75F3D613570),
            (0xE3DDF06B, 0xFD05A84623CC7316),
            (0xAAD1B6F8, 0xED9FF03024B86B0B),
            (0x2B60B55D, 0xAF4BC36300BAC460),
            (0xBC5BA5E4, 0x8083830A4EC2CEAE),
            (0x766B37F1, 0x558345CFB3197C49),
            (0x530ED410, 0x3FC1C24BBCAE428D),
        ];

        for n in 0..EXPECTED.len() {
            let mut crc = DualCrc::new();
            crc.update_with_zeros(&Zeros::new(n));
            assert_eq!(crc.get(), EXPECTED[n]);
        }
    }

    #[test]
    fn update_with_zeros_pow2() {
        // These values have been confirmed with `crc` crate
        const EXPECTED: [(u32, u64); 28] = [
            // starting from 2^4 as lower values are tested in `update_with_zeros_0_to_15`
            (0x42709AEA, 0xE9A13F17FB6A2363), // 2^4 `0u8`:s
            (0x8A9136AA, 0xC95AF8617CD5330C), // 2^5 `0u8`:s
            (0x03C8EB67, 0xDE547AA516302402), // ...
            (0x082764DB, 0xCF856BED6850AD3F),
            (0xB872B190, 0xD0D52C4CE217CEDC),
            (0x30FCEDC0, 0x6992EB22AC5BFC6C),
            (0xEEAEDE7C, 0xC37863972069270C),
            (0xA489834F, 0x38FB68182427E347),
            (0x98F94189, 0x26D3D39425EAF0A5),
            (0x90444623, 0xC7E021A7A1A6DD3A),
            (0x94640B85, 0x0C8AE2138D0DB1A7),
            (0xBC43BAAD, 0x9B3690A319DE92D5),
            (0x72C0C4A4, 0x26AF09CA494F655E),
            (0x5D87814F, 0x7E0B9C545BC6F8EB),
            (0xF032BCF3, 0x261BDF3D299838FC),
            (0xC253E960, 0x233D8C9901440F63),
            (0x14298C12, 0x606B70A23EBAF6C2),
            (0x6CDF7ABE, 0x1DFE9186665A53B6),
            (0xBC29E3A2, 0xDB6109D27C456C6B),
            (0x1E453952, 0xD3184F3ACEE02B2D),
            (0xA3AB8542, 0x20FECDFFF603E3BE),
            (0x7386EDFC, 0xB69357BEC5C5F73B),
            (0x32456B5D, 0x5CC3D936122D1C95),
            (0x61AF04DD, 0x916FE266C23704B8),
            (0x02F63B78, 0x774F05E159A49DA7),
            (0x038D26C4, 0x633566127F604E40),
            (0x036E6F75, 0x310CCD5B843CC70C),
            (0x527D5351, 0xF15374CE0B53F6C1), // 2^31 `0u8`:s
        ];

        for n in 0..EXPECTED.len() {
            let mut crc = DualCrc::new();
            crc.update_with_zeros(&Zeros::new(2usize.pow(n as u32 + 4)));
            assert_eq!(crc.get(), EXPECTED[n]);
        }
    }

    #[test]
    fn update_with_zeros_u32max() {
        let mut crc = DualCrc::new();
        crc.update_with_zeros(&Zeros::new(u32::MAX as usize));

        // These values have been confirmed with `crc` crate
        assert_eq!(crc.get(), (0x527D5351, 0xFE7E66DF9D7120E1));
    }

    #[test]
    fn update_with_zeros_u64max() {
        if usize::BITS >= u64::BITS {
            let mut crc = DualCrc::new();
            crc.update_with_zeros(&Zeros::new(u64::MAX as usize));

            // These values have NOT been confirmed against some other program
            // as I don't know any other program which can calculate these,
            // but have been confirmed internally against loop of
            // 0x1_0000_0001 updates of 0xFFFF_FFFF zeroes each.
            assert_eq!(crc.get(), (0x6064A37A, 0xC7880A0C13D298F1));
        }
    }
}
