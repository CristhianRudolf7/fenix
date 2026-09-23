//! cargo run -p fenix-storage --example memory_benchmark
//! Temporary isolated corpus; no production credentials, writes or model calls.
#[allow(dead_code)]
#[path = "../tests/support/memory_corpus.rs"]
mod support;
fn main() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let store = fenix_storage::Store::open(&dir.path().join("benchmark.db"))?;
    let corpus = support::corpus(&store);
    println!(
        "{}",
        serde_json::to_string_pretty(&store.evaluate_memory_retrieval(&corpus.cases, 5)?)?
    );
    Ok(())
}
