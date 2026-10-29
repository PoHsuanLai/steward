//! steward's D-Bus API: `org.quire.Inference1` (inference and agents), each interface written
//! once under `#[bus_interface]` as the proxy its callers use, the trait inferd implements and the
//! object that serves it. The shared helpers (`Details`, `Call`, `Document`, the vardict codec)
//! are porter-dbus's.

mod json_value;

pub mod inference;
