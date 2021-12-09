use crate::{tables, DualCrc, Zeros};

// ======================================================================
// RollingDualCrc - PUBLIC

/// Computes 32-bit `CRC-32C` and 64-bit `CRC-64/XZ` checksums
/// in a rolling window that moves through the input data.
///
/// # Examples
///
/// Compute checksums of 3-byte windows of `"abcde"`,
/// i.e. `"abc"`, `"bcd"` and `"cde"`.
///
/// ```rust
/// use rolling_dual_crc::RollingDualCrc;
///
/// let mut crc = RollingDualCrc::new("abc");
///
/// // checksum of "abc"
/// assert_eq!(crc.get32(), 0x364B3FB7);
///
/// crc.roll(b'd');
/// // checksum of "bcd"
/// assert_eq!(crc.get32(), 0x1B0D0358);
///
/// crc.roll(b'e');
/// // checksum of "cde"
/// assert_eq!(crc.get32(), 0x364ADB60);
/// ```
#[derive(Clone, Debug)]
pub struct RollingDualCrc {
    inverted_crc32: u32,
    inverted_crc64: u64,

    start_pos: usize,
    window_size: usize,
    data: Vec<u8>,

    table32: Box<[u32; 256]>,
    table64: Box<[u64; 256]>,
}

impl RollingDualCrc {
    /// Returns 32-bit `CRC-32C` and 64-bit `CRC-64/XZ` checksums
    /// of the current window.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::RollingDualCrc;
    ///
    /// let mut crc = RollingDualCrc::new("abc");
    /// crc.roll(b'd');
    /// // checksums of "bcd"
    /// assert_eq!(crc.get(), (0x1B0D0358, 0x0557EA6AA1219070));
    /// ```
    #[inline(always)]
    pub fn get(&self) -> (u32, u64) {
        (!self.inverted_crc32, !self.inverted_crc64)
    }

    /// Returns 32-bit `CRC-32C` checksum of the current window.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::RollingDualCrc;
    ///
    /// let mut crc = RollingDualCrc::new("abc");
    /// crc.roll(b'd');
    /// // checksum of "bcd"
    /// assert_eq!(crc.get32(), 0x1B0D0358);
    /// ```
    #[inline(always)]
    pub fn get32(&self) -> u32 {
        !self.inverted_crc32
    }

    /// Returns 64-bit `CRC-64/XZ` checksum of the current window.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::RollingDualCrc;
    ///
    /// let mut crc = RollingDualCrc::new("abc");
    /// crc.roll(b'd');
    /// // checksum of "bcd"
    /// assert_eq!(crc.get64(), 0x0557EA6AA1219070);
    /// ```
    #[inline(always)]
    pub fn get64(&self) -> u64 {
        !self.inverted_crc64
    }

    /// Begins computation of 32-bit `CRC-32C` and 64-bit `CRC-64/XZ`
    /// rolling checksums.
    ///
    /// - Sets `window_size` to size of the given initial window.
    ///   (`window_size` remains same during rolling.)
    /// - Allocates and initializes buffer of
    ///   `window_size` bytes for the window contents.
    /// - Allocates and initializes local lookup tables (3 kiB total).
    /// - Computes checksums of initial window.
    ///
    /// # Panics
    ///
    /// Panics if `initial_window` is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::RollingDualCrc;
    ///
    /// let mut crc = RollingDualCrc::new("abc");
    /// // checksum of "abc"
    /// assert_eq!(crc.get32(), 0x364B3FB7);
    /// ```
    pub fn new<T: AsRef<[u8]>>(initial_window: T) -> Self {
        let initial_window = initial_window.as_ref();
        let window_size = initial_window.len();

        if window_size == 0 {
            panic!("initial_window is empty");
        }

        let (crc32, crc64) = DualCrc::checksum(initial_window);
        let (table32, table64) = Self::build_tables(window_size);

        Self {
            inverted_crc32: !crc32,
            inverted_crc64: !crc64,

            start_pos: 0,
            window_size,
            data: initial_window.to_vec(),

            table32,
            table64,
        }
    }

    /// Rolls window forward one byte.
    ///
    /// - Appends the given byte to the window.
    /// - Removes first byte of the window.
    /// - Recomputes checksums for the new window.
    ///
    /// This is a fast constant time `Θ(1)` operation
    /// which doesn't depend on the size of the window.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::RollingDualCrc;
    ///
    /// let mut crc = RollingDualCrc::new("abc");
    /// crc.roll(b'd');
    /// // checksum of "bcd"
    /// assert_eq!(crc.get32(), 0x1B0D0358);
    /// ```
    #[inline(always)]
    pub fn roll(&mut self, data: u8) {
        self.inverted_crc32 = tables::update_inverted_crc32(self.inverted_crc32, data)
            ^ self.table32[self.data[self.start_pos] as usize];

        self.inverted_crc64 = tables::update_inverted_crc64(self.inverted_crc64, data)
            ^ self.table64[self.data[self.start_pos] as usize];

        self.data[self.start_pos] = data;
        self.start_pos += 1;
        if self.start_pos == self.window_size {
            self.start_pos = 0;
        }
    }

    /// Rolls window forward.
    ///
    /// This is equivalent to calling [`roll`] for each byte of the given slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rolling_dual_crc::RollingDualCrc;
    ///
    /// let mut crc = RollingDualCrc::new("abc");
    /// crc.roll_slice("de");
    /// // checksum of "cde"
    /// assert_eq!(crc.get32(), 0x364ADB60);
    /// ```
    ///
    /// [`roll`]: RollingDualCrc::roll
    pub fn roll_slice<T: AsRef<[u8]>>(&mut self, data: T) {
        for byte in data.as_ref() {
            self.roll(*byte);
        }
    }
}

// ======================================================================
// RollingDualCrc - PRIVATE

impl RollingDualCrc {
    fn build_tables(window_size: usize) -> (Box<[u32; 256]>, Box<[u64; 256]>) {
        let mut table32 = Box::new([0u32; 256]);
        let mut table64 = Box::new([0u64; 256]);

        let zeros = Zeros::new(window_size);

        let mut zero_crc = DualCrc::new();
        zero_crc.update_with_zeros(&zeros);

        for byte in 0..=255 {
            let mut byte_crc = DualCrc::new();
            byte_crc.update(&[byte]);
            byte_crc.update_with_zeros(&zeros);
            table32[byte as usize] = byte_crc.get32() ^ zero_crc.get32();
            table64[byte as usize] = byte_crc.get64() ^ zero_crc.get64();
        }

        (table32, table64)
    }
}

// ======================================================================
// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // PANICS

    #[test]
    #[should_panic]
    fn empty_initial_window() {
        RollingDualCrc::new(&[]);
    }

    // ============================================================
    // new / get / get32 / get64

    #[test]
    fn new_get_get32_get64() {
        let crc = RollingDualCrc::new("123456789");

        // "check" values from "Catalogue of parametrised CRC algorithms"
        assert_eq!(crc.get(), (0xE3069283, 0x995DC9BBDF1939FA));
        assert_eq!(crc.get32(), 0xE3069283);
        assert_eq!(crc.get64(), 0x995DC9BBDF1939FA);
    }

    // ============================================================
    // roll

    #[test]
    fn roll_1_3() {
        // All checksums here have been confirmed with `crc` crate
        let mut crc = RollingDualCrc::new("a");
        assert_eq!(crc.get(), (0xC1D04330, 0x330284772E652B05));
        crc.roll(b'b');
        assert_eq!(crc.get(), (0xD280B0C4, 0x74A8FE9E8582D431));
        crc.roll(b'c');
        assert_eq!(crc.get(), (0x20EB33C7, 0xC786B22086258B5E));
    }

    #[test]
    fn roll_3_5() {
        // All checksums here have been confirmed with `crc` crate
        let mut crc = RollingDualCrc::new("abc");
        assert_eq!(crc.get(), (0x364B3FB7, 0x2CD8094A1A277627));
        crc.roll(b'd');
        assert_eq!(crc.get(), (0x1B0D0358, 0x0557EA6AA1219070));
        crc.roll(b'e');
        assert_eq!(crc.get(), (0x364ADB60, 0xB534844A0AD06B72));
        crc.roll(b'f');
        assert_eq!(crc.get(), (0x4248D48A, 0x1B4421C40BF0643A));
        crc.roll(b'g');
        assert_eq!(crc.get(), (0x21856D6E, 0x6A5A06867C4C4589));
        crc.roll(b'h');
        assert_eq!(crc.get(), (0x861A094E, 0xB47462AF38541FB8));
    }

    #[test]
    fn roll_1k() {
        // All checksums in `roll_1k-output` have been confirmed with `crc` crate

        const WINDOW_SIZE: usize = 1024;

        // random data
        let input = include_bytes!("testdata/roll_1k-input");
        // checksums of each 1 kiB window
        // - [ u32_le, u64_le, u32_le, u64_le, ... ]
        let output = include_bytes!("testdata/roll_1k-output");

        let mut crc = RollingDualCrc::new(&input[..WINDOW_SIZE]);
        for pos in 0..input.len() - WINDOW_SIZE {
            if pos != 0 {
                crc.roll(input[pos + WINDOW_SIZE - 1]);
            }

            assert_eq!(
                crc.get32(),
                u32::from_le_bytes(output[pos * 12..pos * 12 + 4].try_into().unwrap())
            );
            assert_eq!(
                crc.get64(),
                u64::from_le_bytes(output[pos * 12 + 4..pos * 12 + 12].try_into().unwrap())
            );
        }
    }

    // ============================================================
    // roll_slice

    #[test]
    fn roll_slice_empty() {
        // All checksums here have been confirmed with `crc` crate
        let mut crc = RollingDualCrc::new("abc");
        assert_eq!(crc.get(), (0x364B3FB7, 0x2CD8094A1A277627));
        crc.roll_slice(&[]);
        assert_eq!(crc.get(), (0x364B3FB7, 0x2CD8094A1A277627));
    }

    #[test]
    fn roll_slice_smaller_than_window() {
        // All checksums here have been confirmed with `crc` crate
        let mut crc = RollingDualCrc::new("abcde");
        assert_eq!(crc.get(), (0xC450D697, 0x040BDF58FB0895F2));
        crc.roll_slice("fgh");
        assert_eq!(crc.get(), (0xB5546D6F, 0xEA828BEC74913B8F));
        crc.roll_slice("ijk");
        assert_eq!(crc.get(), (0x3289B67D, 0x9E7D97E675837DE2));
        crc.roll_slice("lmn");
        assert_eq!(crc.get(), (0x399B0EF4, 0x67906E3C3EC81347));
    }

    #[test]
    fn roll_slice_same_size_than_window() {
        // All checksums here have been confirmed with `crc` crate
        let mut crc = RollingDualCrc::new("abc");
        assert_eq!(crc.get(), (0x364B3FB7, 0x2CD8094A1A277627));
        crc.roll_slice("def");
        assert_eq!(crc.get(), (0x4248D48A, 0x1B4421C40BF0643A));
        crc.roll_slice("ghi");
        assert_eq!(crc.get(), (0x364912CE, 0x1435316184304F5D));
    }

    #[test]
    fn roll_slice_larger_than_window() {
        // All checksums here have been confirmed with `crc` crate
        let mut crc = RollingDualCrc::new("abc");
        assert_eq!(crc.get(), (0x364B3FB7, 0x2CD8094A1A277627));
        crc.roll_slice("defgh");
        assert_eq!(crc.get(), (0x861A094E, 0xB47462AF38541FB8));
        crc.roll_slice("ijklm");
        assert_eq!(crc.get(), (0x19D67ED2, 0x38309BD2C6060C2E));
    }
}
