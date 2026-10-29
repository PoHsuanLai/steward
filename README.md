# steward

Steward is the AI broker of the desktop. One background service, `inferd`, lets apps use a model on
this computer, on another of your computers (if you allow it) or one you have an account for, under
the limits you set (what may leave the machine, how much it may spend). Apps ask for what they need
("a chat model that reads images"), never for a brand, and never see a key.

Status: cut from [porter](https://github.com/PoHsuanLai/porter) at `e9e7cdcd`; see
[CHANGELOG.md](CHANGELOG.md). The crates keep porter's names for now.

## What is here

| Crate | What it is |
| --- | --- |
| `inferd` | the daemon: serves `org.quire.Inference1` on the session bus, with the router, the engine host, the model turns and the audit trail |
| `inferd-client` | the app's client of inferd, over D-Bus or hosted in the app |
| `steward-dbus` | `org.quire.Inference1` written once: proxies, server traits, introspection, bus vocabulary |
| `steward-rig` | test tooling, never installed: `steward-rig-client open`, an app opening an inferd session in a jailed scenario |
| `inferd-probe` | the port probe that finds local runtimes (Ollama, llama.cpp, LM Studio) |
| `porter-infer` | typed requests and replies, the routing policy, spend caps, audit entries |
| `porter-router` | who answers a session, the catalogue, the local model book, the session machine |
| `porter-turns` | running model turns: chat, embeddings, computer-use steps, structured replies |
| `porter-bridge` | the one mapping between the wire types and stoker's provider types |
| `porter-tailnet` | lending models to the person's other computers over their Tailscale network |

`ARCHITECTURE.md` has the map and the allowed edges. steward depends on porter by git rev; porter
never depends on steward.

## Install

You need a Linux desktop with systemd and a Rust toolchain to build.

    scripts/install.sh                      # builds, then installs under /usr (run as root)
    scripts/install.sh --prefix /usr/local
    scripts/install.sh --destdir /tmp/stage # copies into a staging folder and touches nothing else

The script also reads `PREFIX` and `DESTDIR` from the environment. It can be run again at any time:
a file that is already right is left alone, and a caller table you edited is kept (the shipped one
is put beside it as `inferd-callers.toml.dist`). It never writes into a home folder. `--bin-dir DIR`
installs a daemon you built yourself instead of building it, and `--help` lists the rest.

Building needs access to the private repositories the dependencies come from (porter, stoker and
quire, as git dependencies); there is no public mirror yet.

## Where files go

Installed by the script (under the prefix, `/usr` by default):

| What | Where |
| --- | --- |
| The daemon | `libexec/quire/inferd` |
| Its systemd user unit | `lib/systemd/user/inferd.service` |
| What lets the session bus start it | `share/dbus-1/services/org.quire.Inference1.service` |
| The AI page's settings description | `share/quire/settings/inferd.settings.toml` |
| The file that lets inferd answer your other computers (kept as data; the Settings switch installs it, with your say-so) | `share/porter/inferd-tailnet.conf` |
| Examples to copy: the configuration and the optional file that lets it reach hosted models | `share/doc/porter/examples/` |
| The table that says which app is Settings, which are the services | `/etc/porter/inferd-callers.toml` |

## Checks

    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo deny check licenses
    scripts/check-boundary.sh
    scripts/check-features.sh

## Licence

MIT OR Apache-2.0.
