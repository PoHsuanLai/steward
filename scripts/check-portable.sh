#!/usr/bin/env bash
# The portable core (quire design/36 section 1, rule 3 and section 4): the crates here that are
# not the bus or the daemon build without the desktop. Each core below must `cargo check` with the
# features named, and its normal dependency tree must reach no D-Bus (zbus, zvariant).
#
# Called from check-boundary.sh, which the lane gate runs. Output says each check and each
# tree; the exit status is the verdict.
set -uo pipefail
cd "$(dirname "$0")/.."

fail=0

# What no portable core reaches.
BUS='zbus|zvariant'

# check <label> <crate> <forbidden regex> [cargo feature flags...]
check() {
  local label=$1 crate=$2 forbidden=$3
  shift 3
  if cargo check -p "$crate" "$@" >/dev/null 2>&1; then
    echo "check ok: $label"
  else
    echo "FAIL check: $label does not build"
    fail=1
  fi
  local tree
  if ! tree=$(cargo tree -p "$crate" "$@" -e normal --prefix none 2>/dev/null); then
    echo "FAIL tree: cargo tree cannot resolve $label"
    fail=1
    return
  fi
  local leaked
  # `name vX.Y.Z` is the first two fields; a path dependency also names its directory.
  leaked=$(awk '{print $1}' <<<"$tree" | grep -xE "$forbidden" | sort -u | tr '\n' ' ')
  if [ -n "$leaked" ]; then
    echo "LEAK: $label reaches ${leaked% }"
    fail=1
  else
    echo "tree ok: $label reaches none of ${forbidden//|/ }"
  fi
}

check "porter-infer" porter-infer "$BUS"
check "porter-bridge" porter-bridge "$BUS"
check "porter-router" porter-router "$BUS"
check "porter-turns" porter-turns "$BUS"
check "porter-tailnet" porter-tailnet "$BUS"
check "inferd-probe" inferd-probe "$BUS"
check "inferd-client (no features)" inferd-client "$BUS" --no-default-features

exit "$fail"
