#!/usr/bin/env bash

set -e

# Rebuid programs
rm -rf lib/sablier_thread_program.so
cd programs/thread && anchor build; cd -;
cp -fv target/deploy/sablier_thread_program.so lib/

# Rebuild plugin
rm -rf lib/libsablier_plugin.dylib
cargo build --manifest-path plugin/Cargo.toml
cp -fv target/debug/libsablier_plugin.dylib lib/

# bpf-program
crate_name="hello_sablier"
cd ~/examples/$crate_name
anchor build
cd -

# Clean ledger
rm -rf test-ledger

RUST_LOG=sablier_plugin sablier localnet \
    --bpf-program ~/examples/$crate_name/target/deploy/$crate_name-keypair.json \
    --bpf-program ~/examples/$crate_name/target/deploy/$crate_name.so

