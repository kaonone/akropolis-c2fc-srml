#!/usr/bin/env bash


set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null && pwd )"
bold=$(tput bold)
normal=$(tput sgr0)


./scripts/build.sh

echo "${bold}Building node${normal}"
# cargo run -- build-spec --chain akropolis > ./target/akropolis.spec.debug.json
cargo run --release -- build-spec --chain akropolis > target/akropolis.spec.json


# Restore initial directory.
popd >/dev/null
