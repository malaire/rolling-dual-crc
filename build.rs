use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
};

use regex::{Captures, Regex};

// ======================================================================
// CONST

// CRC-32C (Castagnoli)
const POLYNOMIAL_32: u32 = 0x1EDC6F41;
const REVERSED_POLYNOMIAL_32: u32 = 0x82F63B78;

// CRC-64/XZ
const POLYNOMIAL_64: u64 = 0x42F0E1EBA9EA3693;
const REVERSED_POLYNOMIAL_64: u64 = 0xC96C5795D7870F42;

const USIZE_BITS: usize = usize::BITS as usize;

// ======================================================================
// MAIN

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = fs::canonicalize(env::var("OUT_DIR")?)?;

    // README

    println!("cargo:rerun-if-changed=README.md");
    let readme = fs::read_to_string("README.md")?;
    let readme = rustdocify_readme(&readme)?;
    fs::write(out_dir.join("README-rustdocified.md"), readme)?;

    // TABLES

    let tables_path = out_dir.join("tables.rs");
    let mut w = BufWriter::new(File::create(&tables_path)?);
    write_crc32_table(&mut w)?;
    write_crc64_table(&mut w)?;
    write_pow256_32_table(&mut w)?;
    write_pow256_64_table(&mut w)?;
    w.flush()?;

    // See: https://github.com/rust-lang/rust/issues/75075
    println!(
        "cargo:rustc-env=GENERATED_TABLES_RS={}",
        tables_path.display()
    );

    Ok(())
}

// ======================================================================
// CRC TABLES

fn write_crc32_table<W: Write>(w: &mut W) -> Result<(), Box<dyn Error>> {
    let mut table = [[0u32; 256]; 8];

    for byte in 0..256 {
        let mut crc = byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0u32.wrapping_sub(crc & 1) & REVERSED_POLYNOMIAL_32);
        }
        table[0][byte] = crc;
    }

    for byte in 0..256 {
        table[1][byte] = (table[0][byte] >> 8) ^ table[0][table[0][byte] as usize & 0xFF];
        table[2][byte] = (table[1][byte] >> 8) ^ table[0][table[1][byte] as usize & 0xFF];
        table[3][byte] = (table[2][byte] >> 8) ^ table[0][table[2][byte] as usize & 0xFF];
        table[4][byte] = (table[3][byte] >> 8) ^ table[0][table[3][byte] as usize & 0xFF];
        table[5][byte] = (table[4][byte] >> 8) ^ table[0][table[4][byte] as usize & 0xFF];
        table[6][byte] = (table[5][byte] >> 8) ^ table[0][table[5][byte] as usize & 0xFF];
        table[7][byte] = (table[6][byte] >> 8) ^ table[0][table[6][byte] as usize & 0xFF];
    }

    writeln!(w, "static CRC32: [[u32; 256]; 8] = [")?;
    for inner in table {
        writeln!(w, "[")?;
        for (byte, x) in inner.iter().enumerate() {
            write!(w, "0x{:08X}, ", x)?;
            if byte % 8 == 7 {
                writeln!(w)?;
            }
        }
        writeln!(w, "],")?;
    }
    writeln!(w, "];")?;

    Ok(())
}

fn write_crc64_table<W: Write>(w: &mut W) -> Result<(), Box<dyn Error>> {
    let mut table = [[0u64; 256]; 8];

    for byte in 0..256 {
        let mut crc = byte as u64;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0u64.wrapping_sub(crc & 1) & REVERSED_POLYNOMIAL_64);
        }
        table[0][byte] = crc;
    }

    for byte in 0..256 {
        table[1][byte] = (table[0][byte] >> 8) ^ table[0][table[0][byte] as usize & 0xFF];
        table[2][byte] = (table[1][byte] >> 8) ^ table[0][table[1][byte] as usize & 0xFF];
        table[3][byte] = (table[2][byte] >> 8) ^ table[0][table[2][byte] as usize & 0xFF];
        table[4][byte] = (table[3][byte] >> 8) ^ table[0][table[3][byte] as usize & 0xFF];
        table[5][byte] = (table[4][byte] >> 8) ^ table[0][table[4][byte] as usize & 0xFF];
        table[6][byte] = (table[5][byte] >> 8) ^ table[0][table[5][byte] as usize & 0xFF];
        table[7][byte] = (table[6][byte] >> 8) ^ table[0][table[6][byte] as usize & 0xFF];
    }

    writeln!(w, "static CRC64: [[u64; 256]; 8] = [")?;
    for inner in table {
        writeln!(w, "[")?;
        for (byte, x) in inner.iter().enumerate() {
            write!(w, "0x{:016X}, ", x)?;
            if byte % 4 == 3 {
                writeln!(w)?;
            }
        }
        writeln!(w, "],")?;
    }
    writeln!(w, "];")?;

    Ok(())
}

// ======================================================================
// ZEROS TABLES

// Computes `a * b` in Galois field.
// - copied from zeros.rs
fn mul32(a: u32, mut b: u32) -> u32 {
    let mut product = 0;
    for _ in 0..32 {
        product = (product << 1) ^ (0u32.wrapping_sub(product >> 31) & POLYNOMIAL_32);
        product ^= (0u32.wrapping_sub(b >> 31)) & a;
        b <<= 1;
    }
    product
}

// Computes `a * b` in Galois field.
// - copied from zeros.rs
fn mul64(a: u64, mut b: u64) -> u64 {
    let mut product = 0;
    for _ in 0..64 {
        product = (product << 1) ^ (0u64.wrapping_sub(product >> 63) & POLYNOMIAL_64);
        product ^= (0u64.wrapping_sub(b >> 63)) & a;
        b <<= 1;
    }
    product
}

fn write_pow256_32_table<W: Write>(w: &mut W) -> Result<(), Box<dyn Error>> {
    let mut table = [0u32; USIZE_BITS];

    table[0] = 256;
    for n in 1..USIZE_BITS {
        table[n] = mul32(table[n - 1], table[n - 1]);
    }

    writeln!(w, "pub(crate) static POW256_32: [u32; {}] = [", USIZE_BITS)?;
    for (n, x) in table.iter().enumerate() {
        write!(w, "0x{:08X}, ", x)?;
        if n % 8 == 7 {
            writeln!(w)?;
        }
    }
    writeln!(w, "\n];")?;

    Ok(())
}

fn write_pow256_64_table<W: Write>(w: &mut W) -> Result<(), Box<dyn Error>> {
    let mut table = [0u64; USIZE_BITS];

    table[0] = 256;
    for n in 1..USIZE_BITS {
        table[n] = mul64(table[n - 1], table[n - 1]);
    }

    writeln!(w, "pub(crate) static POW256_64: [u64; {}] = [", USIZE_BITS)?;
    for (n, x) in table.iter().enumerate() {
        write!(w, "0x{:016X}, ", x)?;
        if n % 4 == 3 {
            writeln!(w)?;
        }
    }
    writeln!(w, "\n];")?;

    Ok(())
}

// ======================================================================
// RUSTDOCIFY README

// Rustdocify `README.md` for inclusion in `lib.rs`.
//
// - Check that internal links have correct version and crate name.
// - Convert internal links to rustdoc format.
//     - https://docs.rs/PACKAGE/VERSION/CRATE/struct.STRUCT.html
//       -->  crate::STRUCT
//     - https://docs.rs/PACKAGE/VERSION/CRATE/struct.STRUCT.html#method.METHOD
//       -->  crate::STRUCT::METHOD
// - Remove top-level header.
// - Change other headers to be one level higher.
fn rustdocify_readme(readme: &str) -> Result<String, Box<dyn Error>> {
    let package_name = env::var("CARGO_PKG_NAME")?;
    // Cargo doesn't seem to provide this.
    // NOTE: This works only if `Cargo.toml` doesn't define different crate name.
    let crate_name = package_name.replace('-', "_");

    let version = env::var("CARGO_PKG_VERSION")?;

    // CONVERT INTERNAL LINKS

    let re_links = Regex::new(&format!(
        r"\bhttps://docs\.rs/{}/([^/]+)/([^/]+)/struct\.([^.]+)\.html(?:#method\.(\w+))?\b",
        package_name
    ))?;

    let readme = re_links
        .replace_all(readme, |cap: &Captures| {
            if cap[1] != version {
                panic!(
                    "ERROR: Wrong version number in link in README.md ({} != {})",
                    &cap[1], version
                );
            }

            if cap[2] != crate_name {
                panic!(
                    "ERROR: Wrong crate name in link in README.md ({} != {})",
                    &cap[2], crate_name
                );
            }

            let struct_name = &cap[3];

            if let Some(method_name) = cap.get(4) {
                format!("crate::{}::{}", struct_name, method_name.as_str())
            } else {
                format!("crate::{}", struct_name)
            }
        })
        .into_owned();

    // HEADERS

    let re_headers = Regex::new(r"(?m)^(#+)(.+)$")?;
    let readme = re_headers
        .replace_all(&readme, |cap: &Captures| {
            if cap[1].len() == 1 {
                // Remove top-level header.
                "".to_owned()
            } else {
                // Change other headers to be one level higher.
                format!("{}{}", &cap[1][1..], &cap[2])
            }
        })
        .into_owned();

    Ok(readme)
}
