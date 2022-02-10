#!/usr/bin/env bash

cargo_build="cargo build --release"

# compile native
echo "building for macOS..."

$cargo_build --target x86_64-apple-darwin

echo "building for macOS ARM..."

$cargo_build --target aarch64-apple-darwin


# cross compile windows
echo "building for windows..."
export CPATH="/usr/local/Cellar/mingw-w64/9.0.0_2/toolchain-x86_64/mingw/include"

$cargo_build --target x86_64-pc-windows-gnu


# cross compile linux
echo "building for linux..."
CPATH="/usr/local/Cellar/x86_64-unknown-linux-gnu/7.2.0/toolchain/x86_64-unknown-linux-gnu/sysroot/usr/include/"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="x86_64-unknown-linux-gnu-gcc"

$cargo_build --target x86_64-unknown-linux-gnu
