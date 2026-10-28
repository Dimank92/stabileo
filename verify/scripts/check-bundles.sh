#!/usr/bin/env bash
# Re-check certificate bundles (linear static, plastic collapse, load combinations) with
# two independent checkers:
# 1) the verified Lean checker (`lake exe certcheck`, CERTIFY.md)
# 2) the independent exact-rational Rust checker (`certcheck-exact`, CERTIFY.md)
#
#   scripts/check-bundles.sh [--no-generate] [DIR...]
#
# Generates bundles for the fixture models (Rust tests `certify_bundle::export_*`, written
# to ../target/cert-bundles) unless --no-generate, builds `certcheck` and `certcheck-exact`,
# and checks every bundle in the generated directory plus the committed samples in samples/
# (or in the DIRs given):
# * `NAME.json` must be ACCEPTED by BOTH checkers, carry a provenance, and its `inputSha256`
#   must be the SHA-256 of the shipped canonical input `NAME.input.json`;
# * `NAME.tampered-*.json` must be REJECTED by BOTH checkers (exit 1);
# * `NAME.wronghash*.json` is accepted by certcheck but its hash must NOT match the input
#   (this script must catch it);
# * `NAME.badformat*.json` (unknown format) must be refused with exit 2 by BOTH checkers;
# * any disagreement between Lean and Rust checkers causes an immediate failure;
# * a `stabileo.linear-certificate.v1` bundle (no provenance) must be accepted.
# Run from anywhere inside the verify dev shell.
set -euo pipefail
cd "$(dirname "$0")/.."

generate=1
if [ "${1:-}" = "--no-generate" ]; then
	generate=0
	shift
fi

export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$(cd .. && pwd)/target}
if [ $# -gt 0 ]; then
	dirs=("$@")
else
	dirs=(samples)
	if [ "$generate" = 1 ]; then
		dirs+=("$CARGO_TARGET_DIR/cert-bundles")
	fi
fi

if [ "$generate" = 1 ]; then
	(cd ../engine && cargo test -j 4 --test certificates certify_bundle::export_ -- --nocapture)
fi

sha256() {
	if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

lake build certcheck
(cd ../engine && cargo build --bin certcheck-exact)

exact_bin=""
for candidate in "${CARGO_TARGET_DIR}/debug/certcheck-exact" "${CARGO_TARGET_DIR}/certcheck-exact" "../target/debug/certcheck-exact" "../target/release/certcheck-exact"; do
	if [ -x "$candidate" ]; then
		exact_bin="$candidate"
		break
	fi
done

if [ -z "$exact_bin" ]; then
	echo "error: certcheck-exact binary not found in target directories" >&2
	exit 1
fi

shopt -s nullglob
bundles=()
for d in "${dirs[@]}"; do
	for f in "$d"/*.json; do
		case "$f" in *.input.json) ;; *) bundles+=("$f") ;; esac
	done
done
if [ ${#bundles[@]} -eq 0 ]; then
	echo "error: no bundles in ${dirs[*]}" >&2
	exit 1
fi

fail=0
for b in "${bundles[@]}"; do
	name=$(basename "$b")
	set +e
	out=$(.lake/build/bin/certcheck "$b")
	status=$?
	exact_out=$("$exact_bin" "$b" 2>&1)
	exact_status=$?
	set -e
	echo "$out"
	case "$name" in
	*.tampered-*) want=1 ;;
	*.badformat*) want=2 ;;
	*) want=0 ;;
	esac
	if [ "$status" != "$want" ]; then
		echo "error: $name: Lean certcheck exit $status, expected $want" >&2
		fail=1
		continue
	fi
	if [ "$exact_status" != "$want" ]; then
		echo "error: $name: Rust certcheck-exact exit $exact_status, expected $want ($exact_out)" >&2
		fail=1
		continue
	fi
	if [ "$status" != "$exact_status" ]; then
		echo "error: $name: checker disagreement! Lean=$status, Rust exact=$exact_status" >&2
		fail=1
		continue
	fi
	[ "$want" != 0 ] && continue
	if [ "$(jq -r .format "$b")" = "stabileo.linear-certificate.v1" ]; then
		echo "  format v1: no provenance to check (read for backward compatibility)"
		continue
	fi
	# provenance: the recorded hash against the shipped input
	input="$(dirname "$b")/${name%%.*}.input.json"
	recorded=$(jq -r '.provenance.inputSha256 // empty' "$b")
	if [ -z "$recorded" ] || [ ! -f "$input" ]; then
		echo "error: $name: no provenance hash or no shipped input $(basename "$input")" >&2
		fail=1
		continue
	fi
	actual=$(sha256 "$input")
	case "$name" in
	*.wronghash*)
		if [ "$recorded" = "$actual" ]; then
			echo "error: $name: wrong-hash sample matches its input" >&2
			fail=1
		else
			echo "  input hash MISMATCH (expected for this negative sample): recorded $recorded, $(basename "$input") has $actual"
		fi
		;;
	*)
		if [ "$recorded" = "$actual" ]; then
			echo "  input hash ok: $(basename "$input")"
		else
			echo "error: $name: input hash mismatch: recorded $recorded, $(basename "$input") has $actual" >&2
			fail=1
		fi
		;;
	esac
done
if [ "$fail" = 1 ]; then
	exit 1
fi
echo "bundles ok (${#bundles[@]} checked by both Lean and Rust exact checkers)"
