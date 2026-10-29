//! A need survives its D-Bus shape, and the shape is what the introspection says: a kind slug and
//! a vardict whose entries are the need's fields by name.

use porter_capability::{
    Access, AgentProgram, AgentProtocol, Albums, CuaEnv, Delta, LibraryRead, LlmFeature, Modality,
    SpeechMode, StorageScope,
};
use porter_ids::{Bytes, Count, Dims, Px, Tokens};
use porter_match::Need;
use porter_match::{
    AgentNeed, CuaNeed, DimsNeed, EmbedNeed, IdentityNeed, ImageGenNeed, KeyValueNeed, LlmNeed,
    MailNeed, NotesNeed, PhotosNeed, PimNeed, PushNeed, RerankNeed, SpeechNeed, StorageNeed,
};
use steward_dbus::inference::{NeedArg, need_from_dbus, need_to_dbus};
use zbus::zvariant::{OwnedValue, Value};

fn every_need() -> Vec<(&'static str, Need)> {
    let pim = PimNeed::new(Access::Read, Delta::Poll);
    vec![
        (
            "identity",
            Need::Identity(IdentityNeed::new().with_profile()),
        ),
        (
            "mail",
            Need::Mail(MailNeed::new(Access::Read, true, Delta::Push)),
        ),
        ("calendar", Need::Calendar(pim.clone())),
        ("contacts", Need::Contacts(pim.clone())),
        ("tasks", Need::Tasks(pim)),
        (
            "notes",
            Need::Notes(NotesNeed::new(Access::ReadWrite, Delta::None)),
        ),
        (
            "storage",
            Need::Storage(StorageNeed::new(
                Access::ReadWrite,
                Delta::Poll,
                StorageScope::AppFolder,
                false,
            )),
        ),
        (
            "photos",
            Need::Photos(PhotosNeed::new(
                LibraryRead::None,
                true,
                Albums::None,
                false,
                Delta::None,
            )),
        ),
        (
            "llm",
            Need::Llm(LlmNeed::new(
                [LlmFeature::Vision, LlmFeature::Tools].into(),
                Tokens(32_000),
            )),
        ),
        (
            "embeddings",
            Need::Embeddings(EmbedNeed::new(
                DimsNeed::Exactly(Dims(768)),
                [Modality::Text, Modality::Image].into(),
            )),
        ),
        (
            "embeddings",
            Need::Embeddings(EmbedNeed::new(DimsNeed::Any, [Modality::Text].into())),
        ),
        (
            "speech",
            Need::Speech(SpeechNeed::new([SpeechMode::Stt, SpeechMode::Tts].into())),
        ),
        (
            "image_gen",
            Need::ImageGen(ImageGenNeed::new(
                [porter_capability::ImageMode::Inpaint].into(),
                Px(512),
            )),
        ),
        ("rerank", Need::Rerank(RerankNeed::new(Count(10)))),
        (
            "computer_use",
            Need::ComputerUse(CuaNeed::new([CuaEnv::Desktop, CuaEnv::Browser].into())),
        ),
        (
            "key_value",
            Need::KeyValue(KeyValueNeed::new(Delta::Poll, Bytes(1024))),
        ),
        ("push", Need::Push(PushNeed::new())),
        (
            "agent",
            Need::Agent(AgentNeed::new(
                AgentProgram::parse("claude-code").expect("program"),
                [AgentProtocol::AnthropicMessages].into(),
                true,
            )),
        ),
    ]
}

#[test]
fn every_need_round_trips_under_its_kind_slug() {
    for (slug, need) in every_need() {
        let (kind, details) = need_to_dbus(&need);
        assert_eq!(kind, slug, "{need:?}");
        assert_eq!(
            need_from_dbus((kind, details)),
            Ok(need.clone()),
            "{need:?}"
        );
    }
}

#[test]
fn a_needs_fields_are_entries_by_name() {
    let need = Need::Llm(LlmNeed::new([LlmFeature::Vision].into(), Tokens(32_000)));
    let (_, details) = need_to_dbus(&need);
    let mut names: Vec<&str> = details.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["context", "features"]);
    let context: &Value<'_> = &details["context"];
    assert_eq!(*context, Value::I64(32_000));
}

#[test]
fn a_need_survives_the_bus_signature() {
    // Through the real wire encoding, not just the in-memory value: `(sa{sv})`.
    let ctxt = zbus::zvariant::serialized::Context::new_dbus(zbus::zvariant::LE, 0);
    for (_, need) in every_need() {
        let arg = need_to_dbus(&need);
        let bytes = zbus::zvariant::to_bytes(ctxt, &arg).expect("encodes");
        let (back, _): (NeedArg, usize) = bytes.deserialize().expect("decodes");
        assert_eq!(need_from_dbus(back), Ok(need));
    }
}

#[test]
fn a_malformed_need_is_refused_not_guessed() {
    let (kind, mut details) = need_to_dbus(&every_need()[8].1);
    details.insert(
        "context".to_owned(),
        OwnedValue::try_from(Value::from("lots")).expect("value"),
    );
    assert!(need_from_dbus((kind.clone(), details.clone())).is_err());
    details.remove("context");
    assert!(need_from_dbus((kind, details)).is_err(), "a missing field");
}

#[test]
fn the_kind_slug_is_the_capability_kind_slug() {
    for (_, need) in every_need() {
        let slug = serde_json::to_value(need.kind())
            .expect("json")
            .as_str()
            .map(str::to_owned);
        assert_eq!(slug, Some(need_to_dbus(&need).0));
    }
}
