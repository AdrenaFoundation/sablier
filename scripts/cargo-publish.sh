#!/bin/bash

# Publish shared libs
cargo publish -p sablier-cron
sleep 25
cargo publish -p sablier-utils
sleep 25
cargo publish -p sablier-relayer-api
sleep 25
cargo publish -p sablier-plugin-utils
sleep 25

# Publish programs
cargo publish -p sablier-network-program
sleep 25
cargo publish -p sablier-thread-program
sleep 25
cargo publish -p sablier-webhook-program
sleep 25

# Publish SDK
cargo publish -p sablier-sdk
sleep 25

# Publish downstream bins and libs
# These are most likely to fail due to Anchor dependency issues.
cargo publish -p sablier-cli
