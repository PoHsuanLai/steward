//! `steward-rig-client --app-id <id> [--proc-root <dir>] open <need> [--class C] [--tier T]
//! [--usage U] [--prompt TEXT]`: acts as an app over the session bus it is given
//! (`DBUS_SESSION_BUS_ADDRESS`), opens an AI session through inferd (`{"result":"opened"}`) and,
//! with a prompt, runs one chat turn, one `{"event":..}` line per event. A refusal is
//! `{"result":"error","error":..}`; a turn refused for want of a permit carries `needs_grant`.
//!
//! `<need>` is `mail`, `storage`, `calendar`, `contacts`, `notes`, `llm` or a JSON `Need`;
//! `C`, `U` and `T` are the wire words (`files`, `photos`, `mail`, `public`, ...; `interactive`,
//! `background`; `fast`, `balanced`, `best`). Exit status 0 when the command did what it was
//! asked, 1 otherwise. `--proc-root` defaults to `$RIG_PROC_ROOT`.
//!
//! Identity: porter-rig's fixture (`porter_rig::fixture`). Test tooling, never installed.

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use porter_rig::fixture::Identity;
use std::path::PathBuf;
use std::process::ExitCode;
use steward_rig::open::{Open, failure, open};

/// An app that uses inferd, for a jailed scenario.
#[derive(Debug, Parser)]
#[command(name = "steward-rig-client", version)]
struct Args {
    /// The app this process is (`org.example.App`).
    #[arg(long)]
    app_id: String,
    /// Where the daemons read callers from (`INFERD_PROC_ROOT`): this process is named there.
    /// Defaults to `$RIG_PROC_ROOT`.
    #[arg(long)]
    proc_root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Opens an AI session and, with a prompt, runs one chat turn on it.
    Open(Open),
}

fn fail(why: impl std::fmt::Display) -> ExitCode {
    println!("{}", failure(why));
    ExitCode::FAILURE
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        // A word or need that is none is an error line like any other failure.
        Err(why) if why.kind() == ErrorKind::ValueValidation => return fail(why.render()),
        Err(why) => why.exit(),
    };
    let proc_root = args
        .proc_root
        .or_else(|| std::env::var_os("RIG_PROC_ROOT").map(PathBuf::from));
    // Named before the first call, removed when the command ends.
    let _identity = match &proc_root {
        Some(root) => match Identity::write(root, &args.app_id, std::process::id()) {
            Ok(identity) => Some(identity),
            Err(why) => return fail(format!("cannot write the identity: {why}")),
        },
        None => None,
    };
    let connection = match zbus::Connection::session().await {
        Ok(connection) => connection,
        Err(why) => return fail(format!("no session bus: {why}")),
    };
    let ok = match args.command {
        Command::Open(asked) => open(&connection, asked, &mut |line| println!("{line}")).await,
    };
    match ok {
        true => ExitCode::SUCCESS,
        false => ExitCode::FAILURE,
    }
}
