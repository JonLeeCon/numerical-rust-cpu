#!/bin/bash
set -euo pipefail

# Set up cache directory if no cache was restored by gitlab
CI_ROOT="$(pwd)"
mkdir -p target
TARGET_CACHE="${CI_ROOT}/target"

# Designate where output data and pictures will go
PICS_OUTPUT="${CI_ROOT}/pics"

# Go to solution clone from docker image
cd ~/solution

# Restore cargo cache for this build, if gitlab provided one
if [[ -e ${TARGET_CACHE} ]]; then
    mv "${TARGET_CACHE}" target
fi

# Run the simulation, produce pictures, collect them to "pics" directory
cargo run --release --offline -- -n10
mkdir -p "${PICS_OUTPUT}"
data-to-pics -o "${PICS_OUTPUT}"
mv output.h5 "${PICS_OUTPUT}"

# Clean up the cache, expose it
cargo sweep --installed
mv target "${TARGET_CACHE}"
