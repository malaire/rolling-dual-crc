use crate::tables;

// ======================================================================
// CONST - PRIVATE

// CRC-32C (Castagnoli)
const POLYNOMIAL_32: u32 = 0x1EDC6F41;

// CRC-64/XZ
const POLYNOMIAL_64: u64 = 0x42F0E1EBA9EA3693;

// ======================================================================
// Zeros - PUBLIC

/// Efficient representation of a `0u8` sequence for [`DualCrc::update_with_zeros`].
///
/// # Examples
///
/// Compute checksum of `"Hello, World!"` padded to 4 kiB by `0u8`:
///
/// ```rust
/// use rolling_dual_crc::{DualCrc, Zeros};
///
/// let data = "Hello, world!";
/// let padding_size = 4096 - data.as_bytes().len();
///
/// let mut crc = DualCrc::new();
/// crc.update(data);
/// // This is equivalent to `crc.update(&vec![0u8; padding_size])`
/// // but more efficient with long sequences.
/// crc.update_with_zeros(&Zeros::new(padding_size));
/// assert_eq!(crc.get32(), 0xCED9AB00);
/// ```
///
/// # Benchmarks
///
/// - These benchmarks are from `cargo bench zeros`
///   with 3.4 GHz i5-3570K (Ivy Bridge, 3rd gen.).
/// - See [crate index](crate#benchmarks) for main benchmarks.
///
/// In these benchmarks [`update`] and [`update_with_zeros`] use sequence lengths
/// 64/256/1024 while [`Zeros::new`] uses sequence lengths 63/64/255/256/1023/1024
/// because its speed depends on the number of 1-bits in the sequence length.
/// Low end (best) timings are with `2^n` while high end (worst) timings are with `2^n-1`.
///
/// |Method                        |ns / \~64 B |ns / \~256 B|ns / \~1024 B|complexity     |
/// |------------------------------|------------|------------|-------------|---------------|
/// |[`DualCrc::update`]           |**65**      |260         |1000         |`Θ(n)`         |
/// |[`DualCrc::update_with_zeros`]|90          |90          |90           |`Θ(1)`         |
/// |[`Zeros::new`]                |3 - 430     |3 - 600     |**3 - 770**  |`Θ(one_bits n)`|
///
/// - For short sequences [`update`] is always faster than [`update_with_zeros`].
/// - For long sequences [`update_with_zeros`] with [`Zeros::new`] is always faster.
/// - For intermediate sequences situation depends on exact sequence length
///   and also whether new [`Zeros`] is created for each [`update_with_zeros`] call,
///   or if same [`Zeros`] is used with multiple [`update_with_zeros`] calls,
///   amortizing its creation time.
///
/// [`DualCrc::update`]: crate::DualCrc::update
/// [`DualCrc::update_with_zeros`]: crate::DualCrc::update_with_zeros
/// [`update`]: crate::DualCrc::update
/// [`update_with_zeros`]: crate::DualCrc::update_with_zeros
#[derive(Clone, Copy, Debug, Hash)]
pub struct Zeros {
    factor32: u32,
    factor64: u64,
}

impl Zeros {
    /// Creates a new [`Zeros`] which represents a sequence of `byte_count` `0u8`:s.
    ///
    /// Complexity: `Θ(one_bits n)` time, `Θ(1)` space
    ///
    /// See [`Zeros`] for example.
    pub fn new(byte_count: usize) -> Self {
        Self {
            factor32: pow256_32(byte_count),
            factor64: pow256_64(byte_count),
        }
    }
}

// ======================================================================
// Zeros - CRATE

impl Zeros {
    #[inline(always)]
    pub(crate) fn apply_to_inverted_crc32(&self, inverted_crc: u32) -> u32 {
        mul32(inverted_crc.reverse_bits(), self.factor32).reverse_bits()
    }

    #[inline(always)]
    pub(crate) fn apply_to_inverted_crc64(&self, inverted_crc: u64) -> u64 {
        mul64(inverted_crc.reverse_bits(), self.factor64).reverse_bits()
    }
}

// ======================================================================
// FUNCTIONS - PRIVATE

/// Computes `a * b` in Galois field.
///
/// Complexity: `Θ(1)`
fn mul32(a: u32, mut b: u32) -> u32 {
    let mut product = 0;
    for _ in 0..32 {
        product = (product << 1) ^ (0u32.wrapping_sub(product >> 31) & POLYNOMIAL_32);
        product ^= (0u32.wrapping_sub(b >> 31)) & a;
        b <<= 1;
    }
    product
}

/// Computes `a * b` in Galois field.
///
/// Complexity: `Θ(1)`
fn mul64(a: u64, mut b: u64) -> u64 {
    let mut product = 0;
    for _ in 0..64 {
        product = (product << 1) ^ (0u64.wrapping_sub(product >> 63) & POLYNOMIAL_64);
        product ^= (0u64.wrapping_sub(b >> 63)) & a;
        b <<= 1;
    }
    product
}

/// Computes `256 ** power` in Galois field using exponentiation by squaring.
///
/// Complexity: `Θ(one_bits n)`
fn pow256_32(mut power: usize) -> u32 {
    if power == 0 {
        return 1;
    }

    // LOWEST ONE BIT

    let mut pos = power.trailing_zeros() as usize;
    let mut result = tables::POW256_32[pos];
    pos += 1;
    power >>= pos;

    // OTHER ONE BITS

    while power > 0 {
        if power & 1 == 1 {
            result = mul32(result, tables::POW256_32[pos]);
        }
        pos += 1;
        power >>= 1;
    }

    result
}

/// Computes `256 ** power` in Galois field using exponentiation by squaring.
///
/// Complexity: `Θ(one_bits n)`
fn pow256_64(mut power: usize) -> u64 {
    if power == 0 {
        return 1;
    }

    // LOWEST ONE BIT

    let mut pos = power.trailing_zeros() as usize;
    let mut result = tables::POW256_64[pos];
    pos += 1;
    power >>= pos;

    // OTHER ONE BITS

    while power > 0 {
        if power & 1 == 1 {
            result = mul64(result, tables::POW256_64[pos]);
        }
        pos += 1;
        power >>= 1;
    }

    result
}

/*
/// Computes `256 ** power` in Galois field using exponentiation by squaring.
///
/// Complexity: `Θ(log2 n + one_bits n)`
fn pow256_32_slow(mut power: usize) -> u32 {
    let mut result: u32 = 1;
    let mut square: u32 = 256;
    while power > 0 {
        if power & 1 == 1 {
            result = mul32(result, square);
        }
        square = mul32(square, square);
        power >>= 1;
    }
    result
}

/// Computes `256 ** power` in Galois field using exponentiation by squaring.
///
/// Complexity: `Θ(log2 n + one_bits n)`
fn pow256_64_slow(mut power: usize) -> u64 {
    let mut result: u64 = 1;
    let mut square: u64 = 256;
    while power > 0 {
        if power & 1 == 1 {
            result = mul64(result, square);
        }
        square = mul64(square, square);
        power >>= 1;
    }
    result
}
*/

// ======================================================================
// TESTS

// `Zeros` is tested with `DualCrc::update_with_zeros` in `dual_crc.rs`
