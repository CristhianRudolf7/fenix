//! Provider-independent conversation contracts for Fénix.
//!
//! These types have no network, credential storage, UI, or runtime dependency.

mod conversation;
mod provider;

pub use conversation::*;
pub use provider::*;
