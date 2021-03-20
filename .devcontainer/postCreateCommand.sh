#!/bin/sh

rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup component add rust-src --toolchain nightly

cargo install cargo-edit
cargo install cargo-contract \
  --vers ${CONTRACT_VERSION} \
  --force --locked 