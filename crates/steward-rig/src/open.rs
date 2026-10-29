//! `open`: an AI session through inferd, the way an app opens one.

use inferd_client::{DbusTransport, InferSession, Inference, OpenOptions};
use porter_capability::{Access, Delta, StorageScope, Tier};
use porter_core::DataClass;
use porter_core::consent::Usage;
use porter_ids::Tokens;
use porter_infer::{
    ChatControl, ChatMessage, ChatRequest, ClientFrame, InferEvent, InferRequest, MessagePart,
    Role, ToolParallelism,
};
use porter_match::{LlmNeed, MailNeed, Need, NotesNeed, PimNeed, StorageNeed};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// A wire word (`files`, `interactive`, `fast`, ...) read as `T`.
pub fn word<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_json::from_value(Value::String(text.to_owned()))
        .map_err(|_| format!("`{text}` is not one of the words"))
}

/// The need a command line names: a preset (`mail`, `storage`, `calendar`, `contacts`,
/// `notes`, `llm`), each asking for the least that is useful, or a JSON `Need` (porter-match's
/// serde form, `{"kind":"storage","v":{...}}`).
pub fn parse_need(text: &str) -> Result<Need, String> {
    if text.trim_start().starts_with('{') {
        return serde_json::from_str(text).map_err(|e| format!("not a Need: {e}"));
    }
    let pim = || PimNeed::new(Access::Read, Delta::None);
    Ok(match text {
        "mail" => Need::Mail(MailNeed::new(Access::Read, false, Delta::None)),
        "storage" => Need::Storage(StorageNeed::new(
            Access::ReadWrite,
            Delta::Poll,
            StorageScope::AppFolder,
            false,
        )),
        "calendar" => Need::Calendar(pim()),
        "contacts" => Need::Contacts(pim()),
        "notes" => Need::Notes(NotesNeed::new(Access::Read, Delta::None)),
        "llm" => Need::Llm(LlmNeed::new(Default::default(), Tokens(1_000))),
        other => {
            return Err(format!(
                "`{other}` is not a need (mail, storage, calendar, contacts, notes, llm, or JSON)"
            ));
        }
    })
}

/// What `open` asks for.
#[derive(Debug, clap::Args)]
pub struct Open {
    /// The need: `mail`, `storage`, `calendar`, `contacts`, `notes`, `llm` or a JSON `Need`.
    #[arg(value_parser = parse_need)]
    pub need: Need,
    /// The data class.
    #[arg(long, default_value = "public", value_parser = word::<DataClass>)]
    pub class: DataClass,
    /// The tier.
    #[arg(long, default_value = "fast", value_parser = word::<Tier>)]
    pub tier: Tier,
    /// `interactive` or `background`.
    #[arg(long, default_value = "interactive", value_parser = word::<Usage>)]
    pub usage: Usage,
    /// What to say, if anything.
    #[arg(long)]
    pub prompt: Option<String>,
}

/// The one error line every failure prints.
pub fn failure(why: impl std::fmt::Display) -> Value {
    json!({ "result": "error", "error": why.to_string() })
}

fn chat(prompt: &str, open: &Open) -> InferRequest {
    InferRequest::Chat(
        ChatRequest::new(
            vec![ChatMessage {
                role: Role::User,
                parts: vec![MessagePart::Text(prompt.to_owned())],
            }],
            open.tier,
            open.class,
            open.usage,
        )
        .with_control(ChatControl::new().with_tool_calls(ToolParallelism::Many)),
    )
}

/// Opens the session (`{"result":"opened"}`) and, with a prompt, runs one turn: one
/// `{"event":..}` line per event until `Finished`. A refusal is a `{"result":"error",..}` line.
/// True when it did what it was asked.
pub async fn open(connection: &zbus::Connection, open: Open, emit: &mut impl FnMut(Value)) -> bool {
    let inference = Inference::over(DbusTransport::over(connection.clone()));
    let options = OpenOptions::default().with_usage(open.usage);
    let mut session = match inference
        .session_with(&open.need, open.class, open.tier, &options)
        .await
    {
        Ok(session) => session,
        Err(why) => {
            emit(failure(why));
            return false;
        }
    };
    emit(json!({ "result": "opened" }));
    let Some(prompt) = &open.prompt else {
        return true;
    };
    if let Err(why) = session
        .send(ClientFrame::Request(chat(prompt, &open)))
        .await
    {
        emit(failure(format!("cannot send: {why}")));
        return false;
    }
    loop {
        match session.next().await {
            Ok(event) => {
                let finished = matches!(event, InferEvent::Finished(_));
                emit(json!({ "event": serde_json::to_value(&event).unwrap_or(Value::Null) }));
                if finished {
                    return true;
                }
            }
            Err(why) => {
                emit(failure(format!("the session ended: {why}")));
                return false;
            }
        }
    }
}
