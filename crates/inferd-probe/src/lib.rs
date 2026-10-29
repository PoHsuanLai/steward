//! The port probe of inferd's local-engine discovery: which runtime answers on a loopback port
//! and what models it offers. Pure over the `Http` seam; no runtime of its own.

mod probe;
#[cfg(test)]
mod testing;

pub use probe::{ProbeHit, probe_ports};
