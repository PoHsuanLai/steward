# Conventions

steward follows the program's shared conventions, whose text lives in quire's `CONVENTIONS.md`
(identical in quire, sill, shell-host, palmrest and detent): types, traits, effects, errors,
tests, comments, change discipline, borrowing. Read it first. `ARCHITECTURE.md` here adds the
crate map and the edge rule; where the two disagree, `ARCHITECTURE.md` wins for steward. These
rules are porter's (cut from porter `e9e7cdcd`), trimmed to what applies to the crates here.

What steward adds or decides differently, each with its reason:

1. **Async in traits is native, never the `async-trait` crate.** A seam is
   `-> impl Future<Output = ...> + Send` (return-position `impl Trait`) or a plain `async fn` in
   its implementations, so every future can cross a multi-threaded runtime and dispatch is
   static. Only a trait used as `dyn` (an `Arc<dyn Machines>`) returns a hand-boxed
   `Pin<Box<dyn Future<Output = T> + Send + 'a>>` (inferd's `Boxed`).
   Events of a model turn go to a `ChatSink` callback, not a `Stream`: a turn is one future that
   pushes as it runs, dropping it cancels, and the sink ends it early with `Flow::Stop`. That is
   deliberate.
2. **Nothing ambient below the daemon.** The clock, the secret store, the consent prompter and
   the transports are passed in; only `inferd` reads the system clock, the environment or the
   bus, and it reads the environment only in `from_env` functions (`scripts/check-boundary.sh`).
   `porter-router` and `porter-turns` read no environment variable at all.
3. **steward depends on porter by git rev; porter never depends on steward.** Moving a porter
   pin is a lane of its own: the revs live in one place, the root `Cargo.toml`, and the lockfile
   is relocked in the same commit.
4. **One integration-test executable per crate.** A crate's integration tests are modules of
   `crates/<crate>/tests/it/main.rs` (`mod <topic>;`, one `mod common;` there, helpers used as
   `crate::common`); fixtures stay in `tests/fixtures/`. A test that starts its own test
   executable again as a fake child does so by module path (`shutdown::fake_inferd`), so it is a
   module too. Every `tests/it/main.rs` carries the one-line guard test
   `test_kit::guard::every_module_is_declared(env!("CARGO_MANIFEST_DIR"), "tests/it",
   include_str!("main.rs"))`, which fails when a file or module directory there is not declared.
   A separate `tests/<name>.rs` with a `[[test]]` entry in the crate's `Cargo.toml` and a one-line
   reason is for a test that sets process-wide state or needs a `required-features` (or a
   feature-less build) the rest must not. Dependencies build without debug info
   (`[profile.dev.package."*"]`).

Type rules (the owner's, 2026-10-28; quote them in review):

- `bool` is fine for fields, returns, locals and private fns. An enum only where a public fn
  would read `f(true, false)`, a third state is realistic, or the value is persisted or on the
  bus.
- A newtype only when it holds a rule (parsed or validated), a unit, or an id that must not be
  mixed up with another.
- An enum for a closed set the code branches on. A set that grows (issuers, files, ports) is
  data, in a provider file or a table.
- Behaviour that varies by case is a trait with default methods. Generics by default;
  `Box<dyn>` or `&'static dyn` when the case is picked from config, a file or the bus. The
  stored or bus id stays as it is, and one function maps it to the impl.
- One fact has one type: no twin enums (use `#[serde]` attributes or `From`), and a two-way
  outcome is a `Result` or an `Option`.
- A data format never changes with a refactor: every persisted or bus value round-trips byte
  for byte, and a type that was stored and becomes another gets a test that the stored text is
  read and written unchanged.
