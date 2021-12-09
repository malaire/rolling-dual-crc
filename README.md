# rolling-dual-crc

A library for computing 32-bit `CRC-32C` and 64-bit `CRC-64/XZ` checksums, featuring:

- [`RollingDualCrc`] for computing checksums in a rolling window
  that moves through the input data.
- [`DualCrc`] for computing checksums in one go or iteratively.
    - [`Zeros`] for efficient handling of long `0u8` sequences.
- Software implementation using lookup tables.
- Optional hardware acceleration for some operations
  using [`crc32c`] and [`crc64fast`] crates.
- No `unsafe` by default.
- No dependencies by default.

[`crc32c`]: https://crates.io/crates/crc32c
[`crc64fast`]: https://crates.io/crates/crc64fast

## Supported CRC algorithms

- 32-bit `CRC-32C (Castagnoli)`
    - known as `CRC_32_ISCSI` in [`crc` crate]
      and `CRC-32/ISCSI` in [Catalogue of parametrised CRC algorithms]
- 64-bit `CRC-64/XZ`
    - known as `CRC_64_XZ` in [`crc` crate]
      and `CRC-64/XZ` in [Catalogue of parametrised CRC algorithms]
    - often misidentified as `CRC-64/ECMA-182`

[`crc` crate]: https://crates.io/crates/crc
[Catalogue of parametrised CRC algorithms]: https://reveng.sourceforge.io/crc-catalogue/all.htm

## Usage

### Compute checksums in a rolling window

[`RollingDualCrc::new`] sets the size of the rolling window and its initial contents.
After that each [`roll`] appends the given byte to the window
and removes first byte of the window, rolling the window forward one byte
and then recomputes checksums for the new window.

[`roll`] is a fast constant time `Θ(1)` operation
which doesn't depend on the size of the window.

```rust
use rolling_dual_crc::RollingDualCrc;

let mut crc = RollingDualCrc::new("abc");

// checksums of "abc"
assert_eq!(crc.get32(), 0x364B3FB7);
assert_eq!(crc.get64(), 0x2CD8094A1A277627);

crc.roll(b'd');
// checksums of "bcd"
assert_eq!(crc.get32(), 0x1B0D0358);
assert_eq!(crc.get64(), 0x0557EA6AA1219070);

crc.roll(b'e');
// checksums of "cde"
assert_eq!(crc.get32(), 0x364ADB60);
assert_eq!(crc.get64(), 0xB534844A0AD06B72);
```

### Compute checksums in one go

```rust
use rolling_dual_crc::DualCrc;

assert_eq!(DualCrc::checksum32("Hello, world!"), 0xC8A106E5);
assert_eq!(DualCrc::checksum64("Hello, world!"), 0x8E59E143665877C4);
```

### Compute checksums iteratively

```rust
use rolling_dual_crc::DualCrc;

let mut crc = DualCrc::new();
crc.update("Hello");
crc.update(", world!");
assert_eq!(crc.get32(), 0xC8A106E5);
assert_eq!(crc.get64(), 0x8E59E143665877C4);
```

See [`Zeros`] for an example of handling long `0u8` sequences.

## Feature flags

Feature flags enable hardware acceleration for some checksum calculations.
While this crate itself doesn't use any `unsafe` code, these dependencies
do use `unsafe` since that is necessary for hardware acceleration.

- `crc32c`
    - Use [`crc32c` crate] for some `CRC-32C` computations.
- `crc64fast`
    - Use [`crc64fast` crate] for some `CRC-64/XZ` computations.
- `fast`
    - Use both of those crates.

Methods/functions which support hardware acceleration:

| Method / Function       | `crc32c` | `crc64fast` |
| ----------------------- | -------- | ----------- |
| [`DualCrc::checksum32`] | X        | -           |
| [`DualCrc::checksum64`] | -        | X           |
| [`DualCrc::checksum`]   | X        | X           |
| [`DualCrc::update`]     | X        | -           |
| [`RollingDualCrc::new`] | X        | X           |

[`crc32c` crate]: https://crates.io/crates/crc32c
[`crc64fast` crate]: https://crates.io/crates/crc64fast

## Benchmarks

- These benchmarks are from `cargo bench main` and `cargo bench main --features fast`
  with 3.4 GHz i5-3570K (Ivy Bridge, 3rd gen.).
- See [`Zeros`] for advanced benchmarks of handling long `0u8` sequences.

### Compute checksums in a rolling window

| Method / Function        | window size | ns        | MiB/s | ns [fast] | MiB/s [fast] |
| ------------------------ | ----------- | --------- | ----- | --------- | ------------ |
| [`RollingDualCrc::new`]  | 1 kiB       | 26 000    | 38    | *28 000*  | *35*         |
| [`RollingDualCrc::new`]  | 32 kiB      | 58 000    | 540   | *40 000*  | *790*        |
| [`RollingDualCrc::new`]  | 1024 kiB    | 1 100 000 | 920   | *430 000* | *2300*       |
| [`RollingDualCrc::roll`] | 1 kiB       | 4         | 240   | *4*       | *240*        |
| [`RollingDualCrc::roll`] | 32 kiB      | 4         | 240   | *4*       | *240*        |
| [`RollingDualCrc::roll`] | 1024 kiB    | 4         | 240   | *4*       | *240*        |

### Compute checksums in one go / iteratively

| Method / Function       | data size | ns   | MiB/s | ns [fast] | MiB/s [fast] |
| ----------------------- | --------- |----- | ----- | --------- | ------------ |
| [`DualCrc::checksum32`] | 1 kiB     | 400  | 2400  | *66*      | *15000*      |
| [`DualCrc::checksum64`] | 1 kiB     | 580  | 1700  | *310*     | *3200*       |
| [`DualCrc::checksum`]   | 1 kiB     | 1000 | 980   | *370*     | *2600*       |
| [`DualCrc::update`]     | 1 kiB     | 1000 | 980   | *660*     | *1500*       |

[fast]: #feature-flags

## Lookup table sizes

Default implementation (i.e. without any [feature flags])
processes 1 or 8 bytes at a time using lookup tables.

| Method / Function              | bytes/iter | Total table size | C32 | C64 | Roll | Zeros |
| ------------------------------ | ---------- | -----------------| --- | --- | ---- | ----- |
| [`DualCrc::checksum32`]        | 8          | 8 kiB            | 8x  | -   | -    | -     |
| [`DualCrc::checksum64`]        | 8          | 16 kiB           | -   | 8x  | -    | -     |
| [`DualCrc::checksum`]          | 8          | 24 kiB           | 8x  | 8x  | -    | -     |
| [`DualCrc::update`]            | 8          | 24 kiB           | 8x  | 8x  | -    | -     |
| [`RollingDualCrc::new`]        | 8          | 27.75 kiB        | 8x  | 8x  | X*   | X     |
| [`RollingDualCrc::roll`]       | 1          | 6 kiB            | 1x  | 1x  | X    | -     |
| [`RollingDualCrc::roll_slice`] | 1          | 6 kiB            | 1x  | 1x  | X    | -     |
| [`Zeros::new`]                 | N/A        | 0.75 kiB         | -   | -   | -    | X     |

- `C32`: global 8 * 1 kiB tables for computing `CRC-32C`
- `C64`: global 8 * 2 kiB tables for computing `CRC-64/XZ`
- `Roll`: local 1 + 2 kiB tables for rolling `CRC-32C` and `CRC-64/XZ`
- `Zeros`: global 0.25 + 0.50 kiB tables for creating [`Zeros`]

\*) creates the local tables

[feature flags]: #feature-flags

## Safety

This crate itself doesn't use any `unsafe` code.
This is enforced by `#![forbid(unsafe_code)]`.

If you enable hardware acceleration with [feature flags],
then those dependencies do use `unsafe` code.

## Credits

This crate is based on

- public domain [rolling-crc] by Igor Pavlov, Bulat Ziganshin and Bart Massey
- [stackoverflow answer] by rcgldr about efficiently appending 0s to CRC
- slicing-by-8 examples at [Fast CRC32] by Stephan Brumme

[rolling-crc]: http://github.com/BartMassey/rolling-crc
[stackoverflow answer]: https://stackoverflow.com/a/62922203/6600109
[Fast CRC32]: https://create.stephan-brumme.com/crc32/

[`DualCrc`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.DualCrc.html
[`DualCrc::checksum`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.DualCrc.html#method.checksum
[`DualCrc::checksum32`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.DualCrc.html#method.checksum32
[`DualCrc::checksum64`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.DualCrc.html#method.checksum64
[`DualCrc::update`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.DualCrc.html#method.update
[`RollingDualCrc`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.RollingDualCrc.html
[`RollingDualCrc::new`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.RollingDualCrc.html#method.new
[`RollingDualCrc::roll`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.RollingDualCrc.html#method.roll
[`RollingDualCrc::roll_slice`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.RollingDualCrc.html#method.roll_slice
[`Zeros`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.Zeros.html
[`Zeros::new`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.Zeros.html#method.new
[`roll`]: https://docs.rs/rolling-dual-crc/0.1.0/rolling_dual_crc/struct.RollingDualCrc.html#method.roll
