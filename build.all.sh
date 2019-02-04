#!/usr/bin/env bash

set -e

# save current CARGO_INCREMENTAL
TEMP___CARGO_INCREMENTAL=${CARGO_INCREMENTAL}

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

./build.wasm.sh

# restore CARGO_INCREMENTAL
export CARGO_INCREMENTAL=${TEMP___CARGO_INCREMENTAL}
export CARGO_INCREMENTAL=0

# cargo build
cargo build --release
