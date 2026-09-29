//! Account-local manual editing with atomic writes and optimistic concurrency.
use anyhow::{Result, ensure};
use fs2::FileExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
};

fn pinned(file: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
fn read(dir: &Path) -> Result<(String, String)> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(dir.join("SKILL.md"))?;
    let meta = file.metadata()?;
    ensure!(
        meta.is_file() && meta.nlink() == 1 && meta.len() <= 32_000,
        "Invalid skill file"
    );
    let mut content = String::new();
    file.take(32_001).read_to_string(&mut content)?;
    ensure!(content.len() <= 32_000, "Skill source is too large");
    let revision = format!("{:x}", Sha256::digest(content.as_bytes()));
    Ok((content, revision))
}
impl crate::Runtime {
    pub fn editable_skill(&self, name: &str) -> Result<Value> {
        ensure!(super::valid_name(name), "Invalid skill name");
        let root = super::bundle::directory(&self.skills_dir)?;
        let dir = super::bundle::directory(&pinned(&root).join(name))?;
        let (content, revision) = read(&pinned(&dir))?;
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
        let root = super::bundle::directory(&self.skills_dir)?;
        root.lock_exclusive()?;
        ensure!(
            self.editable_skill_language(name, language)?["revision"] == expected,
            "This skill changed elsewhere. Reopen it before saving your changes."
        );
        let dir = super::bundle::directory(&pinned(&root).join(name))?;
        let path = pinned(&dir);
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
        let mut temporary = tempfile::NamedTempFile::new_in(&path)?;
        temporary.write_all(english.as_bytes())?;
        temporary.as_file().sync_all()?;
        temporary.persist(path.join("SKILL.md"))?;
        dir.sync_all()?;
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
        let root = super::bundle::directory(&self.skills_dir)?;
        root.lock_exclusive()?;
        let dir = super::bundle::directory(&pinned(&root).join(name))?;
        let path = pinned(&dir);
        let (_, current) = read(&path)?;
        ensure!(
            current == expected,
            "This skill changed elsewhere. Reopen it before saving your changes."
        );
        if format!("{:x}", Sha256::digest(content.as_bytes())) != current {
            let mut temporary = tempfile::NamedTempFile::new_in(&path)?;
            temporary.write_all(content.as_bytes())?;
            temporary.as_file().sync_all()?;
            temporary.persist(path.join("SKILL.md"))?;
            dir.sync_all()?;
            self.translation_notify.notify_one();
        }
        let revision = format!("{:x}", Sha256::digest(content.as_bytes()));
        // Saved translations are keyed by the source hash. The existing background
        // worker prepares the new Spanish document without blocking each keystroke.
        Ok(json!({"revision":revision,"language":"en","saved":true}))
    }
    pub fn delete_private_skill(&self, name: &str, expected: &str) -> Result<()> {
        ensure!(super::valid_name(name), "Invalid skill name");
        let root = super::bundle::directory(&self.skills_dir)?;
        root.lock_exclusive()?;
        let root_path = pinned(&root);
        let dir = super::bundle::directory(&root_path.join(name))?;
        let (_, current) = read(&pinned(&dir))?;
        ensure!(
            current == expected,
            "This skill changed elsewhere. Reopen it before deleting it."
        );
        // Keep default installation/migration from recreating a deleted skill.
        let removed = tempfile::Builder::new()
            .prefix(".deleted-")
            .tempdir_in(&root_path)?;
        self.store.put("deleted_skills", name, &name)?;
        if let Err(error) = fs::rename(root_path.join(name), removed.path()) {
            self.store.delete("deleted_skills", name)?;
            return Err(error.into());
        }
        root.sync_all()?;
        removed.close()?;
        Ok(())
    }
}
