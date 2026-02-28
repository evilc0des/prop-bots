//! GUI automation broker adapter.
//!
//! Fallback for platforms without APIs — uses screen capture and input simulation.

pub struct GuiBroker;

impl GuiBroker {
    pub fn new() -> Self {
        Self
    }
}
