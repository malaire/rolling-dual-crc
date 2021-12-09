#!/usr/bin/env bash

set -e

cargo test
cargo test --features crc32c
cargo test --features crc64fast
cargo test --features crc32c,crc64fast

echo OK
echo
