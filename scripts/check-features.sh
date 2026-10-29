#!/usr/bin/env bash
# The feature sets the consumers of porter's crates build, each one clippy with -D warnings.
#
# The workspace gate builds `--workspace --all-features`, which unifies every feature into one
# build and so never sees the build a consumer makes on its own. docket builds inferd that way
# (`cargo install --git steward`, -D warnings): the client with `dbus` and no other feature once
# left two framed.rs methods unused, and the gate passed (porter 5bdda62; fixed in b12886e).
# Each set below is checked here, alone, so the same break fails this script.
#
# The sets come from the consumers' manifests at origin/master. The comment above each set
# names the consumer it comes from. Keep the list as data: add a set when a consumer adds one.
#
# Output: one line per build, `ok <set>` or `FAIL <set>`, with the first compiler errors of a
# failed build and the path of its log (the logs go to $CARGO_TARGET_DIR/check-features/, so
# nothing is written into the tree). The exit status is the verdict.
set -uo pipefail
cd "$(dirname "$0")/.."

# Each entry: "<set> :: <cargo arguments>". Every build is `cargo clippy <arguments>
# --lib --bins -- -D warnings`: the targets a consumer compiles. Test targets are left out on
# purpose: a test target's helpers are dead code in a default build (accountd's tests/it/ fails
# -D warnings with default features on master, and no consumer compiles it).
BUILDS=(
  # The daemon alone with its default features: `-p` alone, so no other package's features are
  # unified in. inferd is docket's sibling binary.
  "inferd (default) :: -p inferd"

  # inferd-client, in the sets its consumers name: detent's Settings (dbus, lending), docket and
  # almanac (dbus), inferd's tests (dbus), an app that hosts inference itself (engines).
  "inferd-client (no features) :: -p inferd-client --no-default-features"
  "inferd-client (dbus: inferd's tests, docket, almanac) :: -p inferd-client --no-default-features --features dbus"
  "inferd-client (dbus,lending: detent's Settings) :: -p inferd-client --no-default-features --features dbus,lending"
  "inferd-client (engines) :: -p inferd-client --features engines"
)

logs="${CARGO_TARGET_DIR:-target}/check-features"
mkdir -p "$logs"
fail=0
n=0
for build in "${BUILDS[@]}"; do
  n=$((n + 1))
  set_name=${build%% :: *}
  read -r -a args <<<"${build#* :: }"
  log="$logs/$n.log"
  if cargo clippy "${args[@]}" --lib --bins -- -D warnings >"$log" 2>&1; then
    echo "ok $set_name"
  else
    echo "FAIL $set_name"
    echo "  log: $log"
    grep -m 5 -E '^(error|warning)' "$log" | sed 's/^/  /'
    fail=1
  fi
done
exit "$fail"
