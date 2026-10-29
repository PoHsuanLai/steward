# Changelog

## Unreleased

### Cut from porter e9e7cdcd

- steward is porter's AI half, cut with its history (`git filter-repo`) from porter `e9e7cdcd`:
  `inferd`, `inferd-client`, `porter-infer`, `porter-router`, `porter-turns`, `porter-bridge` and
  `porter-tailnet`, whole, plus the inference group of `porter-dbus` as the new crate `steward-dbus`
  and the port probe of `porter-discover` as the new crate `inferd-probe`. Crate names are porter's
  for now; renames and carving come in later lanes.
- The root `Cargo.toml` is a workspace of these nine crates. Every porter crate they use is a git
  dependency at one `rev` (`PORTER_REV` until the cut is gated); stoker and quire are pinned as in porter.
- `steward_dbus::inference` replaces `porter_dbus::inference`; `inferd_probe::{probe_ports, ProbeHit}`
  replaces `porter_discover::{probe_ports, ProbeHit}`. `INFERENCE_BUS` and `INFERENCE_PATH` move to
  `porter_wire::names` and are re-exported by `steward_dbus::inference`.
- The inference parts of porter-dbus's `codec` and `introspection` tests are in `steward-dbus`'s
  `tests/it/`.
- Top-level files, CI and scripts (`check-boundary.sh`, `check-features.sh`, `check-portable.sh`,
  `install.sh`) are porter's, cut down to these crates.
