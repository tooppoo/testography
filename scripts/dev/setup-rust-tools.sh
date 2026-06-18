#!/usr/bin/env sh
set -eu

cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
