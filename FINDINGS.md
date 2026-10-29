# Findings

Open items and standing facts. An entry names the condition that closes it.

## The cut

| Where | Closes when |
| --- | --- |
| The root `Cargo.toml` names porter by `rev = "PORTER_REV"` | the gate session fills in the porter rev that has `porter_wire::names::{INFERENCE_BUS, INFERENCE_PATH}` and no `inference` group in porter-dbus, and relocks |
| `steward-dbus` keeps a private copy of three small helpers of porter-dbus's `json_value` (`bad`, `into_object`, `into_string`, `pub(crate)` there) | porter-dbus exports them, or the need codec stops using them |
| `#[bus_interface]` expands to `crate::Call` and `crate::Unserved`, so `steward-dbus`'s `lib.rs` has private `use porter_dbus::{Call, Unserved}` for it | the macro names its paths through a crate-root argument or `porter_dbus::` |
| The cut compiles nowhere yet: no `cargo` was run in this lane | the gate session's first green `scripts/check-boundary.sh`, clippy and tests |
| `.github/workflows/ci.yml` and `portable.yml` | the repository secret `DEPS_TOKEN` (read access to porter, stoker and quire) exists and the first run of each is green; `bus-tests` is `continue-on-error` until it has passed on a hosted runner |
| `scripts/check-boundary.sh` has no HIDDEN-crates rows (who may depend on `porter-dbus`, `porter-callers`, `porter-settings`, `porter-store`) | they are decided with porter's side of the cut; today the EDGES rows pin every direct edge of the nine crates |
| The test `every_app_inferds_table_names_is_the_same_app_in_accountds` (Q10: inferd's and accountd's shipped caller tables name the same app and programs per unit) was dropped from `crates/inferd/tests/it/dist.rs`: it read porter's `dist/callers.toml`, which is not in this repository | a check that both tables agree runs where both files are present (porter's gate with a steward checkout, or a CI step that fetches porter's table at the pinned rev) |
| The paths `share/porter/`, `share/doc/porter/` and `/etc/porter/inferd-callers.toml` keep porter's names, because inferd and Settings read them | a lane that renames them with their readers |

## Facts carried over from porter

- inferd probes `127.0.0.1` only, at the ports of `[probe]`: Ollama `11434`, llama.cpp `8080`,
  LM Studio `1234`. A LAN host is never auto-probed, by design.
- A stopped runtime is seen at the next look (at most `[probe] longest_s`, 120 s by default; `Rescan`
  looks at once).
- Probed models: licence is `open("unknown")`, no `StructuredOutput` feature, the context is what the
  runtime reports or the 2048 floor.
