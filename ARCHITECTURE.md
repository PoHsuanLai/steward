# Architecture

steward is the desktop's AI broker: `inferd` serves `org.quire.Inference1` on the session bus, routes
each session to a model on this computer, on another of the person's computers, or one they hold an
account for, and runs the turns. This repository was cut from porter at `e9e7cdcd` (CHANGELOG.md).
The design is quire's `design/31-ACCOUNTS.md` and the agent layering in `agent-layers/steward.md`;
this file is the map of the code that exists today. `CONVENTIONS.md` holds the rules; `FINDINGS.md`
the open items.

This cut only moves code. Crate names are porter's; renames and carving into the layers of
`steward.md` come in later lanes, so the table below is the status quo, not the target.

## 1. Crates and allowed edges

The edge rule: **steward depends on porter by git rev; porter never depends on steward.** Every
`porter-*` crate that is not in the table is porter's, taken from `https://github.com/PoHsuanLai/porter`
at the one `rev` in the root `Cargo.toml` (`[workspace.dependencies]`). Stoker's crates and quire's
`ds-settings` are git dependencies at their pinned revs in the same block.

| Crate | Purpose | I/O |
| --- | --- | --- |
| `inferd` | `porter-daemon`, `porter-callers` (`ProcRoot` and `choose_proc_root`), `porter-client` and `porter-client-dbus` (`AiAccounts<Bus>`, how `broker::AccountdBroker` asks accountd for the permits an app holds and the key behind one, reports a local runtime and hears the account news), `porter-store` (`ImportMarker`: `old_keys` removes inferd's old key store once accountd's record covers it), `porter-settings` (the spend rows' paths), `porter-router` (routing, the catalogue, the local model book, the session machine and the seams) and `porter-turns` (a local model's turns, the structured-reply check, the audit trail, a pipeline's stages): inferd names both there, `porter-core`, `porter-match`, `porter-capability`, `porter-sheet`, `porter-wire`, `porter-ids`, `porter-dbus`, `porter-tailnet` and `porter-tailscale` (feature `io` of both: lending models to the person's other computers and calling theirs; `tailnet` is the wiring), `inferd-probe` (the probe of local runtimes' ports), `porter-http` and `porter-http-reqwest` (the plain-HTTP client of that probe), `porter-infer`, `porter-provider` (`Port`), `cua-action`, quire's `ds-settings` (feature `live`, as accountd's, for the model picker), and stoker's `model-provider`, `model-catalog`, `engine-spec`, `engine-manifest` and `engine-probe` (the pinned manifest, where installs live, which engine is here and whether a person's own install is still the pinned one), `engine-supervisor`, `model-http` (features `client` and `tls`), `model-openai-compat`, `vision-prep` (feature `pixels`), `cua-parse`, `cua-session`, `cua-vendors`, `model-extract`, `model-replay`, `model-wire` (the `Driver` over inferd's body-shaping wrapper of `HttpClient`), `speech-provider`, `speech-host-client`, by git rev |
| `inferd-client` | `porter-core`, `porter-tailnet`, `porter-tailscale`, `porter-match`, `porter-capability`, `porter-wire`, `porter-ids`, `porter-infer`; `porter-dbus`, `porter-callers` and zbus with feature `dbus`; `porter-fs` with feature `lending`; with feature `engines` `porter-bridge`, `porter-turns` (the turn the daemon runs, `local::run_chat`, and the clock its retries wait on, `local::TokioSleep`) and stoker's `model-http` (`hyper`, `tls`), `model-openai-compat`, `model-provider`, `model-wire` |
| `porter-infer` | `porter-core`, `porter-match`, `porter-capability`, `porter-ids`, `cua-action` (stoker's computer-use vocabulary, by git rev), and stoker's `model-provider` (`Money`, the type of a reported charge), `rust_decimal` |
| `porter-bridge` | `porter-core`, `porter-capability`, `porter-match`, `porter-ids`, `porter-infer`, and stoker's `model-provider`, `model-catalog`, `model-openai-compat`, `vision-prep` (pure: the one mapping between porter's wire types and stoker's turns, shared by inferd and porter-client's `engines`) |
| `porter-router` | `porter-core`, `porter-tailnet`, `porter-tailscale`, `porter-match`, `porter-capability`, `porter-ids`, `porter-infer`, `cua-action`, and stoker's `engine-spec` (`Engine`: the words, flags and environment each engine kind has; `EnginePaths`; `check_entry` after the catalogue parser), `engine-probe` (`pointed`: the Python of a Strata folder), `engine-supervisor`, `model-catalog`, `model-http` (types only: no `client`, no `tls`), `model-openai-compat`, `model-provider`, `speech-provider`, `vision-prep`; rustix (the owner check of a key file). Reaches no zbus, no runtime and never `inferd` (checked by check-boundary.sh) |
| `porter-turns` | `porter-core`, `porter-match`, `porter-capability`, `porter-ids`, `porter-infer`, `porter-bridge`, `porter-router`, and stoker's `cua-action`, `cua-parse`, `cua-session`, `cua-vendors`, `engine-supervisor`, `model-extract`, `model-http` (feature `client`, the client to a local engine), `model-openai-compat`, `model-provider`, `vision-prep` (feature `pixels`); tokio. Reaches no zbus and never `inferd` (checked by check-boundary.sh) |
| `porter-tailnet` | `porter-ids`, `porter-tailscale`; with feature `persist`, `porter-fs` and toml (`Guests::open`); with feature `io`: hyper, hyper-util, http-body-util and tokio for the sockets |
| `steward-dbus` | `org.quire.Inference1` and `org.quire.Inference1.Agents` written once under `#[bus_interface]`: the `Inference` and `Agents` proxies, the `InferenceServer`/`AgentsServer` traits inferd implements and the objects that serve them, the introspection `Document`s (checked against `dbus/*.xml` by `tests/it/introspection.rs`), the bus vocabulary (option, key and error-name constants; `INFERENCE_BUS` and `INFERENCE_PATH` are `porter_wire::names`', re-exported here) and the row codecs (`NeedArg`, `LocalEngine`, `RunningModel`). The shared helpers (`Details`, `Call`, `Document`, `Unserved`, the vardict codec) are porter-dbus's; `#[bus_interface]` is porter-dbus-macros's. Cut from `porter_dbus::inference` | the bus (zbus types only; no connection of its own) |
| `inferd-probe` | the port probe of local-engine discovery: which runtime (Ollama, llama.cpp, LM Studio, another OpenAI-compatible server) answers on a loopback port and what models it offers, over the `Http` seam. Cut from `porter_discover::probe_ports` | none (the `Http` seam) |

Allowed direct edges (checked by `scripts/check-boundary.sh`; dev-dependencies are outside it) are the `EDGES` table in that script: one row per crate, the crates of ours, porter, stoker and quire it names directly, and nothing else.

External boundaries (default features, measured by `cargo tree`, enforced by check-boundary): `porter-infer`,
`porter-bridge`, `porter-tailnet`, `inferd-client` and `inferd-probe` never reach `zbus`, `zvariant`, `tokio`,
`reqwest`, `hyper`, `ureq`, `oo7`, `keyring`, `secret-service`, `interprocess`, `latchkey` or `ds-settings`
(`porter-tailnet` reaches `tokio` and `hyper` only through its `io` feature; `inferd-client` through `dbus`,
`engines` and `lending`). `porter-router` is the same and never reaches `inferd`; `porter-turns` may reach
`tokio` and `hyper` (its local-engine client) and nothing of the bus. `steward-dbus` reaches `zbus` and no
client, keyring or settings library. Only `porter-http-reqwest` (porter's) names `reqwest`. The libraries read
no environment variable; inferd reads it only in `from_env` functions.

Working on a dependency locally: put a git-ignored `[patch]` in this repository's `.cargo/config.toml` (never
commit it) pointing the porter, stoker or quire git URL at the checkout; a consumer that also takes them by path
needs the same `[patch]` in its own workspace root, or its graph holds two copies.

## 2. Who retries, who repairs

Decided with the rig amendment (the first appears in models.md section 4.2 once the spec is
edited; it is recorded here so the code and the spec agree):

- **inferd owns retry.** It sees the engine's state, so it retries a failed turn (a 5xx, a
  rate limit with its `Retry-After`, a not-ready engine that is loading) with stoker's
  `Retrying` wrapper and its pure `next_wait`, and never after the first event reached the
  client's sink: a half-delivered turn is not idempotent for the planner. A client sees either a
  reply or a `ModelError`; it never loops on `RateLimited` itself.
- **inferd owns validate-and-repair of structured output.** For `ReplyShape::Json` and
  `ReplyShape::Choice` it runs stoker's `ExtractSession` (`model-extract`): constrained decoding
  where the engine supports the shape, a synthetic tool call or a prompted schema where it does
  not, a bounded repair budget (the setting `ai.structured.repair_budget`, with `open_text`, `open_list` and `depth`: `[ai.structured]` of `inferd.toml`, read by `main` and passed down as `structured::Limits`; `config` therefore reads one type from `structured::limits`), and a check of the final text with `Shape::check`.
  A client (readerd, memoryd's consolidator, the action reviewer, the policy writer) gets either
  a `ChatReply.text` that already passes the check or `ModelError::Unparseable`; it parses into
  its type with a second, free check. A reply cut by `StopReason::MaxTokens` is never repaired.
  A repair prompt names the field and the expected shape and never echoes the model's output.
- **`ModelError::ContextOverflow` carries no limit.** The caller reads the context from the
  model's `LlmCap.context` (it has the card); stoker's `ProviderError::ContextOverflow { limit }`
  is mapped without it. `ProviderError::Server(status)` maps to `ModelError::Unreachable`
  (a gateway 5xx is a transient reach failure to an app).
- **Trace context.** `Inference1.Open`, `Prepare` and `Availability` take an `options` vardict
  whose reserved key is `traceparent` (`steward_dbus::inference::OPTION_TRACEPARENT`, a
  `porter_infer::Traceparent`); `Accounts::session_with` and `Transport::open_with` carry it. No
  content ever goes on a span (`prov::trace`).

## 3. inferd hosted

Hosted models (`inferd::cloud`): stoker's curated remote entries are candidates only for an app
that holds a grant on an account reaching them. `cloud::accountd` asks `Peer.Verdicts` once per
session and `Peer.ResolveKey` once per turn (a sealed memfd, read into a `SecretText`);
`cloud::models` picks each entry's reach with stoker's `reachable` (a company's own account before
OpenRouter) and makes the card (locality cloud, billing metered); `cloud::transport` is the TLS
`Transport` under stoker's `Driver<OpenAiCodec, _>`, `cloud::spend` the ledger and caps
(`ai.spend.*`: it adds what each reply's usage reports, its charge when the provider reported one and an unpriced turn when not, and a cap compares that reported total, so it stops the next turn and never the current one; there is no price table and no hold before a turn), `cloud::turn` one turn, `cloud::picker` the `cloud/<model>` choices. The picker's `KeySpec.groups` puts each hosted choice under its company and each on-device model under "On this computer" (`settings::module`), "" and `auto` ungrouped on top. The key is in
no event, log, audit line or file; `tests/cloud.rs` scans every bus message with a positive control.

A server of the person's own (`cloud::own`; the provider `openai-compatible`, "Other provider
(OpenAI-compatible)") is the other source of hosted models. Its account keeps the address the
person typed (a `ChatCompletions` `ServiceEndpoint`; `Account::endpoints`) and its key in the
Secret Service (an empty key when the server needs none). Each such account is its own provider
here: `Cloud::transport` takes its address from `OwnBook` (not from `Doors`), `OwnBook` keeps what
the server listed with `GET <address>/models` (the first time the account is seen, and on
`Rescan`; a list that did not come is asked for again after a minute), and each listed model is a
hosted `RemoteModel` whose context window, longest reply are 0, meaning not known. The
picker lists them "<model> on <account>" (`settings::pool::hosted`). The address rule is
`porter_core::server_address` (https for any host; plain http for this computer and the person's
own computers on their tailnet, which the sheet's machine learns from `Sheet::with_own_machines`,
read through `porter_service_accounts::OwnMachines`); the family (`api_key`) asks for `FieldKind::ServerUrl`
and an optional `ApiKey` when its provider file names no endpoint.

`cloud/<model>` and `local/<model>` have a third kind beside them, `agent/<program>`
(`porter_infer::ModelRef::agent`, `AGENT_ACCOUNT`): an outside coding agent the assistant hands the
whole conversation to, not a model inferd runs. It is a choice of the text slot only: `Set` of it
for another slot, or into `ai.auto.pool.*`, is a bad value, a file that says it is rejected and
falls back (`resolve`), and `TierMap::pick` never returns it for another slot or in a pool. inferd
checks only the word's shape; which agents exist is docket's (`agents.toml`), checked by Settings
(it lists them) and by the host that is handed the turn. A session whose pick names an agent ends
`Unavailable` with a `Declined` event whose reason is `DeclinedBecause::AgentChosen`: the
instruction to hand the turn to the agent, not a failure. The module groups a held agent word under
"Coding Agents" (`settings::module::CODING_AGENTS`). No wire changed.

`org.quire.Inference1` at `/org/quire/Inference1`, claimed by `inferd` on the session bus
(`dist/dbus/org.quire.Inference1.service` activates `dist/inferd.service`; `dist/inferd.toml` is the
annotated sample configuration). One `Open` is:

1. **Who.** `peers`: the bus names the sender's pid, `/proc/<pid>/exe` names the program, the
   caller table names the `Caller` (an `AppId` and a `Role`). A sender the table does not name gets
   `org.freedesktop.DBus.Error.AccessDenied`. The role `Cua` (cuad's executable) is the only one
   that may open a computer-use session; any other caller's route is `Refused(Denied)`.
2. **Where.** A socketpair; one end goes back as the reply descriptor, `serve_session` runs on the
   other over this session's four seams:
   - `Router` is `engines::SessionRouter`: `Engines::route` lists the local models (the stoker
     catalog through `local`) with their readiness from the supervisor's snapshot, then
     `router::choose` filters by the need (`porter_core::matches`), readiness, consent (a model on
     this computer is granted; anything else asks) and `porter_infer::route` with the user's
     policy and tier map. The decision pins the runner through a `Pin` cell.
   - `EngineHost` is `Engines`: `want` asks the `Supervised` handle, which owns stoker's
     `Supervisor` and carries out its effects over `engine_supervisor::{EngineHost, ReadyProbe,
     GpuProbe}` (`ProcessHost`, `HealthProbe`, `NvidiaSmi` in the daemon; fakes in tests). Engines
     unload on the supervisor's idle timer, so `release` does nothing.
   - `TurnRunner` is `runner::Turns`: stoker's `OpenAiCompat` (`Driver<OpenAiCodec, HttpClient>`)
     over the engine's Unix socket, wrapped in `Retrying`; `bridge` maps both ways; computer-use
     steps are `cua_step` over stoker's `CuaSession`; a reply of a shape inferd can read is run
     under `structured` (a `ShapedSession`: validate, repair once); a `Transcribe` turn is
     `speech::SpeechRunner` over `SttBackend::SpeechHost` (stoker's `speech-host-client` to the supervised
     speech host, a CPU engine probed with the host protocol's `Hello`): the session's audio frames
     reach the turn through a channel and the host's events leave as `Heard` deltas.
   - `AuditSink` is `audit::SessionAudit` over an `AuditOut` (a JSON-lines file in the daemon);
     the loop tells it what the request carried (`serve::Carried`: frames sent, audio milliseconds
     sent or produced) beside the reply.
3. **Away.** The first event is `Waiting(readiness)` when the engine is not ready, then `Routed`,
   then the turn's events and one `Finished` per request, exactly as `session::step` decides.

## 4. Caller identity (peephole)

`porter-callers::ProcCallers` identifies a bus sender with [peephole](https://github.com/PoHsuanLai/peephole)
(`BusPeers`, by git tag, never by path): the sender's process is held by pidfd, `/proc` and the user
manager state what it is, and `judge` returns a `Standing`. `standing.rs` turns the standing into a
`Caller`; `translate.rs` turns porter's table into peephole's.

- Fixed roles (`CallerRole::is_fixed`) need `Proof::Unit` or `Proof::Full` (main process of a unit from
  root-owned fragment and drop-ins, not transient, starting a program of the row, loader variables unset;
  `Unit` where a sandbox hides the image, its `ExecStart` program root-only all the way up). With less
  the caller is a plain `App`. `Agent` and `App` units need only the row (`Proof::Cgroup`).
- A Flatpak app is `Proven(Flatpak)` (metadata checked against the cgroup's `app-flatpak-<id>-<n>.scope`;
  a disagreement is nobody), a snap `Proven(Snap)` when its name is a reverse-DNS app name, a program in
  `app-<id>-<n>.scope` `Proven(Unsandboxed)` (the name is a claim; the role is the table's row for the
  app, which names no unit). Every other program, and any image a person can write, is nobody.
- Table translation. A `[[caller]]` row with a `unit` and at least one program becomes a `[[peer]]` row of
  the same app, unit and role (porter's own role spelling), its `exe` being the row's `exe` or, for a
  daemon unit named `<name>.service` (lower case, digits, dashes), `libexec/quire/<name>` under `/usr`,
  `/usr/local` and `/opt/quire/releases/*` (`images::installed`). Any other unit lists its `exe` in the row
  (`dist/callers.toml` does for detent and sill, in `bin`), and without one is nobody. Rows without a unit
  are not translated. One row for the unit manager (`systemd`, role `manager`, `Exe`) is added, and the
  manager's answers are believed only while its owner holds it. A user row can name `exe` but only gets
  a non-fixed role, and only if the program is root-owned.
- Tests: the daemons' `test-proc-root` features turn on porter-callers' `test-proc-root`, which turns on
  peephole's `fake` and adds `ProcCallers::with_proc_root` and `caller_of_pid` over a directory tree
  (`fixture.rs`). Never in a default build; check-boundary.sh checks it.

## 5. Installing engines

Settings (detent) installs llama.cpp, vLLM and Strata; inferd only finds them. inferd's unit has no
network, no write access to the person's files and no capabilities, and it holds the AI keys, so it
is not a place to download and install from: it is never given more.

- What is installed comes from stoker's `engine-manifest` compiled-in manifest alone. A caller names an engine
  and a backend (or nothing: `Manifest::choose` picks the backend from the cards found); the address,
  hash, size and seal come from the row. The privileged helper accepts those two words and nothing
  else, refuses any other argument before touching anything, and runs only through `pkexec` under the
  polkit action `org.quire.engines.install`. The helper (`engine-helper`, from stoker's `engine-install`)
  and that action are stoker's: stoker's `scripts/install.sh` installs both; steward ships neither.
- For everyone: the distro's package when the manifest has one for this release and its manager is
  here (and the pinned download if the package fails), else the pinned build, unpacked as root into
  `/usr/local/lib/porter/engines/<engine>/<release>-<backend>/`. A file of root's in a directory
  chain of root's is what `inferd::config::ProgramSearch` already trusts, so nothing there changes.
- For the person: the same build in `$XDG_DATA_HOME/porter/engines/...`. Anything running as the
  person can write there, so inferd checks the install before each start (`inferd::installed`, as a
  `LaunchGuard` of `ProcessHost`): an archive install must have exactly the tree seal pinned in the
  manifest, so any changed, added or removed file refuses the start (`Cause::ProgramChanged`) and the
  engine stands as changed until installed again. The pins are compiled in, so the check does not
  rest on anything the person's own programs can write.
- A Python environment (vLLM) cannot be pinned whole before it is built. Its Python (a pinned
  python-build-standalone tree) and `uv` are pinned by seal and hash, its wheels by a lock with every
  hash required, and what is checked at each start is the Python tree against its pin and the files
  that can run code at start (launchers, `pyvenv.cfg`, `.pth` and `sitecustomize` files, vLLM's own
  package) against the install's record; the whole environment is checked against the record in the
  background afterwards. The record is written beside the files, so this part is a tripwire for a
  change after the install, not a pin: a program that rewrites both an environment and its record is
  not caught. For everyone's install the record does not matter, because root holds it.
- Residual risk of the person's install: the check and the start are two steps, so a file swapped
  between them is not caught; the engines also run in inferd's sandbox either way.
