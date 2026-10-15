//! Portable skill snapshots; translations remain owned by the runtime.
use anyhow::Result;
pub use fenix_skills::Bundle;
#[cfg(test)]
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
impl crate::Runtime {
    pub fn skill_bundle(&self, name: &str) -> Result<Bundle> {
        let mut bundle = fenix_skills::Library::open(&self.skills_dir)?.export(name)?;
        if let Some(text) = self.saved_skill_translation(&bundle.files["SKILL.md"])? {
            bundle.files.insert("SKILL.es.md".into(), text);
        }
        bundle.validate()?;
        Ok(bundle)
    }
    pub fn install_skill_bundle(&self, bundle: &Bundle, name: &str) -> Result<()> {
        fenix_skills::Library::open(&self.skills_dir)?.install(bundle, name)?;
        self.store.delete("deleted_skills", name)?;
        self.translation_notify.notify_one();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn export_and_install_preserve_files_without_overwriting_or_following_links() -> Result<()>
    {
        let dir = tempfile::tempdir()?;
        let runtime =
            crate::Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
                .await?;
        let source = runtime.skills_dir.join("private-example");
        fs::create_dir_all(source.join("references"))?;
        fs::write(
            source.join("SKILL.md"),
            "description: Reports\n\nRead references/guide.md.\n",
        )?;
        fs::write(
            source.join("references/guide.md"),
            "Summarize the supplied notes.\n",
        )?;
        let bundle = runtime.skill_bundle("private-example")?;
        runtime.install_skill_bundle(&bundle, "installed-example")?;
        assert_eq!(
            runtime.skill_bundle("installed-example")?.files,
            bundle.files
        );
        assert!(
            runtime
                .install_skill_bundle(&bundle, "installed-example")
                .is_err()
        );
        assert!(runtime.install_skill_bundle(&bundle, "../escape").is_err());
        std::os::unix::fs::symlink(dir.path(), source.join("outside"))?;
        assert!(runtime.skill_bundle("private-example").is_err());
        fs::remove_file(source.join("outside"))?;
        std::os::unix::fs::symlink(source.join("references/guide.md"), source.join("linked.md"))?;
        assert!(runtime.skill_bundle("private-example").is_err());
        runtime.shutdown().await?;
        Ok(())
    }
    #[test]
    fn hash_covers_resources_permissions_and_invalid_paths_are_rejected() {
        let mut b = Bundle {
            name: "example".into(),
            files: BTreeMap::from([
                ("SKILL.md".into(), "description: Example".into()),
                ("script.sh".into(), "printf hello".into()),
            ]),
            executable: BTreeSet::new(),
        };
        b.validate().unwrap();
        let original = b.hash();
        b.executable.insert("script.sh".into());
        assert_ne!(b.hash(), original);
        b.files.insert("script.sh".into(), "printf changed".into());
        assert_ne!(b.hash(), original);
        for path in [
            "../escape",
            "a/../escape",
            "/absolute",
            ".secret",
            "a\\b",
            "a//b",
            "a/./b",
        ] {
            let mut invalid = b.clone();
            invalid.files.insert(path.into(), "x".into());
            assert!(invalid.validate().is_err(), "{path}");
        }
    }
}
