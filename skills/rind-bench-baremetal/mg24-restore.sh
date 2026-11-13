#!/usr/bin/env bash
# Restore an EFR32MG24 main-flash image and check it by readback.
# Does not erase and does not unlock. Set LENGTH to the saved image's
# size in bytes, the same span that was read before anything was written.
#
#   CHIP=<probe-rs part name> BACKUP=backup.bin LENGTH=<bytes> ./mg24-restore.sh
#   BASE=0x08000000          default
#   PROBE=<selector>         only when more than one probe is attached; do not commit it
#   SHA256=<hex>             when set, the backup and the readback must both match it
set -euo pipefail

: "${CHIP:?set CHIP to the probe-rs part name for this EFR32MG24}"
: "${BACKUP:?set BACKUP to the saved image}"
: "${LENGTH:?set LENGTH to the image size in bytes}"
BASE=${BASE:-0x08000000}

hash_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

download() {
  if [ -n "${PROBE:-}" ]; then
    probe-rs download --probe "$PROBE" --chip "$CHIP" --binary-format bin \
      --base-address "$BASE" --verify "$BACKUP"
  else
    probe-rs download --chip "$CHIP" --binary-format bin \
      --base-address "$BASE" --verify "$BACKUP"
  fi
}

readback() {
  if [ -n "${PROBE:-}" ]; then
    probe-rs read --probe "$PROBE" --chip "$CHIP" -o "$1" -f binary b32 "$BASE" "$((LENGTH / 4))"
  else
    probe-rs read --chip "$CHIP" -o "$1" -f binary b32 "$BASE" "$((LENGTH / 4))"
  fi
}

reset_chip() {
  if [ -n "${PROBE:-}" ]; then
    probe-rs reset --probe "$PROBE" --chip "$CHIP"
  else
    probe-rs reset --chip "$CHIP"
  fi
}

if [ -n "${SHA256:-}" ]; then
  got=$(hash_of "$BACKUP")
  if [ "$got" != "$SHA256" ]; then
    echo "mg24-restore: backup sha256 is $got, expected $SHA256" >&2
    exit 1
  fi
fi

download
out=$(mktemp)
trap 'rm -f "$out"' EXIT
readback "$out"
if [ -n "${SHA256:-}" ]; then
  got=$(hash_of "$out")
  if [ "$got" != "$SHA256" ]; then
    echo "mg24-restore: readback sha256 is $got, expected $SHA256" >&2
    exit 1
  fi
else
  cmp "$BACKUP" "$out"
fi
reset_chip
echo "mg24-restore: readback matches $BACKUP"
