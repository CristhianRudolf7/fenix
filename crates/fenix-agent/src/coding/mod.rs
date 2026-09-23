//! Repository-aware coding tools. No model calls or embeddings are used here.
mod environment;
mod instructions;
pub(crate) mod lsp;
pub(crate) mod patch;
mod search;
mod symbols;

#[cfg(test)]
mod harness_tests;
