//! The checked-in introspection files are the interfaces as they are declared once. A change to a
//! signature changes the file in the same commit.

use std::path::PathBuf;
use steward_dbus::inference;

#[test]
fn checked_in_introspection_matches_the_interfaces() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dbus");
    for document in [inference::INFERENCE, inference::AGENTS] {
        let path = dir.join(document.file_name);
        let expected =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let actual = document.introspection();
        assert!(
            actual == expected,
            "{} differs from the interfaces; it should read:\n{actual}",
            path.display()
        );
    }
}

#[test]
fn inference1_xml_unchanged_members() {
    let xml = inference::INFERENCE.introspection();
    let members = [
        "<method name=\"Availability\">",
        "<method name=\"Open\">",
        "<method name=\"Prepare\">",
        "<method name=\"Usage\">",
        "<method name=\"Rescan\">",
        "<signal name=\"EnginesChanged\">",
        "<property name=\"Gpu\" type=\"s\" access=\"read\"/>",
    ];
    for member in members {
        assert!(xml.contains(member), "missing {member}");
    }
    // `Places` was added after the freeze: an additive method (no existing member, argument or
    // error changes), as `Sync1.ConfirmDiscard` was.
    assert!(xml.contains("<method name=\"Places\">"));
    assert!(xml.contains("<arg type=\"a(sa{sv})\" direction=\"out\"/>"));
    // So were `AddComputer` and `RemoveComputer`, for Settings.
    assert!(xml.contains("<method name=\"AddComputer\">"));
    assert!(xml.contains("<method name=\"RemoveComputer\">"));
    assert!(xml.contains("<arg name=\"models\" type=\"a(sa{sv})\" direction=\"in\"/>"));
    // And these, for lending models to the person's other computers over their Tailscale
    // network (all additive): the computers that could be added, adding one, and the computers
    // that ask to use this one (who is asking, the answer, taking an answer back).
    for member in [
        "<method name=\"Candidates\">",
        "<method name=\"AddTailnetComputer\">",
        "<method name=\"Guests\">",
        "<method name=\"AnswerGuest\">",
        "<method name=\"ForgetGuest\">",
        "<method name=\"SetHelperPool\">",
        "<signal name=\"GuestAsks\">",
        "<signal name=\"GuestsChanged\">",
        "<arg name=\"answer\" type=\"s\" direction=\"in\"/>",
    ] {
        assert!(xml.contains(member), "missing {member}");
    }
    // And `LocalEngines`, which engines inferd found on this computer (additive).
    assert!(xml.contains("<method name=\"LocalEngines\">"));
    // And `HardwareNotes`, facts about the machine beside a choice (helpers, swarm.md 6.4).
    assert!(xml.contains("<method name=\"HardwareNotes\">"));
    // And `Start`, `Stop`, `Running` and `RunningChanged`: starting and stopping the models
    // inferd runs itself, and seeing them run (additive).
    for member in [
        "<method name=\"Start\">",
        "<method name=\"Stop\">",
        "<method name=\"Running\">",
        "<signal name=\"RunningChanged\">",
    ] {
        assert!(xml.contains(member), "missing {member}");
    }
    let declared = xml.matches("<method ").count()
        + xml.matches("<signal ").count()
        + xml.matches("<property ").count();
    assert_eq!(
        declared,
        members.len() + 3 + 7 + 1 + 1 + 1 + 1 + 4,
        "no member beyond the frozen seven, Places, AddComputer, RemoveComputer, and the five \
         methods and two signals of lending models across the person's network, LocalEngines, \
         PointEngine, HardwareNotes, SetHelperPool, and Start, Stop, Running and RunningChanged"
    );
    assert_eq!(
        inference::COMPUTER_ERROR_PREFIX,
        "org.quire.Inference1.Error.Computer."
    );
}

#[test]
fn speech_and_computer_use_are_fd_only() {
    let xml = inference::INFERENCE.introspection();
    for word in ["Transcribe", "Speak", "Audio", "Cua", "Listen"] {
        assert!(
            !xml.contains(word),
            "{word} must travel on the Open fd, not as a member"
        );
    }
}

#[test]
fn inference_calls_carry_an_options_dict_for_the_reserved_traceparent() {
    let xml = inference::INFERENCE.introspection();
    assert_eq!(
        xml.matches("<arg name=\"options\" type=\"a{sv}\" direction=\"in\"/>")
            .count(),
        4,
        "Availability, Open, Prepare and HardwareNotes"
    );
    assert_eq!(inference::OPTION_TRACEPARENT, "traceparent");
}
