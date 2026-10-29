#!/usr/bin/env bash
# Build the WASM engine into web/src/lib/wasm, or copy it from a cache when the same sources were
# already built. The key is the commit the engine embeds as its provenance (engine/build.rs:
# STABILEO_GIT_COMMIT, else HEAD), with the toolchain pin and the wasm-pack version: a hit is the
# build of that very commit, so certificates and reproduction bundles name the right one. Worktrees
# made from the same base share it. Uncommitted changes under engine/ (or to the files the build
# reads) always build, and are never stored.
#
#   scripts/wasm-cached.sh          # cache hit: a copy, seconds; miss: wasm-pack, then stored
#   STABILEO_WASM_CACHE=0 ...       # always build
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

cache_root=${XDG_CACHE_HOME:-$HOME/.cache}/stabileo/wasm
out=web/src/lib/wasm
build() {
  local top
  top=$(git rev-parse --show-toplevel)
  local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
  local commit_time
  commit_time=$(git log -1 --format=%ct 2>/dev/null || echo "1700000000")
  export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$commit_time}"
  export STABILEO_GIT_COMMIT="${STABILEO_GIT_COMMIT:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}"

  local sep=$'\x1f'
  local flags="-C${sep}codegen-units=1${sep}--remap-path-prefix=${top}=stabileo${sep}--remap-path-prefix=${cargo_home}=/cargo"
  if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    flags="${flags}${sep}--remap-path-prefix=${CARGO_TARGET_DIR}=/target"
  fi
  if sysroot=$(rustc --print sysroot 2>/dev/null); then
    [ -n "$sysroot" ] && flags="${flags}${sep}--remap-path-prefix=${sysroot}=/rustc"
  fi
  (cd engine && CARGO_ENCODED_RUSTFLAGS="$flags" wasm-pack build --target web --out-dir ../web/src/lib/wasm --no-opt)
}

if [ "${STABILEO_WASM_CACHE:-1}" = 0 ] || [ -n "$(git status --porcelain -- engine Cargo.lock Cargo.toml rust-toolchain.toml)" ]; then
  build
  exit 0
fi

key=$( {
  echo "${STABILEO_GIT_COMMIT:-$(git rev-parse HEAD)}"
  git rev-parse HEAD:rust-toolchain.toml 2>/dev/null || echo "no rust-toolchain.toml"
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
# Publish once: the mkdir is the lock, and a key someone stored meanwhile is kept (an mv onto an
# existing directory would nest this build inside it).
if mkdir "$hit.lock" 2>/dev/null; then
  [ -e "$hit" ] || mv "$tmp" "$hit"
  rmdir "$hit.lock"
fi
rm -rf "$tmp"
# Keep the 20 most recent builds.
ls -1dt "$cache_root"/*/ 2>/dev/null | grep -v "\.lock/$" | tail -n +21 | while read -r d; do rm -rf "$d"; done
echo "wasm: built and cached ($key)"
