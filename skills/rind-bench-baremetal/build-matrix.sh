#!/usr/bin/env bash
# Usage: ./build-matrix.sh [target] [crate...]
# Builds each crates/tv-* member as a no_std rlib for the target. If the first
# attempt fails in getrandom, it retries with getrandom's custom backend cfg.
# Prints one TSV line per crate: crate, result, getrandom-cfg, first error.
set -u
T=${1:-thumbv6m-none-eabi}; shift || true
cd "$(dirname "$0")" || exit
if [ ! -d crates ]; then
  echo "build-matrix: crates/ is missing" >&2
  exit 1
fi
CRATES=${*:-$(ls crates)}
mkdir -p "logs/$T"
for c in $CRATES; do
  log=logs/$T/$c.log; cfg=no
  cargo build --release --target "$T" -p "$c" >"$log" 2>&1; rc=$?
  if [ $rc -ne 0 ] && grep -q 'getrandom' "$log" && grep -qi 'custom backend\|not supported' "$log"; then
    cfg=yes
    RUSTFLAGS='--cfg getrandom_backend="custom"' cargo build --release --target "$T" -p "$c" >"$log" 2>&1; rc=$?
  fi
  if [ $rc -eq 0 ]; then res=PASS; err=""; else res=FAIL
    err=$( (grep -m1 -E '^error' "$log" || grep -m1 -iE 'error' "$log") | cut -c1-240); fi
  printf '%s\t%s\tgetrandom-custom=%s\t%s\n' "$c" "$res" "$cfg" "$err"
done
