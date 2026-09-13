//! HTTP model integrations with explicit, isolated credential resolution.
//!
//! Conversation types belong to `fenix-core`; secrets belong to
//! `fenix-credentials`. This crate supplies configuration, protocols, and HTTP.

#![doc = include_str!("../../../docs/en/development.md")]

pub mod catalog;
mod client;
mod config;
mod error;
pub mod protocol;
mod streaming;

pub use client::{ClientOptions, ProviderClient, ProviderConfig};
pub use config::{ConfigurationError, CredentialConfig, ProjectConfig, ProvidersConfig};
pub use error::ProviderError;
