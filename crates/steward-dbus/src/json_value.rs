//! The small helpers `inference::codec` needs that porter-dbus keeps to itself.

use porter_core::CoreError;
use serde_json::{Map, Value as Json};

pub(crate) fn bad(why: &str) -> CoreError {
    CoreError::MalformedValue(format!("D-Bus argument: {why}"))
}

pub(crate) fn into_object(value: Json) -> Option<Map<String, Json>> {
    match value {
        Json::Object(fields) => Some(fields),
        _ => None,
    }
}

pub(crate) fn into_string(value: Json) -> Option<String> {
    match value {
        Json::String(text) => Some(text),
        _ => None,
    }
}
