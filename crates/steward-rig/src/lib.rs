//! The rig's app side of inference, for jailed scenarios: open an inferd session as an app and,
//! with a prompt, run one chat turn on it, printing JSON lines. Test tooling, never installed.
//!
//! The app's identity is porter-rig's fixture (`porter_rig::fixture`), so the `--proc-root` tree
//! a scenario starts accountd and inferd with names this process to both.

pub mod open;
