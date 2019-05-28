#!/usr/bin/env bash

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null && pwd )"
# CHAIN_SPECK=./target/akropolis.spec.debug.json
CHAIN_SPECK=./target/akropolis.spec.json

substrate --chain $CHAIN_SPECK --telemetry-url wss://telemetry.polkadot.io/submit/ 6 --rpc-cors all --validator --key //Alice --name AkroC2FC --force-authoring --base-path ../base --keystore-path ../keystore \
--offchain-worker-execution Wasm \
--block-construction-execution Wasm \
--importing-execution Wasm \
--other-execution Wasm \
--syncing-execution Wasm
