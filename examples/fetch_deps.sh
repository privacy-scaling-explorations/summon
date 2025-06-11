#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

DEPS_DIR="$SCRIPT_DIR/deps"

if [ deps/fetch_done -nt fetch_deps.sh ]; then
  # Dependencies have already been fetched
  exit
fi

rm -rf "$DEPS_DIR"
mkdir -p "$DEPS_DIR"
cd "$DEPS_DIR"

echo "*" >.gitignore

SUMMON_LIB="https://raw.githubusercontent.com/privacy-scaling-explorations/summon-lib/c24b5f32ccb8d8ffe77fb1465425a0575012b4b7"

mkdir -p sha256
pushd sha256
  curl -LO "$SUMMON_LIB/sha256/mod.ts"
  curl -LO "$SUMMON_LIB/sha256/sha256Compress.ts"
popd

touch fetch_done
