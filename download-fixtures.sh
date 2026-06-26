#!/usr/bin/env bash

# Downloads a fixture set into ./fixtures/<name>. Both sets normalize to canonical
# schema-prefixed SSZ input/output bytes that the integration test consumes.
# rpc-bpo2 holds RPC-derived mainnet blocks, one *.json.zst per block with a
# top-level statelessInputBytes. eest-glamsterdam-devnet-5 holds EEST
# blockchain_test blocks with statelessInputBytes/statelessOutputBytes per block.

set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
DIR="$HERE/fixtures"

case "${1:-}" in
rpc-bpo2)
    URL="https://github.com/han0110/ere-guests/releases/download/rpc-fixtures@v0.1.0/rpc-bpo2.tar.zst"
    TAR="$HERE/rpc-bpo2.tar.zst"
    [ -f "$TAR" ] || curl -fL -o "$TAR" "$URL"
    mkdir -p "$DIR"
    # The archive holds a top-level rpc-bpo2/ directory of *.json.zst fixtures.
    tar --zstd -xf "$TAR" -C "$DIR"
    ;;
eest-glamsterdam-devnet-5)
    URL="https://github.com/ethereum/execution-specs/releases/download/tests-zkevm@v0.4.1/fixtures_zkevm.tar.gz"
    TAR="$HERE/fixtures_zkevm.tar.gz"
    [ -f "$TAR" ] || curl -fL -o "$TAR" "$URL"
    DEST="$DIR/eest-glamsterdam-devnet-5"
    mkdir -p "$DEST"
    # Strip the "fixtures/blockchain_tests/" prefix so fork trees land directly under
    # the destination, excluding the archive's .meta sidecar files.
    tar -xzf "$TAR" -C "$DEST" --strip-components=2 fixtures/blockchain_tests
    ;;
*)
    echo "usage: $0 {rpc-bpo2|eest-glamsterdam-devnet-5}" >&2
    exit 1
    ;;
esac
