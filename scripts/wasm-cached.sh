#!/usr/bin/env bash
# Build the WASM engine into web/src/lib/wasm, or copy it from a cache when the same sources were
# already built. The key is everything the build reads: the committed engine tree, Cargo.lock,
# the workspace Cargo.toml, the toolchain pin and the wasm-pack version. Uncommitted changes
# under engine/ (or to those files) always build, and are never stored.
#
#   scripts/wasm-cached.sh          # cache hit: a copy, seconds; miss: wasm-pack, then stored
#   STABILEO_WASM_CACHE=0 ...       # always build
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

cache_root=${XDG_CACHE_HOME:-$HOME/.cache}/stabileo/wasm
out=web/src/lib/wasm
build() { (cd engine && wasm-pack build --target web --out-dir ../web/src/lib/wasm --no-opt); }

if [ "${STABILEO_WASM_CACHE:-1}" = 0 ] || [ -n "$(git status --porcelain -- engine Cargo.lock Cargo.toml rust-toolchain.toml)" ]; then
  build
  exit 0
fi

key=$( {
  for f in engine Cargo.lock Cargo.toml rust-toolchain.toml; do git rev-parse "HEAD:$f" 2>/dev/null || echo "no $f"; done
  wasm-pack --version
} | shasum -a 256 | cut -c1-32)
hit=$cache_root/$key

if [ -f "$hit/.complete" ]; then
  rm -rf "$out"
  mkdir -p "$out"
  cp -R "$hit/." "$out/"
  rm -f "$out/.complete"
  echo "wasm: from cache ($key)"
  exit 0
fi

build
mkdir -p "$cache_root"
tmp=$(mktemp -d "$cache_root/.tmp.XXXXXX")
cp -R "$out/." "$tmp/"
touch "$tmp/.complete"
# Another worktree may have stored the same key meanwhile: keep theirs.
mv "$tmp" "$hit" 2>/dev/null || rm -rf "$tmp"
# Keep the 20 most recent builds.
ls -1dt "$cache_root"/*/ 2>/dev/null | tail -n +21 | while read -r d; do rm -rf "$d"; done
echo "wasm: built and cached ($key)"
