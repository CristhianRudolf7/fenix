//! Portable, bounded skill snapshots. No scripts are executed during export or import.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{OpenOptionsExt, PermissionsExt},
    },
    path::{Component, Path},
};

pub const MAX_BYTES: usize = 128 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub name: String,
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub executable: BTreeSet<String>,
}
impl Bundle {
    pub fn validate(&self) -> Result<()> {
        ensure!(super::valid_name(&self.name), "Invalid skill name");
        ensure!(
            !self.files.is_empty() && self.files.len() <= 128,
            "A publication supports 1–128 text files"
        );
        ensure!(self.files.contains_key("SKILL.md"), "SKILL.md is required");
        ensure!(
            self.files["SKILL.md"].len() <= 32_000 && !self.files["SKILL.md"].trim().is_empty(),
            "Invalid SKILL.md size"
        );
        ensure!(
            self.files.values().map(String::len).sum::<usize>() <= MAX_BYTES,
            "Skill exceeds 128 KiB"
        );
        super::metadata(&self.files["SKILL.md"])?;
        for (path, content) in &self.files {
            validate_path(path)?;
            ensure!(
                !content.contains('\0'),
                "Binary content requires manual review"
            );
            ensure!(
                !self
                    .files
                    .keys()
                    .any(|other| other.starts_with(&format!("{path}/"))),
                "Conflicting file paths"
            );
        }
        ensure!(
            self.executable
                .iter()
                .all(|path| self.files.contains_key(path)),
            "Unknown executable path"
        );
        Ok(())
    }
    pub fn hash(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).expect("Serializable bundle"))
        )
    }
}
fn validate_path(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && path.len() <= 240
            && !path.contains('\\')
            && !path.contains(':')
            && !path.chars().any(char::is_control),
        "Invalid resource path"
    );
    ensure!(
        path.split('/').count() <= 12
            && path
                .split('/')
                .all(|p| !p.is_empty() && !p.starts_with('.')),
        "Hidden or ambiguous paths require manual review"
    );
    ensure!(
        Path::new(path)
            .components()
            .all(|p| matches!(p, Component::Normal(_))),
        "Resource must remain inside skill"
    );
    Ok(())
}
pub(crate) fn directory(path: &Path) -> Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_DIRECTORY)
        .open(path)?)
}
fn collect(
    dir: &File,
    prefix: &str,
    bundle: &mut Bundle,
    bytes: &mut usize,
    entries: &mut usize,
) -> Result<()> {
    let anchored = format!("/proc/self/fd/{}", dir.as_raw_fd());
    for entry in fs::read_dir(&anchored)? {
        *entries += 1;
        ensure!(*entries <= 256, "Too many bundle entries");
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("Non UTF-8 filename"))?;
        let relative = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        validate_path(&relative)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir() {
            collect(
                &directory(&entry.path())?,
                &relative,
                bundle,
                bytes,
                entries,
            )?;
        } else {
            ensure!(
                metadata.is_file(),
                "Links and special files cannot be published"
            );
            ensure!(bundle.files.len() < 128, "Too many skill files");
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
                .open(entry.path())?;
            let metadata = file.metadata()?;
            ensure!(metadata.is_file(), "Expected a regular file");
            let mut content = String::new();
            file.take((MAX_BYTES - *bytes + 1) as u64)
                .read_to_string(&mut content)
                .context("Only inspectable UTF-8 text bundles can be published")?;
            *bytes += content.len();
            ensure!(*bytes <= MAX_BYTES, "Skill exceeds 128 KiB");
            if metadata.permissions().mode() & 0o111 != 0 {
                bundle.executable.insert(relative.clone());
            }
            bundle.files.insert(relative, content);
        }
    }
    Ok(())
}
impl crate::Runtime {
    pub fn skill_bundle(&self, name: &str) -> Result<Bundle> {
        ensure!(super::valid_name(name), "Invalid skill name");
        let root = directory(&self.skills_dir)?;
        let dir = directory(&Path::new(&format!("/proc/self/fd/{}", root.as_raw_fd())).join(name))?;
        let mut bundle = Bundle {
            name: name.into(),
            files: BTreeMap::new(),
            executable: BTreeSet::new(),
        };
        collect(&dir, "", &mut bundle, &mut 0, &mut 0)?;
        if let Some(text) = self.saved_skill_translation(&bundle.files["SKILL.md"])? {
            bundle.files.insert("SKILL.es.md".into(), text);
        }
        bundle.validate()?;
        Ok(bundle)
    }
    /// Installation is a new private copy. Existing user edits are never overwritten.
    pub fn install_skill_bundle(&self, bundle: &Bundle, name: &str) -> Result<()> {
        bundle.validate()?;
        ensure!(super::valid_name(name), "Invalid installation name");
        let root = directory(&self.skills_dir)?;
        let root_path = std::path::PathBuf::from(format!("/proc/self/fd/{}", root.as_raw_fd()));
        // Keep staging outside catalog visibility until every file is durable.
        let staging = tempfile::Builder::new()
            .prefix(".import-")
            .tempdir_in(&root_path)?;
        for (path, content) in &bundle.files {
            let destination = staging.path().join(path);
            fs::create_dir_all(destination.parent().context("Missing parent")?)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(if bundle.executable.contains(path) {
                    0o700
                } else {
                    0o600
                })
                .open(destination)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }
        // Atomic no-replace rename avoids overwriting a concurrent installation.
        nix::fcntl::renameat2(
            &root,
            staging.path(),
            &root,
            &root_path.join(name),
            nix::fcntl::RenameFlags::RENAME_NOREPLACE,
        )?;
        root.sync_all()?;
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
