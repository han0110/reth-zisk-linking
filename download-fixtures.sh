#!/usr/bin/env bash

set -euo pipefail

FILE_ID=1MEt9LQuXVi6iBMGmXyPPVtp60lQa_DYs
HERE=$(cd "$(dirname "$0")" && pwd)
TAR="$HERE/fixtures.tar.zst"
DIR="$HERE/fixtures"

mkdir -p "$DIR"
[ -f "$TAR" ] || curl -fL -o "$TAR" "https://drive.usercontent.google.com/download?id=$FILE_ID&export=download&confirm=t"
tar --zstd -xf "$TAR" -C "$DIR" --strip-components=1
