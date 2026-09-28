//! One operator-owned taxonomy for private skills and published bundles.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub id: String,
    pub labels: BTreeMap<String, String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}
fn key(value: &str) -> String {
    value.trim().to_lowercase()
}
pub fn parse(text: &str) -> Result<Vec<Category>> {
    let values: Vec<Category> = serde_json::from_str(text)?;
    ensure!(
        !values.is_empty() && values.len() <= 100,
        "Invalid category catalog size"
    );
    let mut names = BTreeSet::new();
    for c in &values {
        ensure!(
            !c.id.is_empty()
                && c.id.len() <= 40
                && c.id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "Invalid category ID"
        );
        ensure!(
            ["en", "es"].iter().all(|lang| c
                .labels
                .get(*lang)
                .is_some_and(|s| !s.trim().is_empty() && s.len() <= 100)),
            "Category needs English and Spanish labels"
        );
        for name in std::iter::once(&c.id).chain(c.aliases.iter()) {
            ensure!(
                name.len() <= 100 && names.insert(key(name)),
                "Duplicate category ID or alias: {name}"
            );
        }
    }
    ensure!(
        values.iter().any(|c| c.id == "other"),
        "The other category is required"
    );
    Ok(values)
}
pub fn catalog() -> Result<Vec<Category>> {
    if let Some(path) = std::env::var_os("FENIX_SKILL_CATEGORIES") {
        let path = std::path::PathBuf::from(path);
        ensure!(
            std::fs::metadata(&path)?.len() <= 64 * 1024,
            "Category catalog is too large"
        );
        parse(&std::fs::read_to_string(path)?)
    } else {
        parse(include_str!("../../../resources/skill-categories.json"))
    }
}
pub fn resolve(value: &str) -> Result<String> {
    let name = key(value);
    catalog()?
        .into_iter()
        .find(|c| c.id == name || c.aliases.iter().any(|a| key(a) == name))
        .map(|c| c.id)
        .context("Unknown skill category; choose an ID from the skill category catalog")
}
pub fn normalize(value: &str) -> Result<String> {
    // Legacy/manual packages remain usable, without growing the taxonomy.
    let values = catalog()?;
    let name = key(value);
    Ok(values
        .into_iter()
        .find(|c| c.id == name || c.aliases.iter().any(|a| key(a) == name))
        .map_or_else(|| "other".into(), |c| c.id))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases_share_one_identifier() {
        for s in ["Programación", " desarrollo ", "development", "engineering"] {
            assert_eq!(resolve(s).unwrap(), "engineering");
        }
        assert!(resolve("invented-new-category").is_err());
        assert_eq!(normalize("invented-new-category").unwrap(), "other");
    }
    #[test]
    fn duplicate_aliases_are_rejected() {
        assert!(
            parse(r#"[{"id":"other","labels":{"en":"Other","es":"Otros"},"aliases":["other"]}]"#)
                .is_err()
        );
    }
}
