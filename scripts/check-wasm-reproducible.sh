#!/usr/bin/env bash
# Verifies that building the WASM engine twice from the same source produces byte-identical binaries
# (docs/verification/TRUSTED_BASE.md, issue #771).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

out="web/src/lib/wasm/dedaliano_engine_bg.wasm"
if [ ! -f "$out" ]; then
  echo "Initial build not found at $out, running make wasm..."
  scripts/wasm-cached.sh
fi

tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t 'wasm-repro')
cleanup() { rm -rf "$tmp_dir"; }
trap cleanup EXIT

first_wasm="$tmp_dir/first.wasm"
cp "$out" "$first_wasm"
if command -v sha256sum >/dev/null 2>&1; then
  first_sha=$(sha256sum "$first_wasm" | cut -d' ' -f1)
else
  first_sha=$(shasum -a 256 "$first_wasm" | cut -d' ' -f1)
fi

echo "First build sha256:  $first_sha"

# Perform a second build bypassing cache to test reproducibility
rm -rf web/src/lib/wasm
STABILEO_WASM_CACHE=0 scripts/wasm-cached.sh

second_wasm="$out"
if command -v sha256sum >/dev/null 2>&1; then
  second_sha=$(sha256sum "$second_wasm" | cut -d' ' -f1)
else
  second_sha=$(shasum -a 256 "$second_wasm" | cut -d' ' -f1)
fi

echo "Second build sha256: $second_sha"

if ! cmp -s "$first_wasm" "$second_wasm"; then
  echo "ERROR: Engine WASM builds are not byte-identical!" >&2
  echo "First build sha256:  $first_sha" >&2
  echo "Second build sha256: $second_sha" >&2
  exit 1
fi

echo "OK: WASM builds are byte-identical ($first_sha)"
