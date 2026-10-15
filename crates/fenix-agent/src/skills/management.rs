//! Account-local manual editing with portable locks and optimistic concurrency.
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
impl crate::Runtime {
    pub fn editable_skill(&self, name: &str) -> Result<Value> {
        ensure!(super::valid_name(name), "Invalid skill name");
        let (content, revision) =
            fenix_skills::Library::open(&self.skills_dir)?.read_skill(name)?;
        Ok(json!({"content":content,"revision":revision,"language":"en"}))
    }
    /// UI revisions cover both languages, including manual Spanish corrections.
    pub fn editable_skill_language(&self, name: &str, language: &str) -> Result<Value> {
        ensure!(
            matches!(language, "en" | "es"),
            "Unsupported skill language"
        );
        let source = self.editable_skill(name)?;
        let english = source["content"].as_str().unwrap();
        let spanish = self.saved_skill_translation(english)?;
        let revision = format!(
            "{:x}",
            Sha256::digest(format!("{}\0{}", english, spanish.as_deref().unwrap_or("")).as_bytes())
        );
        let content = if language == "es" {
            let spanish = spanish.as_deref().ok_or_else(|| {
                anyhow::anyhow!("The Spanish document is not ready yet. Please try again shortly.")
            })?;
            let mut meta = serde_yaml_ng::Value::Mapping(
                super::metadata(english)?
                    .as_mapping()
                    .cloned()
                    .unwrap_or_default(),
            );
            let localized = super::metadata(spanish)?;
            for field in ["title", "description"] {
                if let Some(value) = localized.get(field) {
                    meta[field] = value.clone();
                }
            }
            format!(
                "---\n{}---\n\n{}\n",
                serde_yaml_ng::to_string(&meta)?,
                crate::skill_localization::markdown_body(spanish)
            )
        } else {
            english.to_owned()
        };
        Ok(json!({"content":content,"revision":revision,"language":language}))
    }
    pub async fn save_skill_language(
        &self,
        name: &str,
        expected: &str,
        content: &str,
        language: &str,
    ) -> Result<Value> {
        ensure!(
            matches!(language, "en" | "es"),
            "Unsupported skill language"
        );
        ensure!(
            !content.trim().is_empty() && content.len() <= 32_000,
            "Skill must contain between 1 and 32000 bytes"
        );
        super::metadata(content)?;
        let original = self.editable_skill_language(name, language)?;
        ensure!(
            original["revision"] == expected,
            "This skill changed elsewhere. Reopen it before saving your changes."
        );
        let translated = tokio::time::timeout(
            std::time::Duration::from_secs(110),
            self.translate_skill_document(content, if language == "es" { "en" } else { "es" }),
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Translation took too long; your changes have not been saved. Please retry."
            )
        })??;
        let (english, spanish) = if language == "es" {
            (translated.as_str(), content)
        } else {
            (content, translated.as_str())
        };
        ensure!(
            english.len() <= 32_000,
            "Translated skill is too large; changes were not saved"
        );
        let library = fenix_skills::Library::open(&self.skills_dir)?;
        let _lock = library.lock()?;
        ensure!(
            self.editable_skill_language(name, language)?["revision"] == expected,
            "This skill changed elsewhere. Reopen it before saving your changes."
        );

        let display = self.display_path(english)?;
        let mut meta = serde_yaml_ng::Value::Mapping(
            super::metadata(spanish)?
                .as_mapping()
                .cloned()
                .unwrap_or_default(),
        );
        meta["source_hash"] = display
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .into();
        meta["language"] = "es".into();
        let spanish = format!(
            "---\n{}---\n\n{}\n",
            serde_yaml_ng::to_string(&meta)?,
            crate::skill_localization::markdown_body(spanish)
        );
        // Publish the translation first; readers discover it only through the canonical hash.
        crate::write_private(&display, &spanish)?;
        let (_, current) = library.read_skill(name)?;
        library.replace_skill(name, &current, english)?;
        let revision = self.editable_skill_language(name, language)?["revision"].clone();
        Ok(json!({"revision":revision,"language":language,"saved":true}))
    }
    pub fn save_skill_source(&self, name: &str, expected: &str, content: &str) -> Result<Value> {
        ensure!(super::valid_name(name), "Invalid skill name");
        ensure!(
            !content.trim().is_empty() && content.len() <= 32_000,
            "Skill must contain between 1 and 32000 bytes"
        );
        super::metadata(content)?;
        let library = fenix_skills::Library::open(&self.skills_dir)?;
        let _lock = library.lock()?;

        let (_, current) = library.read_skill(name)?;
        ensure!(
            current == expected,
            "This skill changed elsewhere. Reopen it before saving your changes."
        );
        if format!("{:x}", Sha256::digest(content.as_bytes())) != current {
            library.replace_skill(name, expected, content)?;
            self.translation_notify.notify_one();
        }
        let revision = format!("{:x}", Sha256::digest(content.as_bytes()));
        // Saved translations are keyed by the source hash. The existing background
        // worker prepares the new Spanish document without blocking each keystroke.
        Ok(json!({"revision":revision,"language":"en","saved":true}))
    }
    pub fn delete_private_skill(&self, name: &str, expected: &str) -> Result<()> {
        ensure!(super::valid_name(name), "Invalid skill name");
        let library = fenix_skills::Library::open(&self.skills_dir)?;
        let _lock = library.lock()?;
        let (_, current) = library.read_skill(name)?;
        ensure!(
            current == expected,
            "This skill changed elsewhere. Reopen it before deleting it."
        );
        self.store.put("deleted_skills", name, &name)?;
        let removed = match library.detach(name, expected) {
            Ok(removed) => removed,
            Err(error) => {
                self.store.delete("deleted_skills", name)?;
                return Err(error);
            }
        };
        library.remove_detached(&removed)?;
        Ok(())
    }
}
