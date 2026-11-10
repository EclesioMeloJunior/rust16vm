#!/usr/bin/env bash
# Runs clippy over the whole package using the lint policy declared in
# Cargo.toml ([lints.clippy]) and the knobs in clippy.toml.
#
# usage:
#   ./scripts/clippy.sh             lint the library, binaries and tests
#   ./scripts/clippy.sh --fix       auto-apply machine-applicable suggestions
#   ./scripts/clippy.sh --strict    warnings become errors (CI mode)
#
# the crate targets nightly rustc (edition 2024 + #![feature(io_const_error)]),
# so the script forces the nightly toolchain when it is not the default.
set -euo pipefail

cd "$(dirname "$0")/.."

CARGO=(cargo)
if command -v rustup >/dev/null 2>&1; then
  default_tc="$(rustup default 2>/dev/null | awk '{print $1}')"
  if [[ "${default_tc}" != nightly* ]]; then
    CARGO=(cargo +nightly)
  fi
fi

FIX=""
ARGS=()
for arg in "$@"; do
  case "${arg}" in
    --fix)    FIX="1" ;;
    --strict) ARGS+=(-- -D warnings) ;;
    *) echo "unknown option: ${arg} (expected --fix or --strict)"; exit 1 ;;
  esac
done

if [[ -n "${FIX}" ]]; then
  echo "==> clippy --fix --all-targets (machine-applicable suggestions)"
  "${CARGO[@]}" clippy --fix --all-targets --allow-dirty --allow-staged
  echo "==> clippy fixes applied; re-run without --fix to see what is left"
else
  echo "==> clippy --all-targets"
  "${CARGO[@]}" clippy --all-targets "${ARGS[@]}"
fi
